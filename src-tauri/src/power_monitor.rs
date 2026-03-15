//! macOS 电源事件监听模块
//!
//! 监听系统睡眠/唤醒事件，通过异步通道通知应用。

#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(test)]
use std::sync::Mutex as StdMutex;
use std::sync::OnceLock;
use tokio::sync::mpsc;

const K_IO_MESSAGE_CAN_SYSTEM_SLEEP: u32 = 0x1;
const K_IO_MESSAGE_SYSTEM_WILL_SLEEP: u32 = 0x2;
const K_IO_MESSAGE_SYSTEM_WILL_NOT_SLEEP: u32 = 0x4;
const K_IO_MESSAGE_SYSTEM_HAS_POWERED_ON: u32 = 0x8;

/// 电源事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    /// 即将进入睡眠
    WillSleep { notification_id: isize },
    /// 系统已唤醒
    DidWake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallbackEventKind {
    WillSleep,
    DidWake,
}

/// 当前电源状态：0=Awake, 1=Sleeping
static CURRENT_POWER_STATE: AtomicU8 = AtomicU8::new(0);

/// 全局事件发送器，用于在 C 回调中发送事件
static EVENT_SENDER: OnceLock<mpsc::UnboundedSender<PowerEvent>> = OnceLock::new();

/// 检查当前是否处于睡眠状态
pub fn is_sleeping() -> bool {
    CURRENT_POWER_STATE.load(Ordering::SeqCst) == 1
}

/// 设置电源状态
pub fn set_sleeping(sleeping: bool) {
    CURRENT_POWER_STATE.store(if sleeping { 1 } else { 0 }, Ordering::SeqCst);
}

fn parse_apple_clamshell_state(output: &str) -> Option<bool> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("\"AppleClamshellState\" =") {
            let value = rest.trim();
            if value.eq_ignore_ascii_case("Yes") {
                return Some(true);
            }
            if value.eq_ignore_ascii_case("No") {
                return Some(false);
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn read_clamshell_state() -> Option<bool> {
    let output = Command::new("/usr/sbin/ioreg")
        .args(["-r", "-k", "AppleClamshellState", "-d", "1"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    parse_apple_clamshell_state(&stdout)
}

#[cfg(target_os = "macos")]
pub fn is_clamshell_closed() -> bool {
    read_clamshell_state().unwrap_or(true)
}

#[cfg(not(target_os = "macos"))]
pub fn is_clamshell_closed() -> bool {
    false
}

#[cfg(test)]
static TEST_POWER_STATE_LOCK: OnceLock<StdMutex<()>> = OnceLock::new();

#[cfg(test)]
pub fn with_test_power_state_lock<T>(f: impl FnOnce() -> T) -> T {
    let lock = TEST_POWER_STATE_LOCK.get_or_init(|| StdMutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    let original = is_sleeping();
    let result = f();
    set_sleeping(original);

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CallbackDecision {
    emit_event: Option<CallbackEventKind>,
    set_sleeping: Option<bool>,
    allow_sleep: bool,
}

fn decide_power_callback_action(message_type: u32, sleeping: bool) -> CallbackDecision {
    match message_type {
        K_IO_MESSAGE_CAN_SYSTEM_SLEEP => CallbackDecision {
            emit_event: None,
            set_sleeping: None,
            allow_sleep: true,
        },
        K_IO_MESSAGE_SYSTEM_WILL_SLEEP => CallbackDecision {
            emit_event: if sleeping {
                None
            } else {
                Some(CallbackEventKind::WillSleep)
            },
            set_sleeping: Some(true),
            allow_sleep: false,
        },
        K_IO_MESSAGE_SYSTEM_HAS_POWERED_ON => CallbackDecision {
            emit_event: if sleeping {
                Some(CallbackEventKind::DidWake)
            } else {
                None
            },
            set_sleeping: Some(false),
            allow_sleep: false,
        },
        K_IO_MESSAGE_SYSTEM_WILL_NOT_SLEEP => CallbackDecision {
            emit_event: None,
            set_sleeping: Some(false),
            allow_sleep: false,
        },
        _ => CallbackDecision {
            emit_event: None,
            set_sleeping: None,
            allow_sleep: false,
        },
    }
}

/// 电源监听器
pub struct PowerMonitor {
    receiver: mpsc::UnboundedReceiver<PowerEvent>,
}

impl PowerMonitor {
    /// 创建新的电源监听器
    ///
    /// 此函数只能在 macOS 上调用，其他平台会 panic。
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        {
            let (tx, rx) = mpsc::unbounded_channel();

            // 存储发送器供回调使用
            EVENT_SENDER
                .set(tx)
                .expect("PowerMonitor should only be created once");

            // 启动 IOKit 监听
            unsafe {
                start_power_monitoring();
            }

            Self { receiver: rx }
        }

        #[cfg(not(target_os = "macos"))]
        {
            panic!("PowerMonitor is only supported on macOS");
        }
    }

    /// 接收下一个电源事件
    ///
    /// 返回 Some(PowerEvent) 如果收到事件
    /// 返回 None 如果通道关闭
    pub async fn recv(&mut self) -> Option<PowerEvent> {
        self.receiver.recv().await
    }
}

#[cfg(target_os = "macos")]
pub fn acknowledge_sleep(notification_id: isize) {
    unsafe {
        acknowledge_sleep_impl(notification_id);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn acknowledge_sleep(_notification_id: isize) {}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use core_foundation::base::CFTypeRef;
    use core_foundation::runloop::{
        kCFRunLoopDefaultMode, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
    };
    use mach::port::mach_port_t;
    use std::os::raw::c_void;
    use std::ptr;
    use std::sync::atomic::AtomicU32;
    use std::thread;

    // IOKit 类型定义
    #[allow(non_camel_case_types)]
    type io_object_t = mach_port_t;
    #[allow(non_camel_case_types)]
    type io_service_t = io_object_t;
    #[allow(non_camel_case_types)]
    type io_connect_t = io_object_t;
    #[allow(non_camel_case_types)]
    type io_notification_port_t = *mut c_void;
    #[allow(non_camel_case_types)]
    type IONotificationPortRef = io_notification_port_t;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IONotificationPortGetRunLoopSource(notify: IONotificationPortRef) -> CFTypeRef;
        fn IORegisterForSystemPower(
            refCon: *mut c_void,
            thePortRef: *mut IONotificationPortRef,
            callback: extern "C" fn(*mut c_void, io_service_t, u32, *mut c_void),
            notifier: *mut io_object_t,
        ) -> io_connect_t;
        fn IOAllowPowerChange(connect: io_connect_t, notification_id: isize) -> i32;
    }

    static POWER_CONNECTION: AtomicU32 = AtomicU32::new(0);

    fn extract_notification_id(message_argument: *mut c_void) -> isize {
        if message_argument.is_null() {
            0
        } else {
            message_argument as isize
        }
    }

    pub unsafe fn acknowledge_sleep_impl(notification_id: isize) {
        let connection = POWER_CONNECTION.load(Ordering::SeqCst);
        if connection != 0 {
            unsafe {
                let _ = IOAllowPowerChange(connection, notification_id);
            }
        }
    }

    extern "C" fn power_callback(
        _ref_con: *mut c_void,
        _service: io_service_t,
        message_type: u32,
        message_argument: *mut c_void,
    ) {
        let notification_id = extract_notification_id(message_argument);
        let decision = decide_power_callback_action(message_type, is_sleeping());

        if let Some(new_state) = decision.set_sleeping {
            set_sleeping(new_state);
        }

        if let Some(event_kind) = decision.emit_event {
            if let Some(tx) = EVENT_SENDER.get() {
                match event_kind {
                    CallbackEventKind::WillSleep => {
                        if tx.send(PowerEvent::WillSleep { notification_id }).is_err() {
                            unsafe {
                                acknowledge_sleep_impl(notification_id);
                            }
                        }
                    }
                    CallbackEventKind::DidWake => {
                        let _ = tx.send(PowerEvent::DidWake);
                    }
                }
            } else if event_kind == CallbackEventKind::WillSleep {
                unsafe {
                    acknowledge_sleep_impl(notification_id);
                }
            }
        }

        if decision.allow_sleep {
            unsafe {
                acknowledge_sleep_impl(notification_id);
            }
        }
    }

    pub unsafe fn start_power_monitoring() {
        thread::spawn(|| {
            let mut port: IONotificationPortRef = ptr::null_mut();
            let mut notifier: io_object_t = 0;

            let root_port = unsafe {
                IORegisterForSystemPower(ptr::null_mut(), &mut port, power_callback, &mut notifier)
            };

            if root_port == 0 {
                eprintln!("[PowerMonitor] Failed to register for system power notifications");
                return;
            }

            POWER_CONNECTION.store(root_port, Ordering::SeqCst);

            let run_loop_source = unsafe { IONotificationPortGetRunLoopSource(port) };
            let run_loop = CFRunLoopGetCurrent();

            CFRunLoopAddSource(run_loop, run_loop_source as _, kCFRunLoopDefaultMode);

            // 运行 RunLoop
            CFRunLoopRun();
        });
    }
}

#[cfg(target_os = "macos")]
use macos_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_action_for_can_system_sleep() {
        let decision = decide_power_callback_action(K_IO_MESSAGE_CAN_SYSTEM_SLEEP, false);
        assert_eq!(
            decision,
            CallbackDecision {
                emit_event: None,
                set_sleeping: None,
                allow_sleep: true,
            }
        );
    }

    #[test]
    fn test_decide_action_for_will_sleep_when_awake() {
        let decision = decide_power_callback_action(K_IO_MESSAGE_SYSTEM_WILL_SLEEP, false);
        assert_eq!(
            decision,
            CallbackDecision {
                emit_event: Some(CallbackEventKind::WillSleep),
                set_sleeping: Some(true),
                allow_sleep: false,
            }
        );
    }

    #[test]
    fn test_decide_action_for_wake_when_sleeping() {
        let decision = decide_power_callback_action(K_IO_MESSAGE_SYSTEM_HAS_POWERED_ON, true);
        assert_eq!(
            decision,
            CallbackDecision {
                emit_event: Some(CallbackEventKind::DidWake),
                set_sleeping: Some(false),
                allow_sleep: false,
            }
        );
    }

    #[test]
    fn test_decide_action_for_will_not_sleep() {
        let decision = decide_power_callback_action(K_IO_MESSAGE_SYSTEM_WILL_NOT_SLEEP, true);
        assert_eq!(
            decision,
            CallbackDecision {
                emit_event: None,
                set_sleeping: Some(false),
                allow_sleep: false,
            }
        );
    }

    #[test]
    fn test_power_state_tracking() {
        with_test_power_state_lock(|| {
            // 初始状态应该是 awake
            set_sleeping(false);
            assert!(!is_sleeping());

            // 设置为 sleeping
            set_sleeping(true);
            assert!(is_sleeping());

            // 设置为 awake
            set_sleeping(false);
            assert!(!is_sleeping());
        });
    }

    #[tokio::test]
    async fn test_power_event_equality() {
        assert_eq!(
            PowerEvent::WillSleep { notification_id: 1 },
            PowerEvent::WillSleep { notification_id: 1 }
        );
        assert_eq!(PowerEvent::DidWake, PowerEvent::DidWake);
        assert_ne!(
            PowerEvent::WillSleep { notification_id: 1 },
            PowerEvent::DidWake
        );
    }

    #[test]
    fn test_parse_apple_clamshell_state_yes() {
        let output = "\"AppleClamshellState\" = Yes";
        assert_eq!(parse_apple_clamshell_state(output), Some(true));
    }

    #[test]
    fn test_parse_apple_clamshell_state_no() {
        let output = "\"AppleClamshellState\" = No";
        assert_eq!(parse_apple_clamshell_state(output), Some(false));
    }

    #[test]
    fn test_parse_apple_clamshell_state_missing() {
        let output = "\"Wake Type\" = UserActivity Assertion";
        assert_eq!(parse_apple_clamshell_state(output), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_clamshell_closed_fails_closed_when_state_unavailable() {
        assert!(read_clamshell_state().is_some() || is_clamshell_closed());
    }
}

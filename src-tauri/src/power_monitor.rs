//! macOS 电源事件监听模块
//!
//! 监听系统睡眠/唤醒事件，通过异步通道通知应用。

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
use tokio::sync::mpsc;

/// 电源事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    /// 即将进入睡眠
    WillSleep,
    /// 系统已唤醒
    DidWake,
}

/// 当前电源状态：0=Awake, 1=Sleeping
static CURRENT_POWER_STATE: AtomicU8 = AtomicU8::new(0);

/// 全局事件发送器，用于在 C 回调中发送事件
static EVENT_SENDER: OnceLock<mpsc::Sender<PowerEvent>> = OnceLock::new();

/// 检查当前是否处于睡眠状态
pub fn is_sleeping() -> bool {
    CURRENT_POWER_STATE.load(Ordering::SeqCst) == 1
}

/// 设置电源状态
pub fn set_sleeping(sleeping: bool) {
    CURRENT_POWER_STATE.store(if sleeping { 1 } else { 0 }, Ordering::SeqCst);
}

/// 电源监听器
pub struct PowerMonitor {
    _receiver: mpsc::Receiver<PowerEvent>,
}

impl PowerMonitor {
    /// 创建新的电源监听器
    /// 
    /// 此函数只能在 macOS 上调用，其他平台会 panic。
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        {
            let (tx, rx) = mpsc::channel(10);
            
            // 存储发送器供回调使用
            EVENT_SENDER.set(tx).expect("PowerMonitor should only be created once");
            
            // 启动 IOKit 监听
            unsafe {
                start_power_monitoring();
            }
            
            Self {
                _receiver: rx,
            }
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
        self._receiver.recv().await
    }
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use core_foundation::base::{CFTypeRef, TCFType};
    use core_foundation::runloop::{CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun};
    use core_foundation::string::CFString;
    use core_foundation::string::CFStringRef;
    use mach::kern_return::{kern_return_t, KERN_SUCCESS};
    use mach::port::mach_port_t;
    use std::os::raw::c_void;
    use std::ptr;
    use std::thread;

    // IOKit 类型定义
    type io_object_t = mach_port_t;
    type io_service_t = io_object_t;
    type io_connect_t = io_object_t;
    type io_iterator_t = io_object_t;
    type io_notification_port_t = *mut c_void;
    type IONotificationPortRef = io_notification_port_t;

    // 电源消息类型
    const kIOMessageCanSystemSleep: i32 = 0x1;
    const kIOMessageSystemWillSleep: i32 = 0x2;
    const kIOMessageSystemWillNotSleep: i32 = 0x4;
    const kIOMessageSystemHasPoweredOn: i32 = 0x8;

    extern "C" {
        fn IONotificationPortCreate(allocator: CFTypeRef) -> IONotificationPortRef;
        fn IONotificationPortDestroy(notify: IONotificationPortRef);
        fn IONotificationPortGetRunLoopSource(notify: IONotificationPortRef) -> CFTypeRef;
        fn IORegisterForSystemPower(
            refCon: *mut c_void,
            thePortRef: *mut IONotificationPortRef,
            callback: extern "C" fn(*mut c_void, io_service_t, u32, *mut c_void),
            notifier: *mut io_object_t,
        ) -> kern_return_t;
        fn IODeregisterForSystemPower(notifier: io_object_t) -> kern_return_t;
        fn IOAllowPowerChange(connect: io_connect_t, notification_id: u32) -> kern_return_t;
        fn IOCancelPowerChange(connect: io_connect_t, notification_id: u32) -> kern_return_t;
    }

    static mut POWER_PORT: IONotificationPortRef = ptr::null_mut();
    static mut POWER_CONNECTION: io_object_t = 0;

    extern "C" fn power_callback(
        _ref_con: *mut c_void,
        _service: io_service_t,
        message_type: u32,
        message_argument: *mut c_void,
    ) {
        // 从 message_argument 提取 notification ID
        // message_argument 指向一个 i32，解引用获取通知 ID
        let notification_id = if message_argument.is_null() {
            0u32
        } else {
            unsafe { *(message_argument as *const i32) as u32 }
        };

        match message_type as i32 {
            kIOMessageCanSystemSleep | kIOMessageSystemWillSleep => {
                // 系统即将睡眠
                if !is_sleeping() {
                    if let Some(tx) = EVENT_SENDER.get() {
                        if tx.try_send(PowerEvent::WillSleep).is_ok() {
                            // 只有成功发送事件后才设置状态
                            set_sleeping(true);
                        }
                    }
                }
                // 允许系统睡眠 - 必须使用正确的 notification_id
                unsafe {
                    IOAllowPowerChange(POWER_CONNECTION, notification_id);
                }
            }
            kIOMessageSystemHasPoweredOn => {
                // 系统已唤醒
                if is_sleeping() {
                    if let Some(tx) = EVENT_SENDER.get() {
                        if tx.try_send(PowerEvent::DidWake).is_ok() {
                            // 只有成功发送事件后才设置状态
                            set_sleeping(false);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub unsafe fn start_power_monitoring() {
        thread::spawn(|| {
            let mut port: IONotificationPortRef = ptr::null_mut();
            let mut notifier: io_object_t = 0;
            
            let result = IORegisterForSystemPower(
                ptr::null_mut(),
                &mut port,
                power_callback,
                &mut notifier,
            );
            
            if result != KERN_SUCCESS {
                eprintln!("[PowerMonitor] Failed to register for system power notifications: {}", result);
                return;
            }
            
            POWER_PORT = port;
            POWER_CONNECTION = notifier;
            
            let run_loop_source = IONotificationPortGetRunLoopSource(port);
            let run_loop = CFRunLoopGetCurrent();
            
            let mode = CFString::from_static_string("kCFRunLoopDefaultMode");
            CFRunLoopAddSource(
                run_loop,
                run_loop_source as _,
                mode.as_concrete_TypeRef(),
            );
            
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
    fn test_power_state_tracking() {
        // 初始状态应该是 awake
        assert!(!is_sleeping());
        
        // 设置为 sleeping
        set_sleeping(true);
        assert!(is_sleeping());
        
        // 设置为 awake
        set_sleeping(false);
        assert!(!is_sleeping());
    }

    #[tokio::test]
    async fn test_power_event_equality() {
        assert_eq!(PowerEvent::WillSleep, PowerEvent::WillSleep);
        assert_eq!(PowerEvent::DidWake, PowerEvent::DidWake);
        assert_ne!(PowerEvent::WillSleep, PowerEvent::DidWake);
    }
}

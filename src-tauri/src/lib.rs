use scheduler::job_scheduler::{Scheduler, SchedulerState};
use scheduler::task_queue::TaskQueue;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent,
};
use tokio::sync::Mutex;

pub mod commands;
pub mod db;
pub mod environment;
pub mod execution_retention;
pub mod executor;
pub mod models;
pub mod opencode;
#[cfg(target_os = "macos")]
pub mod power_monitor;
pub mod scheduler;
pub mod storage;

use chrono::Utc;
use commands::execute_task_streaming;
use commands::test_channel_stream;
use commands::{create_task, delete_task, get_task, get_tasks, run_task, update_task};
use commands::{delete_output, get_output};
use commands::{get_execution, get_executions, get_running_executions};
use commands::{
    get_scheduler_status, load_scheduler_tasks, reload_scheduler, start_scheduler, stop_scheduler,
};
use models::execution::{get_executions_by_status, ExecutionStatus, UpdateExecution};

#[cfg(target_os = "macos")]
struct SleepAcknowledgeGuard {
    notification_id: isize,
    acknowledged: bool,
}

#[cfg(target_os = "macos")]
impl SleepAcknowledgeGuard {
    fn new(notification_id: isize) -> Self {
        Self {
            notification_id,
            acknowledged: false,
        }
    }

    fn acknowledge(&mut self) {
        if !self.acknowledged {
            power_monitor::acknowledge_sleep(self.notification_id);
            self.acknowledged = true;
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for SleepAcknowledgeGuard {
    fn drop(&mut self) {
        self.acknowledge();
    }
}

fn mark_running_as_failed_blocking(pool: &SqlitePool) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let count = runtime.block_on(async {
        let running_executions =
            match get_executions_by_status(pool, ExecutionStatus::Running).await {
                Ok(execs) => execs,
                Err(e) => {
                    eprintln!("Failed to get running executions: {}", e);
                    return 0;
                }
            };

        let mut updated_count = 0;
        let now = Utc::now().to_rfc3339();

        for execution in running_executions {
            let update = UpdateExecution {
                session_id: None,
                status: Some(ExecutionStatus::Failed),
                finished_at: Some(now.clone()),
                output_file: execution.output_file,
                error_message: Some("Application was terminated unexpectedly".to_string()),
            };

            if let Err(e) = models::execution::update_execution(pool, &execution.id, update).await {
                eprintln!("Failed to mark execution {} as failed: {}", execution.id, e);
            } else {
                updated_count += 1;
            }
        }

        updated_count
    });

    if count > 0 {
        println!("Marked {} running executions as failed", count);
    }
}

#[cfg(target_os = "macos")]
async fn handle_system_sleep(
    scheduler: &Arc<Mutex<Scheduler>>,
    pool: &Arc<SqlitePool>,
    notification_id: isize,
) {
    let mut sleep_ack_guard = SleepAcknowledgeGuard::new(notification_id);

    eprintln!("[PowerMonitor] System entering sleep, pausing scheduler...");

    // 1. Stop scheduler
    let scheduler_guard = scheduler.lock().await;
    if let Err(e) = scheduler_guard.stop().await {
        eprintln!("[PowerMonitor] Failed to stop scheduler: {}", e);
    }
    drop(scheduler_guard);

    scheduler::kill_all_processes();

    match get_executions_by_status(pool, ExecutionStatus::Running).await {
        Ok(running_executions) => {
            let now = Utc::now().to_rfc3339();
            for execution in running_executions {
                let update = UpdateExecution {
                    session_id: None,
                    status: Some(ExecutionStatus::Failed),
                    finished_at: Some(now.clone()),
                    output_file: execution.output_file,
                    error_message: Some("System entering sleep".to_string()),
                };

                if let Err(e) =
                    models::execution::update_execution_if_running(pool, &execution.id, update)
                        .await
                {
                    eprintln!(
                        "[PowerMonitor] Failed to mark execution {} as failed: {}",
                        execution.id, e
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("[PowerMonitor] Failed to get running executions: {}", e);
        }
    }

    sleep_ack_guard.acknowledge();

    eprintln!("[PowerMonitor] System sleep handling completed");
}

#[cfg(target_os = "macos")]
async fn wait_until_clamshell_open(max_wait: Duration, check_interval: Duration) -> bool {
    let start = tokio::time::Instant::now();

    while power_monitor::is_clamshell_closed() {
        if start.elapsed() >= max_wait {
            return false;
        }

        tokio::time::sleep(check_interval).await;
    }

    true
}

#[cfg(target_os = "macos")]
fn can_run_scheduler_now() -> bool {
    !power_monitor::is_sleeping() && !power_monitor::is_clamshell_closed()
}

#[cfg(target_os = "macos")]
async fn handle_system_wake(
    scheduler: &Arc<Mutex<Scheduler>>,
    pool: &Arc<SqlitePool>,
    task_queue: &Arc<Mutex<TaskQueue>>,
    app: &tauri::AppHandle,
) {
    if !can_run_scheduler_now() {
        eprintln!(
            "[PowerMonitor] Wake event while system still unavailable for scheduling, waiting for lid open"
        );
    }

    if power_monitor::is_clamshell_closed() {
        eprintln!(
            "[PowerMonitor] Wake event received while clamshell is closed, waiting for lid open before resuming scheduler"
        );

        let opened =
            wait_until_clamshell_open(Duration::from_secs(8 * 60 * 60), Duration::from_secs(2))
                .await;
        if !opened {
            eprintln!("[PowerMonitor] Timed out waiting for lid open, keeping scheduler paused");
            return;
        }

        eprintln!("[PowerMonitor] Lid opened, continuing scheduler resume");
    }

    eprintln!("[PowerMonitor] System waking up, resuming scheduler in 3 seconds...");

    // Wait 3 seconds for network to be ready
    tokio::time::sleep(Duration::from_secs(3)).await;

    if !can_run_scheduler_now() {
        eprintln!(
            "[PowerMonitor] System unavailable during wake stabilization, skipping scheduler resume"
        );
        return;
    }

    let (loaded_count, load_errors) = {
        let scheduler_guard = scheduler.lock().await;

        if scheduler_guard.get_state().await == SchedulerState::Running {
            if let Err(e) = scheduler_guard.stop().await {
                eprintln!(
                    "[PowerMonitor] Failed to stop scheduler before wake reload: {}",
                    e
                );
                return;
            }
        }

        scheduler_guard.clear_jobs().await;
        drop(scheduler_guard);

        match load_scheduler_tasks(pool, scheduler, task_queue, app).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("[PowerMonitor] Failed to reload tasks after wake: {}", e);
                return;
            }
        }
    };

    if !load_errors.is_empty() {
        eprintln!(
            "[PowerMonitor] Reloaded {} tasks after wake with {} errors: {}",
            loaded_count,
            load_errors.len(),
            load_errors.join(", ")
        );
    }

    if !can_run_scheduler_now() {
        eprintln!(
            "[PowerMonitor] System became unavailable before scheduler start, keeping paused"
        );
        return;
    }

    let scheduler_guard = scheduler.lock().await;
    if let Err(e) = scheduler_guard.start().await {
        eprintln!("[PowerMonitor] Failed to start scheduler: {}", e);
    }

    eprintln!("[PowerMonitor] System wake handling completed");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                if let Some(pool_state) = app.try_state::<Arc<SqlitePool>>() {
                    mark_running_as_failed_blocking(pool_state.inner());
                }
                scheduler::kill_all_processes();
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            get_task,
            create_task,
            update_task,
            delete_task,
            run_task,
            get_executions,
            get_execution,
            get_running_executions,
            get_scheduler_status,
            start_scheduler,
            stop_scheduler,
            reload_scheduler,
            get_output,
            delete_output,
            test_channel_stream,
            execute_task_streaming
        ])
        .setup(|app| {
            let db_path = db::connection::get_database_path(app.handle())?;
            let pool = tauri::async_runtime::block_on(async {
                db::connection::init_database(&db_path).await
            })
            .expect("Failed to initialize database");

            if let Ok(output_dir) = storage::output::get_output_directory(app.handle()) {
                tauri::async_runtime::block_on(async {
                    execution_retention::enforce_execution_history_limit(&pool, &output_dir).await;
                });
            }

            mark_running_as_failed_blocking(&pool);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            scheduler::cleanup_orphan_processes(&app_data_dir);

            let pool_arc = Arc::new(pool);
            app.manage(pool_arc.clone());

            let scheduler = Arc::new(Mutex::new(Scheduler::new()));
            app.manage(scheduler.clone());

            let task_queue = Arc::new(Mutex::new(TaskQueue::new()));
            app.manage(task_queue.clone());

            setup_tray(app)?;

            let scheduler_clone = scheduler.clone();
            let pool_clone = pool_arc.clone();
            let task_queue_clone = task_queue.clone();

            tauri::async_runtime::block_on(async {
                let (loaded_count, load_errors) = load_scheduler_tasks(
                    &pool_clone,
                    &scheduler_clone,
                    &task_queue_clone,
                    &app.handle().clone(),
                )
                .await
                .expect("Failed to load scheduler tasks");

                if !load_errors.is_empty() {
                    eprintln!(
                        "Loaded {} scheduler tasks with {} errors: {}",
                        loaded_count,
                        load_errors.len(),
                        load_errors.join(", ")
                    );
                }

                let scheduler = scheduler_clone.lock().await;
                scheduler.start().await.expect("Failed to start scheduler");
            });

            // Start power monitoring on macOS
            #[cfg(target_os = "macos")]
            {
                use power_monitor::PowerMonitor;

                let scheduler_for_power = scheduler.clone();
                let pool_for_power = pool_arc.clone();
                let task_queue_for_power = task_queue.clone();
                let app_handle_for_power = app.handle().clone();

                tauri::async_runtime::spawn(async move {
                    let mut power_monitor = PowerMonitor::new();

                    while let Some(event) = power_monitor.recv().await {
                        match event {
                            power_monitor::PowerEvent::WillSleep { notification_id } => {
                                handle_system_sleep(
                                    &scheduler_for_power,
                                    &pool_for_power,
                                    notification_id,
                                )
                                .await;
                            }
                            power_monitor::PowerEvent::DidWake => {
                                handle_system_wake(
                                    &scheduler_for_power,
                                    &pool_for_power,
                                    &task_queue_for_power,
                                    &app_handle_for_power,
                                )
                                .await;
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle();
                let _ = app.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|app_handle, event| match event {
        RunEvent::Exit => {
            scheduler::kill_all_processes();
            println!("Application exiting...");
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    });
}

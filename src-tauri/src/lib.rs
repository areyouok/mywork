use std::sync::Arc;
use sqlx::SqlitePool;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent,
};
use tokio::sync::Mutex;
use scheduler::job_scheduler::Scheduler;

pub mod commands;
pub mod db;
pub mod models;
pub mod opencode;
pub mod scheduler;
pub mod storage;

use commands::{get_tasks, get_task, create_task, update_task, delete_task, run_task};
use commands::{get_executions, get_execution};
use commands::{get_scheduler_status, start_scheduler, stop_scheduler, reload_scheduler};
use commands::{get_output, delete_output};
use models::execution::{get_executions_by_status, ExecutionStatus, UpdateExecution};
use chrono::Utc;

fn mark_running_as_failed_blocking(pool: &SqlitePool) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let count = runtime.block_on(async {
        let running_executions = match get_executions_by_status(pool, ExecutionStatus::Running).await {
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
    let app = tauri::Builder::default()
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
            get_scheduler_status,
            start_scheduler,
            stop_scheduler,
            reload_scheduler,
            get_output,
            delete_output
        ])
        .setup(|app| {
            let db_path = db::connection::get_database_path(app.handle())?;
            let pool = tauri::async_runtime::block_on(async {
                db::connection::init_database(&db_path).await
            })
            .expect("Failed to initialize database");

            mark_running_as_failed_blocking(&pool);

            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            scheduler::cleanup_orphan_processes(&app_data_dir);

            let pool_arc = Arc::new(pool);
            app.manage(pool_arc.clone());
            
            let scheduler = Arc::new(Mutex::new(Scheduler::new()));
            app.manage(scheduler.clone());

            setup_tray(app)?;
            
            let scheduler_clone = scheduler.clone();
            let pool_clone = pool_arc.clone();
            
            let _ = tauri::async_runtime::block_on(async {
                let scheduler = scheduler_clone.lock().await;
                
                let tasks = crate::models::task::get_all_tasks(&pool_clone)
                    .await
                    .expect("Failed to get tasks");
                
                for task in tasks.iter() {
                    if task.enabled != 1 {
                        continue;
                    }
                    
                    let cron_expression = match (&task.cron_expression, &task.simple_schedule) {
                        (Some(cron), _) => Some(cron.clone()),
                        (_, Some(json)) => crate::scheduler::parse_simple_schedule(json).ok(),
                        _ => {
                            eprintln!("Task {} has no schedule, skipping", task.id);
                            continue;
                        },
                    };
                    
                    let Some(cron_exp) = cron_expression else {
                        continue;
                    };
                    
                    let pool_inner = pool_clone.clone();
                    let app_handle = app.handle().clone();
                    let task_id = task.id.clone();
                    let timeout = task.timeout_seconds as u64;
                    
                    let callback = Arc::new(move || {
                        let pool = pool_inner.clone();
                        let app = app_handle.clone();
                        let task_id = task_id.clone();
                        let timeout_secs = timeout;
                        
                        Box::pin(async move {
                            let _ = crate::commands::execute_task_internal(task_id, pool, app, timeout_secs).await;
                        }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                    });
                    
                    if let Err(e) = scheduler.add_job(&task.id, &cron_exp, callback).await {
                        eprintln!("Failed to add job for task {}: {}", task.id, e);
                    }
                }
                
                scheduler.start().await.expect("Failed to start scheduler");
            });
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let RunEvent::Exit = event {
            scheduler::kill_all_processes();
            println!("Application exiting...");
        }
    });
}

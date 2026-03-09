use std::sync::Arc;
use sqlx::SqlitePool;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tokio::sync::Mutex;
use scheduler::job_scheduler::Scheduler;

pub mod commands;
pub mod db;
pub mod models;
pub mod opencode;
pub mod scheduler;
pub mod storage;

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = db::connection::get_database_path(app.handle())?;
            let pool = tauri::async_runtime::block_on(async {
                db::connection::init_database(&db_path).await
            })
            .expect("Failed to initialize database");

            app.manage(Arc::new(pool));
            
            let scheduler = Arc::new(Mutex::new(Scheduler::new()));
            app.manage(scheduler);

            setup_tray(app)?;
            
            let scheduler_clone = app.state::<Arc<Mutex<Scheduler>>>().inner().clone();
            let pool_clone = app.state::<Arc<SqlitePool>>().inner().clone();
            
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
                            let _ = crate::commands::execute_task_internal(task_id, pool.clone(), app, timeout_secs).await;
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

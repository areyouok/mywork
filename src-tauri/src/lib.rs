use std::sync::Arc;
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::get_task,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::get_executions,
            commands::get_execution,
            commands::create_execution,
            commands::update_execution,
            commands::start_scheduler,
            commands::stop_scheduler,
            commands::get_scheduler_status,
            commands::run_task,
            commands::get_output,
            commands::delete_output,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

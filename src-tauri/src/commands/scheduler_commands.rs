use crate::scheduler::job_scheduler::{Scheduler, SchedulerState};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Start the scheduler
#[tauri::command]
pub async fn start_scheduler(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    scheduler_guard
        .start()
        .await
        .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    
    Ok("Scheduler started successfully".to_string())
}

/// Stop the scheduler
#[tauri::command]
pub async fn stop_scheduler(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    scheduler_guard
        .stop()
        .await
        .map_err(|e| format!("Failed to stop scheduler: {}", e))?;
    
    Ok("Scheduler stopped successfully".to_string())
}

/// Get scheduler status
#[tauri::command]
pub async fn get_scheduler_status(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    let state = scheduler_guard.get_state().await;
    
    let status = match state {
        SchedulerState::Running => "running",
        SchedulerState::Stopped => "stopped",
    };
    
    Ok(status.to_string())
}

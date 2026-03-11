use crate::models::task::{NewTask, Task, UpdateTask};
use crate::scheduler::job_scheduler::Scheduler;
use crate::scheduler::task_queue::TaskQueue;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// Get all tasks
#[tauri::command]
pub async fn get_tasks(
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<Task>, String> {
    let pool = pool.inner().clone();
    crate::models::task::get_all_tasks(&pool)
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))
}

/// Get a single task by ID
#[tauri::command]
pub async fn get_task(
    id: String,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    crate::models::task::get_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to get task: {}", e))
}

/// Create a new task
#[tauri::command]
pub async fn create_task(
    new_task: NewTask,
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    let task = crate::models::task::create_task(&pool, new_task)
        .await
        .map_err(|e| format!("Failed to create task: {}", e))?;
    
    if task.enabled == 1 {
        let cron_expression = crate::scheduler::get_task_cron_expression(&task);
        
        if let Some(cron_exp) = cron_expression {
            add_task_to_scheduler(&scheduler, &task_queue, &task, &cron_exp, &pool, &app).await?;
        }
    }
    
    Ok(task)
}

/// Update an existing task
#[tauri::command]
pub async fn update_task(
    id: String,
    update: UpdateTask,
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    
    {
            let scheduler_guard = scheduler.inner().lock().await;
            if scheduler_guard.has_job(&id).await {
                let _ = scheduler_guard.remove_job(&id).await;
            }
        }
    
    let task = crate::models::task::update_task(&pool, &id, update)
        .await
        .map_err(|e| format!("Failed to update task: {}", e))?;
    
    if task.enabled == 1 {
        let cron_expression = crate::scheduler::get_task_cron_expression(&task);
        
        if let Some(cron_exp) = cron_expression {
            add_task_to_scheduler(&scheduler, &task_queue, &task, &cron_exp, &pool, &app).await?;
        }
    }
    
    Ok(task)
}

/// Delete a task
#[tauri::command]
pub async fn delete_task(
    id: String,
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<bool, String> {
    let pool = pool.inner().clone();
    
    let scheduler_guard = scheduler.inner().lock().await;
    if scheduler_guard.has_job(&id).await {
        let _ = scheduler_guard.remove_job(&id).await;
    }
    drop(scheduler_guard);
    
    crate::models::task::delete_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to delete task: {}", e))
}

async fn add_task_to_scheduler(
    scheduler: &Arc<Mutex<Scheduler>>,
    task_queue: &Arc<Mutex<TaskQueue>>,
    task: &Task,
    cron_expression: &str,
    pool: &Arc<SqlitePool>,
    app: &AppHandle,
) -> Result<(), String> {
    let scheduler_guard = scheduler.lock().await;
    
    let pool_clone = pool.clone();
    let app_handle = app.clone();
    let task_id = task.id.clone();
    let timeout = task.timeout_seconds as u64;
    let task_queue_clone = task_queue.clone();
    
    let callback = Arc::new(move || {
        let pool = pool_clone.clone();
        let app = app_handle.clone();
        let task_id = task_id.clone();
        let timeout_secs = timeout;
        let task_queue = task_queue_clone.clone();
        
        Box::pin(async move {
            let _ = crate::commands::execute_task_internal(task_id, pool, app, timeout_secs, task_queue).await;
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    });
    
    let _job_id = scheduler_guard
        .add_job(&task.id, cron_expression, callback)
        .await
        .map_err(|e| format!("Failed to add job to scheduler: {}", e))?;
    
    Ok(())
}

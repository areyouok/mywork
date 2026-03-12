use crate::models::task::{NewTask, Task, UpdateTask};
use crate::scheduler::job_scheduler::Scheduler;
use crate::scheduler::TaskSchedule;
use crate::scheduler::task_queue::TaskQueue;
use crate::storage::output;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
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
        add_task_to_scheduler(&scheduler, &task_queue, &task, &pool, &app).await?;
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

    let previous_task = crate::models::task::get_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to load existing task before update: {}", e))?;

    let had_scheduler_job = {
        let scheduler_guard = scheduler.inner().lock().await;
        scheduler_guard.has_job(&id).await
    };

    if had_scheduler_job {
        let scheduler_guard = scheduler.inner().lock().await;
        scheduler_guard
            .remove_job(&id)
            .await
            .map_err(|e| format!("Failed to remove old scheduler job before update: {}", e))?;
    }

    let task = match crate::models::task::update_task(&pool, &id, update).await {
        Ok(task) => task,
        Err(e) => {
            if had_scheduler_job && previous_task.enabled == 1 {
                let restore_result = add_task_to_scheduler(
                    &scheduler,
                    &task_queue,
                    &previous_task,
                    &pool,
                    &app,
                )
                .await;

                if let Err(restore_error) = restore_result {
                    return Err(format!(
                        "Failed to update task: {}. Also failed to restore previous schedule: {}",
                        e, restore_error
                    ));
                }
            }

            return Err(format!("Failed to update task: {}", e));
        }
    };

    if task.enabled == 1 {
        if let Err(add_error) = add_task_to_scheduler(&scheduler, &task_queue, &task, &pool, &app).await {
            if previous_task.enabled == 1 {
                let restore_result = add_task_to_scheduler(
                    &scheduler,
                    &task_queue,
                    &previous_task,
                    &pool,
                    &app,
                )
                .await;

                if let Err(restore_error) = restore_result {
                    return Err(format!(
                        "Failed to schedule updated task: {}. Also failed to restore previous schedule: {}",
                        add_error, restore_error
                    ));
                }
            }

            return Err(format!("Failed to schedule updated task: {}", add_error));
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
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<bool, String> {
    let pool = pool.inner().clone();

    let existing_task = crate::models::task::get_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to load task before deletion: {}", e))?;

    let queue_guard = task_queue.inner().lock().await;
    if queue_guard.is_running(&id).await {
        return Err(format!(
            "Cannot delete task '{}' while it is running",
            id
        ));
    }

    let executions = crate::models::execution::get_executions_by_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to query executions before deleting task: {}", e))?;

    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;

    for execution in executions {
        if let Some(output_file) = execution.output_file {
            output::delete_output_file(&output_dir, &output_file)
                .await
                .map_err(|e| {
                    format!(
                        "Failed to delete output file '{}' for execution '{}': {}",
                        output_file, execution.id, e
                    )
                })?;
        } else {
            output::delete_output_files_for_execution(&output_dir, &execution.id)
                .await
                .map_err(|e| {
                    format!(
                        "Failed to delete output files for execution '{}': {}",
                        execution.id, e
                    )
                })?;
        }
    }

    let had_scheduler_job = {
        let scheduler_guard = scheduler.inner().lock().await;
        scheduler_guard.has_job(&id).await
    };

    if had_scheduler_job {
        let scheduler_guard = scheduler.inner().lock().await;
        scheduler_guard
            .remove_job(&id)
            .await
            .map_err(|e| format!("Failed to remove scheduler job before deleting task: {}", e))?;
    }

    let deleted = match crate::models::task::delete_task(&pool, &id).await {
        Ok(deleted) => deleted,
        Err(e) => {
            if had_scheduler_job && existing_task.enabled == 1 {
                let restore_result = add_task_to_scheduler(
                    &scheduler,
                    &task_queue,
                    &existing_task,
                    &pool,
                    &app,
                )
                .await;

                if let Err(restore_error) = restore_result {
                    return Err(format!(
                        "Failed to delete task with related data: {}. Also failed to restore scheduler entry: {}",
                        e, restore_error
                    ));
                }
            }

            return Err(format!("Failed to delete task with related data: {}", e));
        }
    };

    if !deleted && had_scheduler_job && existing_task.enabled == 1 {
        add_task_to_scheduler(&scheduler, &task_queue, &existing_task, &pool, &app)
            .await
            .map_err(|e| format!("Task was not deleted and scheduler restore failed: {}", e))?;
    }

    drop(queue_guard);

    Ok(deleted)
}

async fn add_task_to_scheduler(
    scheduler: &Arc<Mutex<Scheduler>>,
    task_queue: &Arc<Mutex<TaskQueue>>,
    task: &Task,
    pool: &Arc<SqlitePool>,
    app: &AppHandle,
) -> Result<(), String> {
    let schedule = match crate::scheduler::get_task_schedule(task) {
        Some(value) => value,
        None => return Ok(()),
    };

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

    match schedule {
        TaskSchedule::Cron(cron_expression) => {
            let _job_id = scheduler_guard
                .add_job(&task.id, &cron_expression, callback)
                .await
                .map_err(|e| format!("Failed to add job to scheduler: {}", e))?;
        }
        TaskSchedule::Once(run_at) => {
            let now = Utc::now();
            if run_at <= now {
                return Ok(());
            }

            let duration = (run_at - now)
                .to_std()
                .unwrap_or_else(|_| Duration::from_secs(0));

            let _job_id = scheduler_guard
                .add_one_shot_job(&task.id, duration, callback)
                .await
                .map_err(|e| format!("Failed to add one-time job to scheduler: {}", e))?;
        }
    }

    Ok(())
}

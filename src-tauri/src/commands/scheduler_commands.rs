use crate::db::connection;
use crate::execution_retention::enforce_execution_history_limit;
use crate::models::task::touch_task;
use crate::scheduler::job_scheduler::{JobCallback, Scheduler, SchedulerState};
use crate::scheduler::task_queue::{TaskQueue, TaskQueueError};
use crate::scheduler::TaskSchedule;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
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
pub async fn stop_scheduler(scheduler: State<'_, Arc<Mutex<Scheduler>>>) -> Result<String, String> {
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
    let job_count = scheduler_guard.job_count().await;

    let status = match state {
        SchedulerState::Running => "running",
        SchedulerState::Stopped => "stopped",
    };

    Ok(format!("{} ({} jobs)", status, job_count))
}

/// Reload scheduler with all enabled tasks from database
#[tauri::command]
pub async fn reload_scheduler(
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<String, String> {
    let pool = pool.inner().clone();
    let scheduler = scheduler.inner().clone();
    let task_queue = task_queue.inner().clone();

    // Get all tasks from database
    let tasks = crate::models::task::get_all_tasks(&pool)
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))?;

    // Stop scheduler if running
    {
        let scheduler_guard = scheduler.lock().await;
        if scheduler_guard.get_state().await == SchedulerState::Running {
            let _ = scheduler_guard.stop().await;
        }
    }

    // Track loaded count
    let mut loaded_count = 0;
    let mut errors = Vec::new();

    // Add each enabled task with schedule
    for task in tasks.iter() {
        // Skip disabled tasks
        if task.enabled == 0 {
            continue;
        }

        let schedule = match crate::scheduler::get_task_schedule(task) {
            Some(schedule) => schedule,
            None => {
                errors.push(format!("Task '{}' has no valid schedule", task.id));
                continue;
            }
        };

        // Create callback for this task
        let task_id = task.id.clone();
        let pool_clone = pool.clone();
        let app_handle = app.clone();
        let task_timeout = task.timeout_seconds as u64;
        let task_queue_clone = task_queue.clone();

        let callback: JobCallback = Arc::new(move || {
            let task_id = task_id.clone();
            let pool = pool_clone.clone();
            let app = app_handle.clone();
            let timeout = task_timeout;
            let task_queue = task_queue_clone.clone();

            Box::pin(async move {
                // Execute the task
                let result = execute_task_internal(task_id, pool, app, timeout, task_queue).await;
                if let Err(e) = result {
                    eprintln!("Task execution failed: {}", e);
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        // Add job to scheduler
        let scheduler_guard = scheduler.lock().await;
        let add_result = match schedule {
            TaskSchedule::Cron(cron_expr) => {
                scheduler_guard
                    .add_job(&task.id, &cron_expr, callback)
                    .await
            }
            TaskSchedule::Once(run_at) => {
                let now = Utc::now();
                if run_at <= now {
                    continue;
                }
                let duration = (run_at - now)
                    .to_std()
                    .unwrap_or_else(|_| Duration::from_secs(0));
                scheduler_guard
                    .add_one_shot_job(&task.id, duration, callback)
                    .await
            }
        };

        match add_result {
            Ok(_) => {
                loaded_count += 1;
            }
            Err(e) => {
                errors.push(format!("Failed to add job for task '{}': {}", task.id, e));
            }
        }
    }

    // Start scheduler
    {
        let scheduler_guard = scheduler.lock().await;
        scheduler_guard
            .start()
            .await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    }

    // Return result
    if errors.is_empty() {
        Ok(format!("Successfully loaded {} tasks", loaded_count))
    } else {
        Ok(format!(
            "Loaded {} tasks with {} errors: {}",
            loaded_count,
            errors.len(),
            errors.join(", ")
        ))
    }
}

struct ExecutionFinishedEventGuard {
    app: AppHandle,
    task_id: String,
}

impl Drop for ExecutionFinishedEventGuard {
    fn drop(&mut self) {
        let _ = self.app.emit("execution-finished", &self.task_id);
    }
}

/// Internal function to execute a task (used by scheduler callbacks)
pub async fn execute_task_internal(
    task_id: String,
    pool: Arc<SqlitePool>,
    app: AppHandle,
    timeout_seconds: u64,
    task_queue: Arc<Mutex<TaskQueue>>,
) -> Result<(), String> {
    use crate::models::execution::{generate_output_file_name, ExecutionStatus};
    use crate::opencode::executor::run_opencode_task;
    use crate::storage::output;
    use chrono::Utc;

    // Atomically check if running and acquire slot (prevents race condition)
    let queue = task_queue.lock().await;
    let _guard = match queue.acquire_slot(&task_id).await {
        Ok(guard) => guard,
        Err(TaskQueueError::TaskAlreadyRunning { task_id }) => {
            eprintln!("Task '{}' is already running, skipping execution", task_id);
            return Ok(());
        }
        Err(e) => {
            eprintln!("Failed to acquire slot for task '{}': {}", task_id, e);
            return Ok(());
        }
    };
    drop(queue);

    // Get task
    let task = crate::models::task::get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;

    // Get database directory for working directory
    let db_path = connection::get_database_directory(&app)
        .map_err(|e| format!("Failed to get database directory: {}", e))?;
    let cwd = db_path.parent();

    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;

    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    if let Err(e) = touch_task(&pool, &task_id).await {
        eprintln!(
            "Failed to update task timestamp for task '{}': {}",
            task_id, e
        );
    }

    // Create execution record
    let new_execution = crate::models::execution::NewExecution {
        task_id: task_id.clone(),
        session_id: None,
        status: Some(crate::models::execution::ExecutionStatus::Running),
        output_file: None,
        error_message: None,
    };

    let execution = crate::models::execution::create_execution(&pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;

    let _ = app.emit("execution-started", &task_id);
    let _finished_guard = ExecutionFinishedEventGuard {
        app: app.clone(),
        task_id: task_id.clone(),
    };

    // Run opencode task
    let result = run_opencode_task(&task.prompt, None, Some(timeout_seconds), None, cwd).await;

    let output_file_name = generate_output_file_name(&execution.id, &Utc::now());

    let (session_id, status, finished_at, output_file, error_message) = match result {
        Ok(opencode_output) => {
            let (final_status, err_msg) = if opencode_output.timed_out {
                (
                    ExecutionStatus::Timeout,
                    Some("Execution timed out".to_string()),
                )
            } else if !opencode_output.success {
                (
                    ExecutionStatus::Failed,
                    Some(opencode_output.stdout.clone()),
                )
            } else {
                (ExecutionStatus::Success, None)
            };

            let content = format!(
                "Session ID: {}\n{}",
                opencode_output.session_id, opencode_output.stdout
            );

            let _file_path = output::write_output_file(&output_dir, &output_file_name, &content)
                .await
                .map_err(|e| format!("Failed to write output file: {}", e))?;

            (
                Some(opencode_output.session_id),
                final_status,
                Utc::now().to_rfc3339(),
                Some(output_file_name.clone()),
                err_msg,
            )
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            let content = format!("Error: {}", error_msg);

            // Try to write error output file, but don't set output_file if it fails
            let output_file_result =
                match output::write_output_file(&output_dir, &output_file_name, &content).await {
                    Ok(_) => Some(output_file_name.clone()),
                    Err(write_err) => {
                        eprintln!("Failed to write error output file: {}", write_err);
                        None
                    }
                };

            (
                None,
                ExecutionStatus::Failed,
                Utc::now().to_rfc3339(),
                output_file_result,
                Some(error_msg),
            )
        }
    };

    // Update execution record
    let update = crate::models::execution::UpdateExecution {
        session_id,
        status: Some(status.clone()),
        finished_at: Some(finished_at),
        output_file: output_file.clone(),
        error_message: error_message.clone(),
    };

    crate::models::execution::update_execution(&pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;

    enforce_execution_history_limit(&pool, &output_dir).await;

    Ok(())
}

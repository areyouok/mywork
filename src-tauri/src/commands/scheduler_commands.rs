use crate::db::connection;
use crate::execution_retention::enforce_execution_history_limit;
use crate::executor::streaming_executor::{StreamLine, StreamingExecutor};
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

#[cfg(target_os = "macos")]
fn is_system_sleeping() -> bool {
    crate::power_monitor::is_sleeping() || crate::power_monitor::is_clamshell_closed()
}

#[cfg(not(target_os = "macos"))]
fn is_system_sleeping() -> bool {
    false
}

/// Start the scheduler
#[tauri::command]
pub async fn start_scheduler(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    if is_system_sleeping() {
        return Err(
            "System is unavailable (sleeping or lid closed); scheduler start is paused".to_string(),
        );
    }

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

/// Load enabled tasks from database into scheduler
///
/// This function is used internally by reload_scheduler and for wake-up handling.
/// It assumes the scheduler is already stopped.
pub async fn load_scheduler_tasks(
    pool: &Arc<SqlitePool>,
    scheduler: &Arc<Mutex<Scheduler>>,
    task_queue: &Arc<Mutex<TaskQueue>>,
    app: &AppHandle,
) -> Result<(usize, Vec<String>), String> {
    let tasks = crate::models::task::get_all_tasks(pool)
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))?;

    let mut loaded_count = 0;
    let mut errors = Vec::new();

    for task in tasks.iter() {
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
                let result = execute_task_internal(task_id, pool, app, timeout, task_queue).await;
                if let Err(e) = result {
                    eprintln!("Task execution failed: {}", e);
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

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

    Ok((loaded_count, errors))
}

/// Reload scheduler with all enabled tasks from database
#[tauri::command]
pub async fn reload_scheduler(
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<String, String> {
    if is_system_sleeping() {
        return Err(
            "System is unavailable (sleeping or lid closed); scheduler reload is paused"
                .to_string(),
        );
    }

    let pool = pool.inner().clone();
    let scheduler = scheduler.inner().clone();
    let task_queue = task_queue.inner().clone();

    // Stop scheduler if running
    {
        let scheduler_guard = scheduler.lock().await;
        if scheduler_guard.get_state().await == SchedulerState::Running {
            scheduler_guard
                .stop()
                .await
                .map_err(|e| format!("Failed to stop scheduler: {}", e))?;
        }

        scheduler_guard.clear_jobs().await;
    }

    // Load tasks
    let (loaded_count, errors) = load_scheduler_tasks(&pool, &scheduler, &task_queue, &app).await?;

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
    use crate::storage::output;
    use chrono::Utc;

    if is_system_sleeping() {
        eprintln!(
            "Skipping scheduled execution for task '{}' because the system is unavailable (sleeping or lid closed)",
            task_id
        );
        return Ok(());
    }

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
    let cwd = Some(&db_path);

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

    if is_system_sleeping() {
        let update = crate::models::execution::UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Failed),
            finished_at: Some(Utc::now().to_rfc3339()),
            output_file: execution.output_file.clone(),
            error_message: Some("System entering sleep".to_string()),
        };

        let _ = crate::models::execution::update_execution_if_running(&pool, &execution.id, update)
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;

        return Ok(());
    }

    let output_file_name = generate_output_file_name(&execution.id, &Utc::now());

    output::write_output_file(&output_dir, &output_file_name, "")
        .await
        .map_err(|e| format!("Failed to initialize output file: {}", e))?;

    let _ = crate::models::execution::update_execution_if_running(
        &pool,
        &execution.id,
        crate::models::execution::UpdateExecution {
            session_id: None,
            status: None,
            finished_at: None,
            output_file: Some(output_file_name.clone()),
            error_message: None,
        },
    )
    .await
    .map_err(|e| format!("Failed to set output file on execution: {}", e))?;

    let args: Vec<&str> = vec!["run", &task.prompt];

    if is_system_sleeping() {
        let update = crate::models::execution::UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Failed),
            finished_at: Some(Utc::now().to_rfc3339()),
            output_file: Some(output_file_name.clone()),
            error_message: Some("System entering sleep".to_string()),
        };

        let _ = crate::models::execution::update_execution_if_running(&pool, &execution.id, update)
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;

        return Ok(());
    }

    let opencode_binary = crate::opencode::executor::resolve_opencode_binary_path()
        .map_err(|e| format!("Failed to locate opencode binary: {}", e))?;

    let mut executor = StreamingExecutor::spawn(&opencode_binary, &args, cwd)
        .await
        .map_err(|e| format!("Failed to start opencode streaming: {}", e))?;

    let mut parsed_session_id: Option<String> = None;
    let stream_future = async {
        while let Some(line) = executor.read_line().await {
            match line {
                StreamLine::Stdout(text) => {
                    if parsed_session_id.is_none() {
                        parsed_session_id =
                            crate::opencode::session_parser::parse_session_id(&text);
                    }

                    output::append_output_file(
                        &output_dir,
                        &output_file_name,
                        &format!("{}\n", text),
                    )
                    .await
                    .map_err(|e| format!("Failed to append stdout: {}", e))?;
                }
                StreamLine::Stderr(text) => {
                    output::append_output_file(
                        &output_dir,
                        &output_file_name,
                        &format!("{}\n", text),
                    )
                    .await
                    .map_err(|e| format!("Failed to append stderr: {}", e))?;
                }
                StreamLine::Finished => break,
            }
        }

        Ok::<i32, String>(executor.exit_code().await.unwrap_or(-1))
    };

    let timeout_result =
        tokio::time::timeout(Duration::from_secs(timeout_seconds), stream_future).await;

    let (session_id, status, finished_at, output_file, error_message) = match timeout_result {
        Ok(Ok(exit_code)) => {
            let final_status = if exit_code == 0 {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failed
            };

            let err_msg = if exit_code == 0 {
                None
            } else {
                Some(format!("Process exited with code {}", exit_code))
            };

            (
                parsed_session_id,
                final_status,
                Utc::now().to_rfc3339(),
                Some(output_file_name.clone()),
                err_msg,
            )
        }
        Ok(Err(e)) => {
            let _ = output::append_output_file(
                &output_dir,
                &output_file_name,
                &format!("Error: {}\n", e),
            )
            .await;

            (
                parsed_session_id,
                ExecutionStatus::Failed,
                Utc::now().to_rfc3339(),
                Some(output_file_name.clone()),
                Some(e),
            )
        }
        Err(_) => {
            executor.kill().await;
            let msg = "Execution timed out".to_string();
            let _ =
                output::append_output_file(&output_dir, &output_file_name, &format!("{}\n", msg))
                    .await;

            (
                parsed_session_id,
                ExecutionStatus::Timeout,
                Utc::now().to_rfc3339(),
                Some(output_file_name.clone()),
                Some(msg),
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

    let _ = crate::models::execution::update_execution_if_running(&pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;

    enforce_execution_history_limit(&pool, &output_dir).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_system_sleeping_tracks_power_monitor_state() {
        crate::power_monitor::with_test_power_state_lock(|| {
            crate::power_monitor::set_sleeping(true);
            assert!(is_system_sleeping());

            crate::power_monitor::set_sleeping(false);
            assert!(!is_system_sleeping());
        });
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_is_system_sleeping_is_false_off_macos() {
        assert!(!is_system_sleeping());
    }
}

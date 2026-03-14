use crate::db::connection;
use crate::execution_retention::enforce_execution_history_limit;
use crate::executor::streaming_executor::{StreamLine, StreamingExecutor};
use crate::models::execution::{
    create_execution, generate_output_file_name, update_execution, ExecutionStatus, NewExecution,
    UpdateExecution,
};
use crate::models::task::{get_task, touch_task};
use crate::opencode::executor::resolve_opencode_binary_path;
use crate::opencode::session_parser::parse_session_id;
use crate::scheduler::task_queue::{TaskQueue, TaskQueueError};
use crate::storage::output;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

#[cfg(target_os = "macos")]
fn is_system_sleeping() -> bool {
    crate::power_monitor::is_sleeping()
}

#[cfg(not(target_os = "macos"))]
fn is_system_sleeping() -> bool {
    false
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

async fn mark_execution_failed(
    pool: &SqlitePool,
    execution_id: &str,
    output_dir: Option<&std::path::Path>,
    output_file: Option<&str>,
    error_message: &str,
) {
    if let (Some(dir), Some(file_name)) = (output_dir, output_file) {
        let _ = output::append_output_file(dir, file_name, &format!("Error: {}\n", error_message))
            .await;
    }

    let _ = update_execution(
        pool,
        execution_id,
        UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Failed),
            finished_at: Some(Utc::now().to_rfc3339()),
            output_file: output_file.map(std::string::ToString::to_string),
            error_message: Some(error_message.to_string()),
        },
    )
    .await;
}

#[tauri::command]
pub async fn run_task(
    task_id: String,
    pool: State<'_, Arc<SqlitePool>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<String, String> {
    if is_system_sleeping() {
        return Err("System is sleeping; task execution is paused".to_string());
    }

    let queue = task_queue.inner().lock().await;
    let _guard = match queue.acquire_slot(&task_id).await {
        Ok(guard) => guard,
        Err(TaskQueueError::TaskAlreadyRunning { task_id }) => {
            return Err(format!("Task '{}' is already running", task_id));
        }
        Err(e) => {
            return Err(format!("Failed to acquire slot: {}", e));
        }
    };
    drop(queue);

    let pool = pool.inner().clone();

    let task = get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;

    if let Err(e) = touch_task(&pool, &task_id).await {
        eprintln!(
            "Failed to update task timestamp for task '{}': {}",
            task_id, e
        );
    }

    let new_execution = NewExecution {
        task_id: task_id.clone(),
        session_id: None,
        status: Some(ExecutionStatus::Running),
        output_file: None,
        error_message: None,
    };

    let execution = create_execution(&pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;

    let _ = app.emit("execution-started", &task_id);
    let _finished_guard = ExecutionFinishedEventGuard {
        app: app.clone(),
        task_id: task_id.clone(),
    };

    if is_system_sleeping() {
        let update = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Failed),
            finished_at: Some(Utc::now().to_rfc3339()),
            output_file: execution.output_file.clone(),
            error_message: Some("System entering sleep".to_string()),
        };

        let _ = crate::models::execution::update_execution_if_running(&pool, &execution.id, update)
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;

        return Err("System is sleeping; task execution is paused".to_string());
    }

    let timeout_secs = task.timeout_seconds as u64;

    let execution_id = execution.id.clone();

    // Get database directory for working directory
    let db_path_result = connection::get_database_directory(&app)
        .map_err(|e| format!("Failed to get database directory: {}", e));
    let db_path = match db_path_result {
        Ok(path) => path,
        Err(message) => {
            mark_execution_failed(&pool, &execution_id, None, None, &message).await;
            return Err(message);
        }
    };
    let cwd = db_path.parent();

    let output_dir_result = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e));
    let output_dir = match output_dir_result {
        Ok(dir) => dir,
        Err(message) => {
            mark_execution_failed(&pool, &execution_id, None, None, &message).await;
            return Err(message);
        }
    };

    if let Err(e) = output::create_output_directory(&output_dir).await {
        let message = format!("Failed to create output directory: {}", e);
        mark_execution_failed(&pool, &execution_id, None, None, &message).await;
        return Err(message);
    }

    let output_file_name = generate_output_file_name(&execution.id, &Utc::now());

    if let Err(e) = output::write_output_file(&output_dir, &output_file_name, "").await {
        let message = format!("Failed to initialize output file: {}", e);
        mark_execution_failed(&pool, &execution_id, Some(&output_dir), None, &message).await;
        return Err(message);
    }

    if let Err(e) = update_execution(
        &pool,
        &execution.id,
        UpdateExecution {
            session_id: None,
            status: None,
            finished_at: None,
            output_file: Some(output_file_name.clone()),
            error_message: None,
        },
    )
    .await
    {
        let message = format!("Failed to set output file on execution: {}", e);
        mark_execution_failed(
            &pool,
            &execution_id,
            Some(&output_dir),
            Some(&output_file_name),
            &message,
        )
        .await;
        return Err(message);
    }

    let args: Vec<&str> = vec!["run", &task.prompt];
    let opencode_binary = match resolve_opencode_binary_path() {
        Ok(path) => path,
        Err(e) => {
            let message = format!("Failed to locate opencode binary: {}", e);
            mark_execution_failed(
                &pool,
                &execution_id,
                Some(&output_dir),
                Some(&output_file_name),
                &message,
            )
            .await;
            return Err(message);
        }
    };

    if is_system_sleeping() {
        let message = "System entering sleep".to_string();
        mark_execution_failed(
            &pool,
            &execution_id,
            Some(&output_dir),
            Some(&output_file_name),
            &message,
        )
        .await;

        return Err("System is sleeping; task execution is paused".to_string());
    }

    let mut executor = match StreamingExecutor::spawn(&opencode_binary, &args, cwd).await {
        Ok(executor) => executor,
        Err(e) => {
            let message = format!("Failed to start opencode streaming: {}", e);
            mark_execution_failed(
                &pool,
                &execution_id,
                Some(&output_dir),
                Some(&output_file_name),
                &message,
            )
            .await;
            return Err(message);
        }
    };

    let mut parsed_session_id: Option<String> = None;
    let stream_future = async {
        while let Some(line) = executor.read_line().await {
            match line {
                StreamLine::Stdout(text) => {
                    if parsed_session_id.is_none() {
                        parsed_session_id = parse_session_id(&text);
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

    let timeout_result = timeout(Duration::from_secs(timeout_secs), stream_future).await;

    let (status, finished_at, output_file, error_message) = match timeout_result {
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
                ExecutionStatus::Timeout,
                Utc::now().to_rfc3339(),
                Some(output_file_name.clone()),
                Some(msg),
            )
        }
    };

    let update = UpdateExecution {
        session_id: parsed_session_id,
        status: Some(status.clone()),
        finished_at: Some(finished_at),
        output_file: output_file.clone(),
        error_message: error_message.clone(),
    };

    let _ = crate::models::execution::update_execution_if_running(&pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;

    enforce_execution_history_limit(&pool, &output_dir).await;

    if status == ExecutionStatus::Success {
        Ok(execution.id)
    } else {
        Err(format!(
            "Task execution failed with status: {:?}{}",
            status,
            error_message
                .map(|e| format!(" - {}", e))
                .unwrap_or_default()
        ))
    }
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

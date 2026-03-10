use crate::models::execution::{create_execution, update_execution, ExecutionStatus, NewExecution, UpdateExecution};
use crate::models::task::get_task;
use crate::executor::streaming_executor::{StreamLine, StreamingExecutor};
use crate::scheduler::task_queue::{TaskQueue, SkipResult};
use crate::storage::output;
use crate::db::connection;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

#[tauri::command]
pub async fn run_task(
    task_id: String,
    pool: State<'_, Arc<SqlitePool>>,
    task_queue: State<'_, Arc<Mutex<TaskQueue>>>,
    app: AppHandle,
) -> Result<String, String> {
    let queue = task_queue.inner().lock().await;
    let _guard = match queue.acquire_slot_with_skip(&task_id).await {
        Ok(Ok(guard)) => guard,
        Ok(Err(SkipResult::Skipped { task_id })) => {
            return Err(format!("Task '{}' is already running", task_id));
        }
        Ok(Err(SkipResult::Execute)) => unreachable!(),
        Err(e) => {
            return Err(format!("Failed to acquire slot: {}", e));
        }
    };
    drop(queue);
    
    let pool = pool.inner().clone();
    
    let task = get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;
    
    let new_execution = NewExecution {
        task_id: task_id.clone(),
        session_id: None,
        status: Some(ExecutionStatus::Running),
        output_file: Some(task_id.clone()),
        error_message: None,
    };
    
    let execution = create_execution(&pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;
    
    let timeout_secs = task.timeout_seconds as u64;

    // Get database directory for working directory
    let db_path = connection::get_database_directory(&app)
        .map_err(|e| format!("Failed to get database directory: {}", e))?;
    let cwd = db_path.parent();

    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    output::write_output_file(&output_dir, &execution.id, "")
        .await
        .map_err(|e| format!("Failed to initialize output file: {}", e))?;

    let args: Vec<&str> = vec!["run", &task.prompt];
    let mut executor = StreamingExecutor::spawn("opencode", &args, cwd)
        .await
        .map_err(|e| format!("Failed to start opencode streaming: {}", e))?;

    let mut parsed_session_id: Option<String> = None;
    let stream_future = async {
        while let Some(line) = executor.read_line().await {
            match line {
                StreamLine::Stdout(text) => {
                    if parsed_session_id.is_none() {
                        let trimmed = text.trim();
                        if let Some(rest) = trimmed.strip_prefix("Session ID:") {
                            parsed_session_id = Some(rest.trim().to_string());
                        }
                    }
                    output::append_output_file(&output_dir, &execution.id, &format!("{}\n", text))
                        .await
                        .map_err(|e| format!("Failed to append stdout: {}", e))?;
                }
                StreamLine::Stderr(text) => {
                    output::append_output_file(
                        &output_dir,
                        &execution.id,
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
            (final_status, Utc::now().to_rfc3339(), Some(execution.id.clone()), err_msg)
        }
        Ok(Err(e)) => {
            let _ = output::append_output_file(&output_dir, &execution.id, &format!("Error: {}\n", e)).await;
            (ExecutionStatus::Failed, Utc::now().to_rfc3339(), Some(execution.id.clone()), Some(e))
        }
        Err(_) => {
            executor.kill().await;
            let msg = "Execution timed out".to_string();
            let _ = output::append_output_file(&output_dir, &execution.id, &format!("{}\n", msg)).await;
            (ExecutionStatus::Timeout, Utc::now().to_rfc3339(), Some(execution.id.clone()), Some(msg))
        }
    };
    
    let update = UpdateExecution {
        session_id: parsed_session_id,
        status: Some(status.clone()),
        finished_at: Some(finished_at),
        output_file: output_file.clone(),
        error_message: error_message.clone(),
    };
    
    update_execution(&pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;
    
    if status == ExecutionStatus::Success {
        Ok(execution.id)
    } else {
        Err(format!(
            "Task execution failed with status: {:?}{}",
            status,
            error_message.map(|e| format!(" - {}", e)).unwrap_or_default()
        ))
    }
}

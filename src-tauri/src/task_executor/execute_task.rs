use crate::models::execution::{create_execution, update_execution, ExecutionStatus, NewExecution, UpdateExecution};
use crate::models::task::get_task;
use crate::opencode::executor::run_opencode_task;
use crate::storage::output;
use crate::db::connection;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::TaskExecutionResult;

pub async fn execute_task(
    task_id: &str,
    pool: &SqlitePool,
    app: &AppHandle,
    timeout_seconds: u64,
) -> Result<TaskExecutionResult, String> {
    let task = get_task(pool, task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;
    
    let new_execution = NewExecution {
        task_id: task_id.to_string(),
        session_id: None,
        status: Some(ExecutionStatus::Running),
        output_file: None,
        error_message: None,
    };
    
    let execution = create_execution(pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;
    
    let db_path = connection::get_database_directory(app)
        .map_err(|e| format!("Failed to get database directory: {}", e))?;
    let cwd = db_path.parent();
    
    let result = run_opencode_task(&task.prompt, None, Some(timeout_seconds), None, cwd).await;
    
    let output_dir = output::get_output_directory(app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    let (session_id, status, finished_at, output_file, error_message) = match result {
        Ok(opencode_output) => {
            let (final_status, err_msg) = if opencode_output.timed_out {
                (ExecutionStatus::Timeout, Some("Execution timed out".to_string()))
            } else if !opencode_output.success {
                (ExecutionStatus::Failed, Some(opencode_output.stderr.clone()))
            } else {
                (ExecutionStatus::Success, None)
            };
            
            let content = format!(
                "Session ID: {}\n{}{}",
                opencode_output.session_id,
                opencode_output.stdout,
                if opencode_output.stderr.is_empty() {
                    String::new()
                } else {
                    format!("\n{}", opencode_output.stderr)
                }
            );
            
            let _file_path = output::write_output_file(&output_dir, &execution.id, &content)
                .await
                .map_err(|e| format!("Failed to write output file: {}", e))?;
            
            (
                Some(opencode_output.session_id),
                final_status,
                Utc::now().to_rfc3339(),
                Some(format!("{}.txt", execution.id)),
                err_msg,
            )
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            let content = format!("Error: {}", error_msg);
            
            let _file_path = output::write_output_file(&output_dir, &execution.id, &content)
                .await
                .ok();
            
            (
                None,
                ExecutionStatus::Failed,
                Utc::now().to_rfc3339(),
                Some(format!("{}.txt", execution.id)),
                Some(error_msg),
            )
        }
    };
    
    let update = UpdateExecution {
        session_id,
        status: Some(status.clone()),
        finished_at: Some(finished_at),
        output_file: output_file.clone(),
        error_message: error_message.clone(),
    };
    
    update_execution(pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;
    
    Ok(TaskExecutionResult {
        execution_id: execution.id,
        status: status.as_str().to_string(),
        error_message,
        output_file,
    })
}

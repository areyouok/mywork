use crate::models::execution::{create_execution, update_execution, ExecutionStatus, NewExecution, UpdateExecution};
use crate::models::task::get_task;
use crate::opencode::executor::run_opencode_task;
use crate::storage::output;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run_task(
    task_id: String,
    pool: State<'_, Arc<SqlitePool>>,
    app: AppHandle,
) -> Result<String, String> {
    let pool = pool.inner().clone();
    
    let task = get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;
    
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
    
    let timeout_secs = task.timeout_seconds as u64;
    
    let result = run_opencode_task(&task.prompt, None, Some(timeout_secs), None).await;
    
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    let (status, finished_at, output_file, error_message) = match result {
        Ok(opencode_output) => {
            let (final_status, err_msg) = if opencode_output.timed_out {
                (ExecutionStatus::Timeout, Some("Execution timed out".to_string()))
            } else if !opencode_output.success {
                (ExecutionStatus::Failed, Some(opencode_output.stderr.clone()))
            } else {
                (ExecutionStatus::Success, None)
            };
            
            let content = format!(
                "Session ID: {}\n\n=== STDOUT ===\n{}\n\n=== STDERR ===\n{}",
                opencode_output.session_id,
                opencode_output.stdout,
                opencode_output.stderr
            );
            
            let file_path = output::write_output_file(&output_dir, &execution.id, &content)
                .await
                .map_err(|e| format!("Failed to write output file: {}", e))?;
            
            let file_path_str = file_path.to_string_lossy().to_string();
            
            (final_status, Utc::now().to_rfc3339(), Some(file_path_str), err_msg)
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            let content = format!("Error: {}", error_msg);

            let file_path = output::write_output_file(&output_dir, &execution.id, &content)
                .await
                .ok();

            let file_path_str = file_path.map(|p| p.to_string_lossy().to_string());

            (ExecutionStatus::Failed, Utc::now().to_rfc3339(), file_path_str, Some(error_msg))
        }
    };
    
    let update = UpdateExecution {
        session_id: None,
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

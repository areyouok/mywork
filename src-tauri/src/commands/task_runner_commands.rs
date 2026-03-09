use crate::task_executor::execute_task;
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
    
    let timeout_secs = task.timeout_seconds as u64;
    
    // Use shared executor
    let result = execute_task(task_id.clone(), &pool, &app, timeout_secs)
        .await
        .map_err(|e| format!("Task execution failed: {}", e))?;
    
    if result.status == "success" {
        Ok(result.execution_id)
    } else {
        Err(format!(
            "Task execution failed with status: {}{}",
            result.status,
            result.error_message.map(|e| format!(" - {}", e)).unwrap_or_default()
        ))
    }
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

use crate::storage::output;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::State;

#[tauri::command]
pub async fn get_output(
    execution_id: String,
    pool: State<'_, Arc<SqlitePool>>,
    app: AppHandle,
) -> Result<String, String> {
    let pool = pool.inner().clone();
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;

    let execution = crate::models::execution::get_execution(&pool, &execution_id)
        .await
        .map_err(|e| format!("Failed to get execution: {}", e))?;

    let output_file = if let Some(file_name) = execution.output_file.clone() {
        file_name
    } else {
        match output::find_output_file_for_execution(&output_dir, &execution_id)
            .await
            .map_err(|e| format!("Failed to find output file: {}", e))?
        {
            Some(file_name) => file_name,
            None => return Ok(String::new()),
        }
    };

    let content = output::read_output_file(&output_dir, &output_file)
        .await
        .map_err(|e| format!("Failed to read output file: {}", e))?;
    
    Ok(content)
}

#[tauri::command]
pub async fn delete_output(
    execution_id: String,
    pool: State<'_, Arc<SqlitePool>>,
    app: AppHandle,
) -> Result<bool, String> {
    let pool = pool.inner().clone();
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;

    let execution = crate::models::execution::get_execution(&pool, &execution_id)
        .await
        .map_err(|e| format!("Failed to get execution: {}", e))?;

    let output_file = if let Some(file_name) = execution.output_file {
        file_name
    } else {
        match output::find_output_file_for_execution(&output_dir, &execution_id)
            .await
            .map_err(|e| format!("Failed to find output file: {}", e))?
        {
            Some(file_name) => file_name,
            None => return Ok(true),
        }
    };

    output::delete_output_file(&output_dir, &output_file)
        .await
        .map_err(|e| format!("Failed to delete output file: {}", e))?;
    
    Ok(true)
}

use crate::storage::output;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_output(
    execution_id: String,
    app: AppHandle,
) -> Result<String, String> {
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    let content = output::read_output_file(&output_dir, &execution_id)
        .await
        .map_err(|e| format!("Failed to read output file: {}", e))?;
    
    Ok(content)
}

#[tauri::command]
pub async fn delete_output(
    execution_id: String,
    app: AppHandle,
) -> Result<bool, String> {
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    output::delete_output_file(&output_dir, &execution_id)
        .await
        .map_err(|e| format!("Failed to delete output file: {}", e))?;
    
    Ok(true)
}

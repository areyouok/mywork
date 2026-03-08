use crate::models::execution::{Execution, NewExecution, UpdateExecution};
use std::sync::Arc;
use sqlx::SqlitePool;
use tauri::State;

/// Get all executions for a task
#[tauri::command]
pub async fn get_executions(
    task_id: String,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<Execution>, String> {
    let pool = pool.inner().clone();
    crate::models::execution::get_executions_by_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Failed to get executions: {}", e))
}

/// Get a single execution by ID
#[tauri::command]
pub async fn get_execution(
    id: String,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Execution, String> {
    let pool = pool.inner().clone();
    crate::models::execution::get_execution(&pool, &id)
        .await
        .map_err(|e| format!("Failed to get execution: {}", e))
}

/// Create a new execution (for testing/debugging)
#[tauri::command]
pub async fn create_execution(
    new_execution: NewExecution,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Execution, String> {
    let pool = pool.inner().clone();
    crate::models::execution::create_execution(&pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))
}

/// Update an execution (for testing/debugging)
#[tauri::command]
pub async fn update_execution(
    id: String,
    update: UpdateExecution,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Execution, String> {
    let pool = pool.inner().clone();
    crate::models::execution::update_execution(&pool, &id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))
}

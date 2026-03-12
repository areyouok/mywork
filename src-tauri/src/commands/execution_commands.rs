use crate::models::execution::{Execution, ExecutionStatus, NewExecution, UpdateExecution};
use sqlx::SqlitePool;
use std::sync::Arc;
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

/// Get all currently running executions
#[tauri::command]
pub async fn get_running_executions(
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let pool = pool.inner().clone();
    let running =
        crate::models::execution::get_executions_by_status(&pool, ExecutionStatus::Running)
            .await
            .map_err(|e| format!("Failed to get running executions: {}", e))?;

    Ok(running
        .into_iter()
        .map(|execution| execution.task_id)
        .collect())
}

/// Create a new execution (for testing/debugging)
#[tauri::command]
pub async fn create_execution(
    execution: NewExecution,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Execution, String> {
    let pool = pool.inner().clone();
    crate::models::execution::create_execution(&pool, execution)
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

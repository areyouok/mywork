use crate::models::task::{NewTask, Task, UpdateTask};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::State;

/// Get all tasks
#[tauri::command]
pub async fn get_tasks(
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<Task>, String> {
    let pool = pool.inner().clone();
    crate::models::task::get_all_tasks(&pool)
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))
}

/// Get a single task by ID
#[tauri::command]
pub async fn get_task(
    id: String,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    crate::models::task::get_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to get task: {}", e))
}

/// Create a new task
#[tauri::command]
pub async fn create_task(
    new_task: NewTask,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    crate::models::task::create_task(&pool, new_task)
        .await
        .map_err(|e| format!("Failed to create task: {}", e))
}

/// Update an existing task
#[tauri::command]
pub async fn update_task(
    id: String,
    update: UpdateTask,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Task, String> {
    let pool = pool.inner().clone();
    crate::models::task::update_task(&pool, &id, update)
        .await
        .map_err(|e| format!("Failed to update task: {}", e))
}

/// Delete a task
#[tauri::command]
pub async fn delete_task(
    id: String,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<bool, String> {
    let pool = pool.inner().clone();
    crate::models::task::delete_task(&pool, &id)
        .await
        .map_err(|e| format!("Failed to delete task: {}", e))
}

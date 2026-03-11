use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Task model representing a scheduled task
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub cron_expression: Option<String>,
    pub simple_schedule: Option<String>,
    pub enabled: i32,
    pub timeout_seconds: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// NewTask struct for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub name: String,
    pub prompt: String,
    pub cron_expression: Option<String>,
    pub simple_schedule: Option<String>,
    pub enabled: Option<i32>,
    pub timeout_seconds: Option<i32>,
}

/// UpdateTask struct for updating an existing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub cron_expression: Option<String>,
    pub simple_schedule: Option<String>,
    pub enabled: Option<i32>,
    pub timeout_seconds: Option<i32>,
}

/// Create a new task
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `new_task` - NewTask struct with task data
///
/// # Returns
/// * `Ok(Task)` - Successfully created task
/// * `Err(sqlx::Error)` - Database error
pub async fn create_task(pool: &SqlitePool, new_task: NewTask) -> Result<Task, sqlx::Error> {
    let now: DateTime<Utc> = Utc::now();
    let id = Uuid::new_v4().to_string();
    let created_at = now.to_rfc3339();
    let updated_at = created_at.clone();

    let enabled = new_task.enabled.unwrap_or(1);
    let timeout_seconds = new_task.timeout_seconds.unwrap_or(300);

    sqlx::query(
        r#"
        INSERT INTO tasks (id, name, prompt, cron_expression, simple_schedule, enabled, timeout_seconds, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&new_task.name)
    .bind(&new_task.prompt)
    .bind(&new_task.cron_expression)
    .bind(&new_task.simple_schedule)
    .bind(enabled)
    .bind(timeout_seconds)
    .bind(&created_at)
    .bind(&updated_at)
    .execute(pool)
    .await?;

    Ok(Task {
        id,
        name: new_task.name,
        prompt: new_task.prompt,
        cron_expression: new_task.cron_expression,
        simple_schedule: new_task.simple_schedule,
        enabled,
        timeout_seconds,
        created_at,
        updated_at
    })
}

/// Get a task by ID
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Task ID
///
/// # Returns
/// * `Ok(Task)` - Found task
/// * `Err(sqlx::Error)` - Database error or not found
pub async fn get_task(pool: &SqlitePool, id: &str) -> Result<Task, sqlx::Error> {
    sqlx::query_as::<_, Task>(
        r#"
        SELECT id, name, prompt, cron_expression, simple_schedule, enabled, timeout_seconds, created_at, updated_at
        FROM tasks
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Get all tasks
///
/// # Arguments
/// * `pool` - SQLite connection pool
///
/// # Returns
/// * `Ok(Vec<Task>)` - List of all tasks
/// * `Err(sqlx::Error)` - Database error
pub async fn get_all_tasks(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
    sqlx::query_as::<_, Task>(
        r#"
        SELECT id, name, prompt, cron_expression, simple_schedule, enabled, timeout_seconds, created_at, updated_at
        FROM tasks
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Update an existing task
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Task ID
/// * `update_task` - UpdateTask struct with updated fields
///
/// # Returns
/// * `Ok(Task)` - Updated task
/// * `Err(sqlx::Error)` - Database error or not found
pub async fn update_task(pool: &SqlitePool, id: &str, update: UpdateTask) -> Result<Task, sqlx::Error> {
    let existing = get_task(pool, id).await?;

    let name = update.name.unwrap_or(existing.name);
    let prompt = update.prompt.unwrap_or(existing.prompt);
    let cron_expression = update.cron_expression.or(existing.cron_expression);
    let simple_schedule = update.simple_schedule.or(existing.simple_schedule);
    let enabled = update.enabled.unwrap_or(existing.enabled);
    let timeout_seconds = update.timeout_seconds.unwrap_or(existing.timeout_seconds);
    let updated_at = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE tasks
        SET name = ?, prompt = ?, cron_expression = ?, simple_schedule = ?, enabled = ?, timeout_seconds = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&prompt)
    .bind(&cron_expression)
    .bind(&simple_schedule)
    .bind(enabled)
    .bind(timeout_seconds)
    .bind(&updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(Task {
        id: id.to_string(),
        name,
        prompt,
        cron_expression,
        simple_schedule,
        enabled,
        timeout_seconds,
        created_at: existing.created_at,
        updated_at,
    })
}

/// Delete a task by ID
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Task ID
///
/// # Returns
/// * `Ok(bool)` - True if task was deleted
/// * `Err(sqlx::Error)` - Database error
pub async fn delete_task(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update task's updated_at timestamp
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Task ID
///
/// # Returns
/// * `Ok(())` - Successfully updated
/// * `Err(sqlx::Error)` - Database error
pub async fn touch_task(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let updated_at = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE tasks
        SET updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_database;
    use tempfile::TempDir;

    async fn setup_test_db() -> (SqlitePool, TempDir) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let pool = init_database(&db_path)
            .await
            .expect("Failed to init database");
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_task() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "Test Task".to_string(),
            prompt: "This is a test prompt".to_string(),
            cron_expression: Some("0 * * * *".to_string()),
            simple_schedule: None,
            enabled: Some(1),
            timeout_seconds: Some(300),
        };

        let result = create_task(&pool, new_task).await;

        assert!(result.is_ok(), "Task creation should succeed");
        let task = result.unwrap();
        assert!(!task.id.is_empty(), "Task ID should be generated");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.prompt, "This is a test prompt");
        assert_eq!(task.cron_expression, Some("0 * * * *".to_string()));
        assert_eq!(task.enabled, 1);
        assert_eq!(task.timeout_seconds, 300);
        assert!(!task.created_at.is_empty());
        assert!(!task.updated_at.is_empty());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_task_with_defaults() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "Minimal Task".to_string(),
            prompt: "Simple prompt".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };

        let result = create_task(&pool, new_task).await;

        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.enabled, 1, "Default enabled should be 1");
        assert_eq!(task.timeout_seconds, 300, "Default timeout should be 300");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_task() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "Get Test".to_string(),
            prompt: "Get this task".to_string(),
            cron_expression: None,
            simple_schedule: Some(r#"{"type":"interval","value":5,"unit":"minutes"}"#.to_string()),
            enabled: Some(1),
            timeout_seconds: Some(600),
        };

        let created = create_task(&pool, new_task).await.expect("Failed to create task");

        let result = get_task(&pool, &created.id).await;

        assert!(result.is_ok(), "Should find the task");
        let task = result.unwrap();
        assert_eq!(task.id, created.id);
        assert_eq!(task.name, "Get Test");
        assert_eq!(task.prompt, "Get this task");
        assert_eq!(task.simple_schedule, Some(r#"{"type":"interval","value":5,"unit":"minutes"}"#.to_string()));
        assert_eq!(task.timeout_seconds, 600);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let (pool, _temp_dir) = setup_test_db().await;
        let fake_id = "non-existent-id";

        let result = get_task(&pool, fake_id).await;

        assert!(result.is_err(), "Should return error for non-existent task");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_all_tasks_empty() {
        let (pool, _temp_dir) = setup_test_db().await;

        let result = get_all_tasks(&pool).await;

        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 0, "Should return empty list");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_all_tasks_with_data() {
        let (pool, _temp_dir) = setup_test_db().await;
        
        let task1 = NewTask {
            name: "Task 1".to_string(),
            prompt: "Prompt 1".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };
        let task2 = NewTask {
            name: "Task 2".to_string(),
            prompt: "Prompt 2".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };

        create_task(&pool, task1).await.expect("Failed to create task1");
        create_task(&pool, task2).await.expect("Failed to create task2");

        let result = get_all_tasks(&pool).await;

        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 2, "Should return 2 tasks");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_task() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "Original Name".to_string(),
            prompt: "Original prompt".to_string(),
            cron_expression: Some("0 0 * * *".to_string()),
            simple_schedule: None,
            enabled: Some(1),
            timeout_seconds: Some(300),
        };

        let created = create_task(&pool, new_task).await.expect("Failed to create task");
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let update = UpdateTask {
            name: Some("Updated Name".to_string()),
            prompt: Some("Updated prompt".to_string()),
            cron_expression: None,
            simple_schedule: None,
            enabled: Some(0),
            timeout_seconds: Some(600),
        };

        let result = update_task(&pool, &created.id, update).await;

        assert!(result.is_ok(), "Update should succeed");
        let updated = result.unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.prompt, "Updated prompt");
        assert_eq!(updated.cron_expression, Some("0 0 * * *".to_string()));
        assert_eq!(updated.enabled, 0);
        assert_eq!(updated.timeout_seconds, 600);
        assert_eq!(updated.created_at, created.created_at);
        assert_ne!(updated.updated_at, created.updated_at, "updated_at should change");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_task_partial() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "Original".to_string(),
            prompt: "Original prompt".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: Some(1),
            timeout_seconds: Some(300),
        };

        let created = create_task(&pool, new_task).await.expect("Failed to create task");

        let update = UpdateTask {
            name: Some("Updated Name Only".to_string()),
            prompt: None,
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };

        let result = update_task(&pool, &created.id, update).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Name Only");
        assert_eq!(updated.prompt, "Original prompt");
        assert_eq!(updated.enabled, 1);
        assert_eq!(updated.timeout_seconds, 300);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_task_not_found() {
        let (pool, _temp_dir) = setup_test_db().await;
        let update = UpdateTask {
            name: Some("Updated".to_string()),
            prompt: None,
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };

        let result = update_task(&pool, "non-existent-id", update).await;

        assert!(result.is_err(), "Update should fail for non-existent task");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_task() {
        let (pool, _temp_dir) = setup_test_db().await;
        let new_task = NewTask {
            name: "To Delete".to_string(),
            prompt: "Delete this".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };

        let created = create_task(&pool, new_task).await.expect("Failed to create task");

        let result = delete_task(&pool, &created.id).await;

        assert!(result.is_ok(), "Delete should succeed");
        let deleted = result.unwrap();
        assert!(deleted, "Should return true for deleted task");

        let get_result = get_task(&pool, &created.id).await;
        assert!(get_result.is_err(), "Task should not exist after deletion");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_task_not_found() {
        let (pool, _temp_dir) = setup_test_db().await;

        let result = delete_task(&pool, "non-existent-id").await;

        assert!(result.is_ok(), "Delete should succeed even if task not found");
        let deleted = result.unwrap();
        assert!(!deleted, "Should return false for non-existent task");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_task_full_lifecycle() {
        let (pool, _temp_dir) = setup_test_db().await;

        let new_task = NewTask {
            name: "Lifecycle Task".to_string(),
            prompt: "Test full lifecycle".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: Some(1),
            timeout_seconds: Some(300),
        };
        let created = create_task(&pool, new_task).await.expect("Create failed");

        let fetched = get_task(&pool, &created.id).await.expect("Get failed");
        assert_eq!(fetched.name, "Lifecycle Task");

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let update = UpdateTask {
            name: Some("Updated Lifecycle".to_string()),
            prompt: None,
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
        };
        let updated = update_task(&pool, &created.id, update).await.expect("Update failed");
        assert_eq!(updated.name, "Updated Lifecycle");

        let deleted = delete_task(&pool, &created.id).await.expect("Delete failed");
        assert!(deleted);

        let get_result = get_task(&pool, &created.id).await;
        assert!(get_result.is_err());

        pool.close().await;
    }
}

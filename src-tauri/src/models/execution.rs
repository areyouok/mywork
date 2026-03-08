use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Execution status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Skipped,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStatus::Pending => "pending",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Success => "success",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Timeout => "timeout",
            ExecutionStatus::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(ExecutionStatus::Pending),
            "running" => Some(ExecutionStatus::Running),
            "success" => Some(ExecutionStatus::Success),
            "failed" => Some(ExecutionStatus::Failed),
            "timeout" => Some(ExecutionStatus::Timeout),
            "skipped" => Some(ExecutionStatus::Skipped),
            _ => None,
        }
    }
}

/// Execution model representing a task execution record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Execution {
    pub id: String,
    pub task_id: String,
    pub session_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub output_file: Option<String>,
    pub error_message: Option<String>,
}

/// NewExecution struct for creating a new execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewExecution {
    pub task_id: String,
    pub session_id: Option<String>,
    pub status: Option<ExecutionStatus>,
    pub output_file: Option<String>,
    pub error_message: Option<String>,
}

/// UpdateExecution struct for updating an existing execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExecution {
    pub session_id: Option<String>,
    pub status: Option<ExecutionStatus>,
    pub finished_at: Option<String>,
    pub output_file: Option<String>,
    pub error_message: Option<String>,
}

/// Create a new execution
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `new_execution` - NewExecution struct with execution data
///
/// # Returns
/// * `Ok(Execution)` - Successfully created execution
/// * `Err(sqlx::Error)` - Database error
pub async fn create_execution(
    pool: &SqlitePool,
    new_execution: NewExecution,
) -> Result<Execution, sqlx::Error> {
    let now: DateTime<Utc> = Utc::now();
    let id = Uuid::new_v4().to_string();
    let started_at = now.to_rfc3339();
    let status = new_execution
        .status
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("pending");

    sqlx::query(
        r#"
        INSERT INTO executions (id, task_id, session_id, status, started_at, finished_at, output_file, error_message)
        VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&new_execution.task_id)
    .bind(&new_execution.session_id)
    .bind(status)
    .bind(&started_at)
    .bind(&new_execution.output_file)
    .bind(&new_execution.error_message)
    .execute(pool)
    .await?;

    Ok(Execution {
        id,
        task_id: new_execution.task_id,
        session_id: new_execution.session_id,
        status: status.to_string(),
        started_at,
        finished_at: None,
        output_file: new_execution.output_file,
        error_message: new_execution.error_message,
    })
}

/// Get an execution by ID
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Execution ID
///
/// # Returns
/// * `Ok(Execution)` - Found execution
/// * `Err(sqlx::Error)` - Database error or not found
pub async fn get_execution(pool: &SqlitePool, id: &str) -> Result<Execution, sqlx::Error> {
    sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, task_id, session_id, status, started_at, finished_at, output_file, error_message
        FROM executions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Get all executions for a task
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `task_id` - Task ID
///
/// # Returns
/// * `Ok(Vec<Execution>)` - List of executions for the task
/// * `Err(sqlx::Error)` - Database error
pub async fn get_executions_by_task(
    pool: &SqlitePool,
    task_id: &str,
) -> Result<Vec<Execution>, sqlx::Error> {
    sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, task_id, session_id, status, started_at, finished_at, output_file, error_message
        FROM executions
        WHERE task_id = ?
        ORDER BY started_at DESC
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
}

/// Update an existing execution
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `id` - Execution ID
/// * `update` - UpdateExecution struct with updated fields
///
/// # Returns
/// * `Ok(Execution)` - Updated execution
/// * `Err(sqlx::Error)` - Database error or not found
pub async fn update_execution(
    pool: &SqlitePool,
    id: &str,
    update: UpdateExecution,
) -> Result<Execution, sqlx::Error> {
    let existing = get_execution(pool, id).await?;

    let session_id = update.session_id.or(existing.session_id);
    let status = update
        .status
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&existing.status);
    let finished_at = update.finished_at.or(existing.finished_at);
    let output_file = update.output_file.or(existing.output_file);
    let error_message = update.error_message.or(existing.error_message);

    sqlx::query(
        r#"
        UPDATE executions
        SET session_id = ?, status = ?, finished_at = ?, output_file = ?, error_message = ?
        WHERE id = ?
        "#,
    )
    .bind(&session_id)
    .bind(status)
    .bind(&finished_at)
    .bind(&output_file)
    .bind(&error_message)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(Execution {
        id: id.to_string(),
        task_id: existing.task_id,
        session_id,
        status: status.to_string(),
        started_at: existing.started_at,
        finished_at,
        output_file,
        error_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_database;
    use crate::models::task::{create_task, NewTask};
    use tempfile::TempDir;

    async fn setup_test_db() -> (SqlitePool, TempDir) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let pool = init_database(&db_path)
            .await
            .expect("Failed to init database");
        (pool, temp_dir)
    }

    async fn create_test_task(pool: &SqlitePool) -> String {
        let new_task = NewTask {
            name: "Test Task".to_string(),
            prompt: "Test prompt".to_string(),
            cron_expression: None,
            simple_schedule: None,
            enabled: None,
            timeout_seconds: None,
            skip_if_running: None,
        };
        let task = create_task(pool, new_task).await.expect("Failed to create task");
        task.id
    }

    #[tokio::test]
    async fn test_create_execution() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("session-123".to_string()),
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };

        let result = create_execution(&pool, new_execution).await;

        assert!(result.is_ok(), "Execution creation should succeed");
        let execution = result.unwrap();
        assert!(!execution.id.is_empty(), "Execution ID should be generated");
        assert_eq!(execution.task_id, task_id);
        assert_eq!(execution.session_id, Some("session-123".to_string()));
        assert_eq!(execution.status, "pending");
        assert!(!execution.started_at.is_empty());
        assert!(execution.finished_at.is_none());
        assert!(execution.output_file.is_none());
        assert!(execution.error_message.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_execution_with_defaults() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: None,
            status: None,
            output_file: None,
            error_message: None,
        };

        let result = create_execution(&pool, new_execution).await;

        assert!(result.is_ok());
        let execution = result.unwrap();
        assert_eq!(execution.status, "pending", "Default status should be pending");
        assert!(execution.session_id.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_create_execution_with_all_fields() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("session-456".to_string()),
            status: Some(ExecutionStatus::Running),
            output_file: Some("/path/to/output.txt".to_string()),
            error_message: Some("Initial error".to_string()),
        };

        let result = create_execution(&pool, new_execution).await;

        assert!(result.is_ok());
        let execution = result.unwrap();
        assert_eq!(execution.status, "running");
        assert_eq!(execution.session_id, Some("session-456".to_string()));
        assert_eq!(execution.output_file, Some("/path/to/output.txt".to_string()));
        assert_eq!(execution.error_message, Some("Initial error".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_execution() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("session-789".to_string()),
            status: Some(ExecutionStatus::Success),
            output_file: Some("/result.txt".to_string()),
            error_message: None,
        };

        let created = create_execution(&pool, new_execution)
            .await
            .expect("Failed to create execution");

        let result = get_execution(&pool, &created.id).await;

        assert!(result.is_ok(), "Should find the execution");
        let execution = result.unwrap();
        assert_eq!(execution.id, created.id);
        assert_eq!(execution.task_id, task_id);
        assert_eq!(execution.session_id, Some("session-789".to_string()));
        assert_eq!(execution.status, "success");
        assert_eq!(execution.output_file, Some("/result.txt".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_execution_not_found() {
        let (pool, _temp_dir) = setup_test_db().await;
        let fake_id = "non-existent-id";

        let result = get_execution(&pool, fake_id).await;

        assert!(
            result.is_err(),
            "Should return error for non-existent execution"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_executions_by_task_empty() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let result = get_executions_by_task(&pool, &task_id).await;

        assert!(result.is_ok());
        let executions = result.unwrap();
        assert_eq!(executions.len(), 0, "Should return empty list");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_executions_by_task_with_data() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let execution1 = NewExecution {
            task_id: task_id.clone(),
            session_id: None,
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };
        let execution2 = NewExecution {
            task_id: task_id.clone(),
            session_id: None,
            status: Some(ExecutionStatus::Running),
            output_file: None,
            error_message: None,
        };

        create_execution(&pool, execution1)
            .await
            .expect("Failed to create execution1");
        create_execution(&pool, execution2)
            .await
            .expect("Failed to create execution2");

        let result = get_executions_by_task(&pool, &task_id).await;

        assert!(result.is_ok());
        let executions = result.unwrap();
        assert_eq!(executions.len(), 2, "Should return 2 executions");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_executions_by_task_ordered() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let execution1 = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("first".to_string()),
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };
        create_execution(&pool, execution1)
            .await
            .expect("Failed to create execution1");

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let execution2 = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("second".to_string()),
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };
        create_execution(&pool, execution2)
            .await
            .expect("Failed to create execution2");

        let result = get_executions_by_task(&pool, &task_id).await;

        assert!(result.is_ok());
        let executions = result.unwrap();
        assert_eq!(executions.len(), 2);
        assert_eq!(executions[0].session_id, Some("second".to_string()));
        assert_eq!(executions[1].session_id, Some("first".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_execution_status() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: None,
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };

        let created = create_execution(&pool, new_execution)
            .await
            .expect("Failed to create execution");

        let update = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Running),
            finished_at: None,
            output_file: None,
            error_message: None,
        };

        let result = update_execution(&pool, &created.id, update).await;

        assert!(result.is_ok(), "Update should succeed");
        let updated = result.unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.status, "running");
        assert!(updated.finished_at.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_execution_finish() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: None,
            status: Some(ExecutionStatus::Running),
            output_file: None,
            error_message: None,
        };

        let created = create_execution(&pool, new_execution)
            .await
            .expect("Failed to create execution");

        let finished_at = Utc::now().to_rfc3339();
        let update = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Success),
            finished_at: Some(finished_at.clone()),
            output_file: Some("/output/result.txt".to_string()),
            error_message: None,
        };

        let result = update_execution(&pool, &created.id, update).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.status, "success");
        assert_eq!(updated.finished_at, Some(finished_at));
        assert_eq!(
            updated.output_file,
            Some("/output/result.txt".to_string())
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_execution_partial() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("session-original".to_string()),
            status: Some(ExecutionStatus::Running),
            output_file: None,
            error_message: None,
        };

        let created = create_execution(&pool, new_execution)
            .await
            .expect("Failed to create execution");

        let update = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Failed),
            finished_at: None,
            output_file: None,
            error_message: Some("Task failed".to_string()),
        };

        let result = update_execution(&pool, &created.id, update).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.status, "failed");
        assert_eq!(
            updated.session_id,
            Some("session-original".to_string()),
            "session_id should be preserved"
        );
        assert_eq!(
            updated.error_message,
            Some("Task failed".to_string())
        );
        assert!(updated.finished_at.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_update_execution_not_found() {
        let (pool, _temp_dir) = setup_test_db().await;
        let update = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Success),
            finished_at: None,
            output_file: None,
            error_message: None,
        };

        let result = update_execution(&pool, "non-existent-id", update).await;

        assert!(
            result.is_err(),
            "Update should fail for non-existent execution"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_execution_status_variants() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let statuses = vec![
            ExecutionStatus::Pending,
            ExecutionStatus::Running,
            ExecutionStatus::Success,
            ExecutionStatus::Failed,
            ExecutionStatus::Timeout,
            ExecutionStatus::Skipped,
        ];

        for status in statuses {
            let new_execution = NewExecution {
                task_id: task_id.clone(),
                session_id: None,
                status: Some(status.clone()),
                output_file: None,
                error_message: None,
            };

            let result = create_execution(&pool, new_execution).await;
            assert!(result.is_ok());
            let execution = result.unwrap();
            assert_eq!(execution.status, status.as_str());
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_execution_full_lifecycle() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let new_execution = NewExecution {
            task_id: task_id.clone(),
            session_id: Some("session-lifecycle".to_string()),
            status: Some(ExecutionStatus::Pending),
            output_file: None,
            error_message: None,
        };
        let created = create_execution(&pool, new_execution)
            .await
            .expect("Create failed");
        assert_eq!(created.status, "pending");

        let update1 = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Running),
            finished_at: None,
            output_file: None,
            error_message: None,
        };
        let running = update_execution(&pool, &created.id, update1)
            .await
            .expect("Update to running failed");
        assert_eq!(running.status, "running");

        let finished_at = Utc::now().to_rfc3339();
        let update2 = UpdateExecution {
            session_id: None,
            status: Some(ExecutionStatus::Success),
            finished_at: Some(finished_at.clone()),
            output_file: Some("/output/final.txt".to_string()),
            error_message: None,
        };
        let success = update_execution(&pool, &created.id, update2)
            .await
            .expect("Update to success failed");
        assert_eq!(success.status, "success");
        assert_eq!(success.finished_at, Some(finished_at));

        let fetched = get_execution(&pool, &created.id)
            .await
            .expect("Get failed");
        assert_eq!(fetched.status, "success");
        assert_eq!(fetched.output_file, Some("/output/final.txt".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_multiple_executions_same_task() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        for i in 0..5 {
            let new_execution = NewExecution {
                task_id: task_id.clone(),
                session_id: Some(format!("session-{}", i)),
                status: Some(ExecutionStatus::Success),
                output_file: None,
                error_message: None,
            };
            create_execution(&pool, new_execution)
                .await
                .expect("Failed to create execution");
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        let executions = get_executions_by_task(&pool, &task_id)
            .await
            .expect("Failed to get executions");

        assert_eq!(executions.len(), 5);
        assert_eq!(executions[0].session_id, Some("session-4".to_string()));
        assert_eq!(executions[4].session_id, Some("session-0".to_string()));

        pool.close().await;
    }
}

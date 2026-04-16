use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::str::FromStr;
use uuid::Uuid;

pub const EXECUTION_HISTORY_LIMIT: i64 = 20;

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
}

impl FromStr for ExecutionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ExecutionStatus::Pending),
            "running" => Ok(ExecutionStatus::Running),
            "success" => Ok(ExecutionStatus::Success),
            "failed" => Ok(ExecutionStatus::Failed),
            "timeout" => Ok(ExecutionStatus::Timeout),
            "skipped" => Ok(ExecutionStatus::Skipped),
            _ => Err(format!("Invalid execution status: {}", s)),
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

pub fn generate_output_file_name(execution_id: &str, timestamp: &DateTime<Utc>) -> String {
    let ts = timestamp.format("%Y%m%d_%H%M%S_%3f");
    format!("{}_{}.jsonl", execution_id, ts)
}

pub async fn get_execution_ids_exceeding_limit(
    pool: &SqlitePool,
    keep_latest: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let keep_latest = keep_latest.max(0);

    sqlx::query_scalar::<_, String>(
        r#"
        SELECT id
        FROM executions
        ORDER BY started_at DESC, id DESC
        LIMIT -1 OFFSET ?
        "#,
    )
    .bind(keep_latest)
    .fetch_all(pool)
    .await
}

pub async fn get_stale_terminal_executions(
    pool: &SqlitePool,
    keep_latest: i64,
) -> Result<Vec<Execution>, sqlx::Error> {
    let keep_latest = keep_latest.max(0);

    sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, task_id, session_id, status, started_at, finished_at, output_file, error_message
        FROM executions
        WHERE status != 'running' AND id IN (
            SELECT id
            FROM executions
            WHERE status != 'running'
            ORDER BY started_at DESC, id DESC
            LIMIT -1 OFFSET ?
        )
        ORDER BY started_at ASC, id ASC
        "#,
    )
    .bind(keep_latest)
    .fetch_all(pool)
    .await
}

pub async fn delete_executions_by_ids(
    pool: &SqlitePool,
    execution_ids: &[String],
) -> Result<u64, sqlx::Error> {
    if execution_ids.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut deleted_count = 0;

    for execution_id in execution_ids {
        let result = sqlx::query(
            r#"
            DELETE FROM executions
            WHERE id = ?
            "#,
        )
        .bind(execution_id)
        .execute(&mut *tx)
        .await?;

        deleted_count += result.rows_affected();
    }

    tx.commit().await?;

    Ok(deleted_count)
}

pub async fn prune_execution_history(
    pool: &SqlitePool,
    keep_latest: i64,
) -> Result<Vec<Execution>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let keep_latest = keep_latest.max(0);

    let stale_executions = sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, task_id, session_id, status, started_at, finished_at, output_file, error_message
        FROM executions
        WHERE status != 'running' AND id IN (
            SELECT id
            FROM executions
            WHERE status != 'running'
            ORDER BY started_at DESC, id DESC
            LIMIT -1 OFFSET ?
        )
        ORDER BY started_at ASC, id ASC
        "#,
    )
    .bind(keep_latest)
    .fetch_all(&mut *tx)
    .await?;

    if !stale_executions.is_empty() {
        for stale_execution in &stale_executions {
            sqlx::query(
                r#"
                DELETE FROM executions
                WHERE id = ?
                "#,
            )
            .bind(&stale_execution.id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(stale_executions)
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

/// Get all executions with a specific status
///
/// # Arguments
/// * `pool` - SQLite connection pool
/// * `status` - Execution status to filter by
///
/// # Returns
/// * `Ok(Vec<Execution>)` - List of executions with the specified status
/// * `Err(sqlx::Error)` - Database error
pub async fn get_executions_by_status(
    pool: &SqlitePool,
    status: ExecutionStatus,
) -> Result<Vec<Execution>, sqlx::Error> {
    sqlx::query_as::<_, Execution>(
        r#"
        SELECT id, task_id, session_id, status, started_at, finished_at, output_file, error_message
        FROM executions
        WHERE status = ?
        ORDER BY started_at DESC
        "#,
    )
    .bind(status.as_str())
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

pub async fn update_execution_if_running(
    pool: &SqlitePool,
    id: &str,
    update: UpdateExecution,
) -> Result<Option<Execution>, sqlx::Error> {
    let existing = get_execution(pool, id).await?;

    if existing.status != ExecutionStatus::Running.as_str() {
        return Ok(None);
    }

    let session_id = update.session_id.or(existing.session_id);
    let status = update
        .status
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&existing.status);
    let finished_at = update.finished_at.or(existing.finished_at);
    let output_file = update.output_file.or(existing.output_file);
    let error_message = update.error_message.or(existing.error_message);

    let result = sqlx::query(
        r#"
        UPDATE executions
        SET session_id = ?, status = ?, finished_at = ?, output_file = ?, error_message = ?
        WHERE id = ? AND status = 'running'
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

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    let updated = get_execution(pool, id).await?;
    Ok(Some(updated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_database;
    use crate::models::task::{create_task, NewTask};
    use std::collections::HashSet;
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
            once_at: None,
            enabled: None,
            timeout_seconds: None,
            working_directory: None,
        };
        let task = create_task(pool, new_task)
            .await
            .expect("Failed to create task");
        task.id
    }

    #[test]
    fn test_generate_output_file_name() {
        let timestamp = DateTime::parse_from_rfc3339("2026-03-11T12:34:56.789Z")
            .expect("timestamp should parse")
            .with_timezone(&Utc);

        let file_name = generate_output_file_name("exec-123", &timestamp);

        assert_eq!(file_name, "exec-123_20260311_123456_789.jsonl");
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
        assert!(
            Uuid::parse_str(&execution.id).is_ok(),
            "Execution ID should be UUID"
        );
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
        assert_eq!(
            execution.status, "pending",
            "Default status should be pending"
        );
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
            output_file: Some("/path/to/output.jsonl".to_string()),
            error_message: Some("Initial error".to_string()),
        };

        let result = create_execution(&pool, new_execution).await;

        assert!(result.is_ok());
        let execution = result.unwrap();
        assert_eq!(execution.status, "running");
        assert_eq!(execution.session_id, Some("session-456".to_string()));
        assert_eq!(
            execution.output_file,
            Some("/path/to/output.jsonl".to_string())
        );
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
            output_file: Some("/result.jsonl".to_string()),
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
        assert_eq!(execution.output_file, Some("/result.jsonl".to_string()));

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
            output_file: Some("/output/result.jsonl".to_string()),
            error_message: None,
        };

        let result = update_execution(&pool, &created.id, update).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.status, "success");
        assert_eq!(updated.finished_at, Some(finished_at));
        assert_eq!(updated.output_file, Some("/output/result.jsonl".to_string()));

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
        assert_eq!(updated.error_message, Some("Task failed".to_string()));
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
            output_file: Some("/output/final.jsonl".to_string()),
            error_message: None,
        };
        let success = update_execution(&pool, &created.id, update2)
            .await
            .expect("Update to success failed");
        assert_eq!(success.status, "success");
        assert_eq!(success.finished_at, Some(finished_at));

        let fetched = get_execution(&pool, &created.id).await.expect("Get failed");
        assert_eq!(fetched.status, "success");
        assert_eq!(fetched.output_file, Some("/output/final.jsonl".to_string()));

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

    #[tokio::test]
    async fn test_get_execution_ids_exceeding_limit_returns_oldest_execution_ids() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let mut created_execution_ids = Vec::new();
        for i in 0..25 {
            let execution = create_execution(
                &pool,
                NewExecution {
                    task_id: task_id.clone(),
                    session_id: Some(format!("session-{}", i)),
                    status: Some(ExecutionStatus::Success),
                    output_file: None,
                    error_message: None,
                },
            )
            .await
            .expect("Failed to create execution");

            created_execution_ids.push(execution.id);
            tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        }

        let stale_ids = get_execution_ids_exceeding_limit(&pool, 20)
            .await
            .expect("Failed to query stale execution ids");

        assert_eq!(
            stale_ids.len(),
            5,
            "Should return only 5 stale execution ids"
        );

        let expected_ids: HashSet<String> = created_execution_ids[..5].iter().cloned().collect();
        let actual_ids: HashSet<String> = stale_ids.into_iter().collect();
        assert_eq!(
            actual_ids, expected_ids,
            "Should target the oldest 5 executions"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_executions_by_ids_removes_only_selected_records() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let mut created_execution_ids = Vec::new();
        for i in 0..3 {
            let execution = create_execution(
                &pool,
                NewExecution {
                    task_id: task_id.clone(),
                    session_id: Some(format!("session-{}", i)),
                    status: Some(ExecutionStatus::Success),
                    output_file: None,
                    error_message: None,
                },
            )
            .await
            .expect("Failed to create execution");

            created_execution_ids.push(execution.id);
        }

        let deleted_count = delete_executions_by_ids(&pool, &created_execution_ids[..2])
            .await
            .expect("Failed to delete executions");

        assert_eq!(
            deleted_count, 2,
            "Should delete exactly 2 execution records"
        );

        let remaining = get_executions_by_task(&pool, &task_id)
            .await
            .expect("Failed to query executions");

        assert_eq!(remaining.len(), 1, "Only one execution should remain");
        assert_eq!(remaining[0].id, created_execution_ids[2]);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_stale_terminal_executions_skips_running_records() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        for i in 0..20 {
            create_execution(
                &pool,
                NewExecution {
                    task_id: task_id.clone(),
                    session_id: Some(format!("success-{}", i)),
                    status: Some(ExecutionStatus::Success),
                    output_file: None,
                    error_message: None,
                },
            )
            .await
            .expect("Failed to create success execution");

            tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        }

        let running_execution = create_execution(
            &pool,
            NewExecution {
                task_id,
                session_id: Some("running-session".to_string()),
                status: Some(ExecutionStatus::Running),
                output_file: None,
                error_message: None,
            },
        )
        .await
        .expect("Failed to create running execution");

        let stale = get_stale_terminal_executions(&pool, 20)
            .await
            .expect("Failed to query stale terminal executions");

        assert_eq!(stale.len(), 0, "No terminal execution should be stale");
        assert!(!stale.iter().any(|e| e.id == running_execution.id));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_prune_execution_history_deletes_only_stale_terminal_records() {
        let (pool, _temp_dir) = setup_test_db().await;
        let task_id = create_test_task(&pool).await;

        let mut terminal_execution_ids = Vec::new();
        for i in 0..22 {
            let execution = create_execution(
                &pool,
                NewExecution {
                    task_id: task_id.clone(),
                    session_id: Some(format!("done-{}", i)),
                    status: Some(ExecutionStatus::Success),
                    output_file: Some(format!("done-{}.jsonl", i)),
                    error_message: None,
                },
            )
            .await
            .expect("Failed to create terminal execution");

            terminal_execution_ids.push(execution.id);
            tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        }

        let running_execution = create_execution(
            &pool,
            NewExecution {
                task_id,
                session_id: Some("running-session".to_string()),
                status: Some(ExecutionStatus::Running),
                output_file: Some("running.jsonl".to_string()),
                error_message: None,
            },
        )
        .await
        .expect("Failed to create running execution");

        let pruned = prune_execution_history(&pool, 20)
            .await
            .expect("Failed to prune execution history");

        assert_eq!(
            pruned.len(),
            2,
            "Should prune exactly 2 stale terminal records"
        );

        let expected_pruned: HashSet<String> =
            terminal_execution_ids[..2].iter().cloned().collect();
        let actual_pruned: HashSet<String> = pruned.into_iter().map(|e| e.id).collect();
        assert_eq!(actual_pruned, expected_pruned);

        let still_running = get_execution(&pool, &running_execution.id)
            .await
            .expect("Running execution should remain");
        assert_eq!(still_running.status, "running");

        let terminal_remaining = get_executions_by_task(&pool, &still_running.task_id)
            .await
            .expect("Failed to query executions");
        let terminal_count = terminal_remaining
            .iter()
            .filter(|execution| execution.status != "running")
            .count();
        assert_eq!(terminal_count, 20, "Should keep 20 terminal records");

        pool.close().await;
    }
}

use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Initialize database connection pool and create schema
///
/// # Arguments
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
/// * `Ok(SqlitePool)` - Successfully initialized connection pool
/// * `Err(sqlx::Error)` - Database initialization failed
pub async fn init_database(db_path: &PathBuf) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| sqlx::Error::Io(e))?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    let schema = include_str!("schema.sql");
    for statement in schema.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(&pool)
                .await?;
        }
    }

    Ok(pool)
}

pub fn get_database_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    Ok(app_data_dir.join("mywork.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_database_creates_file() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Act
        let pool = init_database(&db_path).await;

        // Assert
        assert!(pool.is_ok(), "Database initialization should succeed");
        assert!(db_path.exists(), "Database file should be created");
        pool.unwrap().close().await;
    }

    #[tokio::test]
    async fn test_init_database_creates_tables() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Act
        let pool = init_database(&db_path).await.expect("Failed to init database");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok(), "tasks table should exist");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='executions'")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok(), "executions table should exist");
        pool.close().await;
    }

    #[tokio::test]
    async fn test_init_database_creates_indexes() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Act
        let pool = init_database(&db_path).await.expect("Failed to init database");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_executions_task_id'")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok(), "idx_executions_task_id should exist");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_executions_started_at'")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok(), "idx_executions_started_at should exist");
        pool.close().await;
    }

    #[tokio::test]
    async fn test_init_database_idempotent() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool1 = init_database(&db_path).await.expect("First init failed");
        pool1.close().await;

        let pool2 = init_database(&db_path).await.expect("Second init failed");

        let result = sqlx::query("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool2)
            .await;
        assert!(result.is_ok(), "Database should be valid after re-init");
        pool2.close().await;
    }
}

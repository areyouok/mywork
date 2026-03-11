use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub async fn init_database(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(sqlx::Error::Io)?;
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

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let has_legacy_column: bool = match sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('tasks') WHERE name='skip_if_running'",
    )
    .fetch_one(pool)
    .await {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to check legacy schema: {}", e);
            false
        }
    };

    if has_legacy_column {
        sqlx::query("ALTER TABLE tasks DROP COLUMN skip_if_running")
            .execute(pool)
            .await?;
    }

    let has_once_at_column: bool = match sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('tasks') WHERE name='once_at'",
    )
    .fetch_one(pool)
    .await
    {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to check once_at schema: {}", e);
            false
        }
    };

    if !has_once_at_column {
        sqlx::query("ALTER TABLE tasks ADD COLUMN once_at TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub fn get_database_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    Ok(app_data_dir.join("mywork.db"))
}

pub fn get_database_directory(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    Ok(app_data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_database_creates_file() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = init_database(&db_path).await;

        assert!(pool.is_ok(), "Database initialization should succeed");
        assert!(db_path.exists(), "Database file should be created");
        pool.unwrap().close().await;
    }

    #[tokio::test]
    async fn test_init_database_creates_tables() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

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
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

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

    #[tokio::test]
    async fn test_migration_removes_skip_if_running() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
            .await
            .expect("Failed to connect");

        sqlx::query(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                prompt TEXT NOT NULL,
                skip_if_running INTEGER DEFAULT 1
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create table with legacy column");

        pool.close().await;

        let pool2 = init_database(&db_path).await.expect("Failed to re-init");

        let column_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('tasks') WHERE name='skip_if_running'",
        )
        .fetch_one(&pool2)
        .await
        .expect("Failed to check column");

        assert!(!column_exists, "skip_if_running column should be removed");
        pool2.close().await;
    }

    #[tokio::test]
    async fn test_migration_adds_once_at_column() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
            .await
            .expect("Failed to connect");

        sqlx::query(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                prompt TEXT NOT NULL,
                cron_expression TEXT,
                simple_schedule TEXT,
                enabled INTEGER DEFAULT 1,
                timeout_seconds INTEGER DEFAULT 300,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create legacy tasks table");

        sqlx::query(
            "CREATE TABLE executions (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                session_id TEXT,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                output_file TEXT,
                error_message TEXT,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create executions table");

        pool.close().await;

        let pool2 = init_database(&db_path).await.expect("Failed to run migration");

        let has_once_at: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('tasks') WHERE name='once_at'",
        )
        .fetch_one(&pool2)
        .await
        .expect("Failed to check once_at column");

        assert!(has_once_at, "once_at column should be added by migration");
        pool2.close().await;
    }
}

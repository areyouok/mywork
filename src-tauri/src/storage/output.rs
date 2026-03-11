use chrono::{DateTime, Duration, Utc};
use std::path::Path;
use std::io;
use tokio::io::AsyncWriteExt;
use tauri::{AppHandle, Manager};

/// Get the output directory path for the application
///
/// # Arguments
/// * `app` - Tauri application handle
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the outputs directory
/// * `Err(io::Error)` - Failed to get app data directory
pub fn get_output_directory(app: &AppHandle) -> Result<std::path::PathBuf, io::Error> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(io::Error::other)?;
    Ok(app_data_dir.join("outputs"))
}

/// Create the output directory if it doesn't exist
///
/// # Arguments
/// * `output_dir` - Path to the output directory
///
/// # Returns
/// * `Ok(())` - Directory created or already exists
/// * `Err(io::Error)` - Failed to create directory
pub async fn create_output_directory(output_dir: &Path) -> Result<(), io::Error> {
    tokio::fs::create_dir_all(output_dir).await
}

/// Write content to an output file for a specific execution
///
/// # Arguments
/// * `output_dir` - Path to the output directory
/// * `content` - Content to write to the file
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created file
/// * `Err(io::Error)` - Failed to write file
pub async fn write_output_file(
    output_dir: &Path,
    output_file_name: &str,
    content: &str,
) -> Result<std::path::PathBuf, io::Error> {
    let file_path = output_dir.join(output_file_name);
    tokio::fs::write(&file_path, content).await?;
    Ok(file_path)
}

pub async fn append_output_file(
    output_dir: &Path,
    output_file_name: &str,
    content: &str,
) -> Result<std::path::PathBuf, io::Error> {
    let file_path = output_dir.join(output_file_name);
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await?;
    file.write_all(content.as_bytes()).await?;
    Ok(file_path)
}

/// Read content from an output file
///
/// # Arguments
/// * `output_dir` - Path to the output directory
///
/// # Returns
/// * `Ok(String)` - Content of the file
/// * `Err(io::Error)` - Failed to read file
pub async fn read_output_file(
    output_dir: &Path,
    output_file_name: &str,
) -> Result<String, io::Error> {
    let file_path = output_dir.join(output_file_name);
    tokio::fs::read_to_string(&file_path).await
}

/// Delete an output file
///
/// # Arguments
/// * `output_dir` - Path to the output directory
///
/// # Returns
/// * `Ok(())` - File deleted successfully
/// * `Err(io::Error)` - Failed to delete file
pub async fn delete_output_file(
    output_dir: &Path,
    output_file_name: &str,
) -> Result<(), io::Error> {
    let file_path = output_dir.join(output_file_name);
    tokio::fs::remove_file(&file_path).await
}

/// Clean up output files older than a specified number of days
///
/// # Arguments
/// * `output_dir` - Path to the output directory
/// * `days_to_keep` - Number of days to keep files (files older than this will be deleted)
///
/// # Returns
/// * `Ok(u64)` - Number of files deleted
/// * `Err(io::Error)` - Failed to clean up files
pub async fn cleanup_old_outputs(
    output_dir: &Path,
    days_to_keep: i64,
) -> Result<u64, io::Error> {
    let cutoff_time = Utc::now() - Duration::days(days_to_keep);
    let mut deleted_count = 0;

    let mut entries = tokio::fs::read_dir(output_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.extension().is_some_and(|ext| ext == "txt") {
            let metadata = entry.metadata().await?;
            
            if let Ok(modified_time) = metadata.modified() {
                let modified_datetime: DateTime<Utc> = modified_time.into();
                
                if modified_datetime < cutoff_time {
                    tokio::fs::remove_file(&path).await?;
                    deleted_count += 1;
                }
            }
        }
    }

    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_create_output_directory() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");

        // Act
        let result = create_output_directory(&output_dir).await;

        // Assert
        assert!(result.is_ok(), "Directory creation should succeed");
        assert!(output_dir.exists(), "Output directory should exist");
    }

    #[tokio::test]
    async fn test_create_output_directory_idempotent() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");

        // Act
        let result1 = create_output_directory(&output_dir).await;
        let result2 = create_output_directory(&output_dir).await;

        // Assert
        assert!(result1.is_ok(), "First creation should succeed");
        assert!(result2.is_ok(), "Second creation should succeed");
        assert!(output_dir.exists(), "Output directory should exist");
    }

    #[tokio::test]
    async fn test_write_output_file() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        let output_file_name = "test-exec-123_20260311_120000_123.txt";
        let content = "This is test output content";

        // Act
        let result = write_output_file(&output_dir, output_file_name, content).await;

        // Assert
        assert!(result.is_ok(), "File write should succeed");
        let file_path = result.unwrap();
        assert!(file_path.exists(), "Output file should exist");
        assert_eq!(
            file_path.file_name().unwrap(),
            "test-exec-123_20260311_120000_123.txt"
        );
    }

    #[tokio::test]
    async fn test_read_output_file() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        let output_file_name = "test-exec-456_20260311_120001_456.txt";
        let content = "This is test output content for reading";
        write_output_file(&output_dir, output_file_name, content)
            .await
            .expect("Failed to write file");

        // Act
        let result = read_output_file(&output_dir, output_file_name).await;

        // Assert
        assert!(result.is_ok(), "File read should succeed");
        assert_eq!(result.unwrap(), content);
    }

    #[tokio::test]
    async fn test_append_output_file() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");

        let output_file_name = "test-exec-append_20260311_120002_789.txt";
        append_output_file(&output_dir, output_file_name, "line1\n")
            .await
            .expect("failed to append line1");
        append_output_file(&output_dir, output_file_name, "line2\n")
            .await
            .expect("failed to append line2");

        let content = read_output_file(&output_dir, output_file_name)
            .await
            .expect("failed to read output");
        assert_eq!(content, "line1\nline2\n");
    }

    #[tokio::test]
    async fn test_cleanup_old_outputs_mixed_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        write_output_file(
            &output_dir,
            "recent-exec_20260311_120003_111.txt",
            "recent content",
        )
        .await
        .expect("Failed to write file");
        
        let old_file = output_dir.join("old-exec.txt");
        fs::write(&old_file, "old content").await.expect("Failed to write file");
        let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .expect("Failed to set file time");

        let result = cleanup_old_outputs(&output_dir, 30).await;

        assert!(result.is_ok(), "Cleanup should succeed");
        assert_eq!(result.unwrap(), 1, "Only old file should be deleted");
        
        let recent_path = output_dir.join("recent-exec_20260311_120003_111.txt");
        assert!(recent_path.exists(), "Recent file should still exist");
        assert!(!old_file.exists(), "Old file should be deleted");
    }

    #[tokio::test]
    async fn test_delete_output_file() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        let output_file_name = "test-exec-789_20260311_120004_222.txt";
        write_output_file(&output_dir, output_file_name, "content")
            .await
            .expect("Failed to write file");

        // Act
        let result = delete_output_file(&output_dir, output_file_name).await;

        // Assert
        assert!(result.is_ok(), "File deletion should succeed");
        let file_path = output_dir.join(output_file_name);
        assert!(!file_path.exists(), "File should be deleted");
    }

    #[tokio::test]
    async fn test_delete_output_file_not_found() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        let output_file_name = "non-existent_20260311_120005_333.txt";

        // Act
        let result = delete_output_file(&output_dir, output_file_name).await;

        // Assert
        assert!(result.is_err(), "Deleting non-existent file should fail");
    }

    #[tokio::test]
    async fn test_cleanup_old_outputs_no_files() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");

        // Act
        let result = cleanup_old_outputs(&output_dir, 30).await;

        // Assert
        assert!(result.is_ok(), "Cleanup should succeed even with no files");
        assert_eq!(result.unwrap(), 0, "No files should be deleted");
    }

    #[tokio::test]
    async fn test_cleanup_old_outputs_keeps_recent_files() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        // Create a recent file
        write_output_file(
            &output_dir,
            "recent-exec_20260311_120006_444.txt",
            "recent content",
        )
        .await
        .expect("Failed to write file");

        // Act
        let result = cleanup_old_outputs(&output_dir, 30).await;

        // Assert
        assert!(result.is_ok(), "Cleanup should succeed");
        assert_eq!(result.unwrap(), 0, "Recent files should not be deleted");
        
        let file_path = output_dir.join("recent-exec_20260311_120006_444.txt");
        assert!(file_path.exists(), "Recent file should still exist");
    }

    #[tokio::test]
    async fn test_cleanup_old_outputs_deletes_old_files() {
        // Arrange
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        // Create a file and set its modification time to 31 days ago
        let old_file = output_dir.join("old-exec.txt");
        fs::write(&old_file, "old content").await.expect("Failed to write file");
        
        let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .expect("Failed to set file time");

        // Act
        let result = cleanup_old_outputs(&output_dir, 30).await;

        // Assert
        assert!(result.is_ok(), "Cleanup should succeed");
        assert_eq!(result.unwrap(), 1, "Old file should be deleted");
        assert!(!old_file.exists(), "Old file should be deleted");
    }

    #[tokio::test]
    async fn test_cleanup_ignores_non_txt_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("outputs");
        create_output_directory(&output_dir).await.expect("Failed to create dir");
        
        let old_file = output_dir.join("old-file.log");
        fs::write(&old_file, "old content").await.expect("Failed to write file");
        let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .expect("Failed to set file time");

        let result = cleanup_old_outputs(&output_dir, 30).await;

        assert!(result.is_ok(), "Cleanup should succeed");
        assert_eq!(result.unwrap(), 0, "Non-txt files should not be deleted");
        assert!(old_file.exists(), "Non-txt file should still exist");
    }
}

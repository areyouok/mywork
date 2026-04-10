use std::path::{Path, PathBuf};

/// Resolve the working directory for task execution
///
/// If working_directory is specified and is a valid directory, use it.
/// Otherwise fall back to the default directory.
///
/// # Arguments
/// * `working_directory` - Optional custom working directory from task config
/// * `default_dir` - Default directory to use if custom is not specified or invalid
///
/// # Returns
/// * `PathBuf` - The resolved working directory path
pub fn resolve_working_directory(
    working_directory: &Option<String>,
    default_dir: &Path,
) -> PathBuf {
    if let Some(ref dir) = working_directory {
        let path = Path::new(dir);
        // Check if path exists and is a directory
        if path.exists() && path.is_dir() {
            return path.to_path_buf();
        }
        // Path doesn't exist or isn't a directory, fall back to default
        eprintln!(
            "Warning: Specified working directory '{}' is invalid or does not exist, using default",
            dir
        );
    }
    default_dir.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_working_directory_with_valid_custom_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let custom_dir = temp_dir.path().join("custom_work");
        std::fs::create_dir(&custom_dir).expect("Failed to create custom dir");

        let default_dir = temp_dir.path().join("default");

        let result = resolve_working_directory(
            &Some(custom_dir.to_string_lossy().to_string()),
            &default_dir,
        );

        assert_eq!(result, custom_dir);
    }

    #[test]
    fn test_resolve_working_directory_with_none_uses_default() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let default_dir = temp_dir.path().join("default");
        std::fs::create_dir(&default_dir).expect("Failed to create default dir");

        let result = resolve_working_directory(&None, &default_dir);

        assert_eq!(result, default_dir);
    }

    #[test]
    fn test_resolve_working_directory_with_nonexistent_dir_uses_default() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let nonexistent_dir = temp_dir.path().join("does_not_exist");
        let default_dir = temp_dir.path().join("default");
        std::fs::create_dir(&default_dir).expect("Failed to create default dir");

        let result = resolve_working_directory(
            &Some(nonexistent_dir.to_string_lossy().to_string()),
            &default_dir,
        );

        assert_eq!(result, default_dir);
    }

    #[test]
    fn test_resolve_working_directory_with_file_instead_of_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("a_file.txt");
        std::fs::write(&file_path, "content").expect("Failed to create file");
        let default_dir = temp_dir.path().join("default");
        std::fs::create_dir(&default_dir).expect("Failed to create default dir");

        let result =
            resolve_working_directory(&Some(file_path.to_string_lossy().to_string()), &default_dir);

        assert_eq!(result, default_dir);
    }
}

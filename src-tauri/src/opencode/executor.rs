use crate::scheduler::timeout::{run_with_timeout, TimeoutError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OpenCode execution error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpenCodeError {
    /// CLI execution failed
    ExecutionFailed { message: String },
    /// Process timed out
    Timeout { timeout_secs: u64 },
    /// Failed to spawn process
    SpawnFailed { message: String },
    /// Invalid session ID
    InvalidSession { message: String },
    /// Output parsing failed
    OutputParseFailed { message: String },
}

impl std::fmt::Display for OpenCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenCodeError::ExecutionFailed { message } => {
                write!(f, "OpenCode execution failed: {}", message)
            }
            OpenCodeError::Timeout { timeout_secs } => {
                write!(f, "OpenCode execution timed out after {} seconds", timeout_secs)
            }
            OpenCodeError::SpawnFailed { message } => {
                write!(f, "Failed to spawn OpenCode process: {}", message)
            }
            OpenCodeError::InvalidSession { message } => {
                write!(f, "Invalid session: {}", message)
            }
            OpenCodeError::OutputParseFailed { message } => {
                write!(f, "Failed to parse OpenCode output: {}", message)
            }
        }
    }
}

impl std::error::Error for OpenCodeError {}

impl From<TimeoutError> for OpenCodeError {
    fn from(error: TimeoutError) -> Self {
        match error {
            TimeoutError::Timeout { timeout_secs } => OpenCodeError::Timeout { timeout_secs },
            TimeoutError::SpawnFailed { message } => OpenCodeError::SpawnFailed { message },
            TimeoutError::KillFailed { pid, message } => OpenCodeError::ExecutionFailed {
                message: format!("Failed to kill process {}: {}", pid, message),
            },
            TimeoutError::ExecutionFailed { message } => OpenCodeError::ExecutionFailed { message },
        }
    }
}

/// Output from OpenCode execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenCodeOutput {
    /// Session ID for this execution
    pub session_id: String,
    /// Standard output from OpenCode
    pub stdout: String,
    /// Standard error from OpenCode
    pub stderr: String,
    /// Whether the execution was successful
    pub success: bool,
    /// Whether the execution timed out
    pub timed_out: bool,
}

/// Configuration for OpenCode execution
#[derive(Debug, Clone)]
pub struct OpenCodeConfig {
    /// Path to opencode binary (defaults to "opencode" in PATH)
    pub binary_path: String,
    /// Default timeout in seconds (defaults to 300)
    pub default_timeout_secs: u64,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            binary_path: "opencode".to_string(),
            default_timeout_secs: 300,
        }
    }
}

/// Session manager for OpenCode executions
#[derive(Debug, Clone)]
pub struct SessionManager {
    /// Current session ID (if any)
    current_session: Option<String>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            current_session: None,
        }
    }

    /// Create a new session ID
    pub fn create_session(&mut self) -> String {
        let session_id = format!("sess_{}", Uuid::new_v4());
        self.current_session = Some(session_id.clone());
        session_id
    }

    /// Get current session ID (if exists)
    pub fn get_session(&self) -> Option<&str> {
        self.current_session.as_deref()
    }

    /// Set a specific session ID
    pub fn set_session(&mut self, session_id: String) {
        self.current_session = Some(session_id);
    }

    /// Clear the current session
    pub fn clear_session(&mut self) {
        self.current_session = None;
    }

    /// Get or create a session ID
    pub fn get_or_create_session(&mut self) -> String {
        if let Some(session) = &self.current_session {
            session.clone()
        } else {
            self.create_session()
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Run an OpenCode task with timeout control
///
/// # Arguments
/// * `prompt` - The prompt to send to OpenCode
/// * `session_id` - Optional session ID to reuse (None creates new session)
/// * `timeout_secs` - Timeout in seconds (None uses default)
/// * `config` - Optional configuration (None uses defaults)
///
/// # Returns
/// `Ok(OpenCodeOutput)` if execution completes, `Err(OpenCodeError)` on failure
///
/// # Example
/// ```no_run
/// use tauri_app_lib::opencode::executor::{run_opencode_task, OpenCodeConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let output = run_opencode_task(
///         "Write a hello world program",
///         None,
///         Some(60),
///         None::<&OpenCodeConfig>,
///     ).await.unwrap();
///     
///     assert!(output.success);
///     println!("Session ID: {}", output.session_id);
/// }
/// ```
pub async fn run_opencode_task(
    prompt: &str,
    session_id: Option<&str>,
    timeout_secs: Option<u64>,
    config: Option<&OpenCodeConfig>,
) -> Result<OpenCodeOutput, OpenCodeError> {
    let config = config.cloned().unwrap_or_default();
    let timeout = timeout_secs.unwrap_or(config.default_timeout_secs);

    // Build command arguments
    let mut args = Vec::new();

    // Add session argument if provided
    if let Some(sid) = session_id {
        args.push("--session".to_string());
        args.push(sid.to_string());
    }

    // Add the prompt
    args.push("--prompt".to_string());
    args.push(prompt.to_string());

    // Execute the command with timeout
    let process_output = run_with_timeout(&config.binary_path, &args.iter().map(String::as_str).collect::<Vec<_>>(), timeout)
        .await
        .map_err(OpenCodeError::from)?;

    // Determine session ID from output or use provided one
    let result_session_id = session_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Try to parse session ID from stdout, or generate a new one
            parse_session_from_output(&process_output.stdout)
                .unwrap_or_else(|| format!("sess_{}", Uuid::new_v4()))
        });

    let success = process_output.success();
    let timed_out = process_output.timed_out;

    Ok(OpenCodeOutput {
        session_id: result_session_id,
        stdout: process_output.stdout,
        stderr: process_output.stderr,
        success,
        timed_out,
    })
}

/// Parse session ID from OpenCode output
///
/// OpenCode outputs session ID in format: "Session: sess_xxx"
fn parse_session_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Session:") || line.starts_with("session:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let session_id = parts[1].trim();
                if !session_id.is_empty() {
                    return Some(session_id.to_string());
                }
            }
        }
    }
    None
}

/// Create a new OpenCode session
///
/// # Arguments
/// * `config` - Optional configuration
///
/// # Returns
/// `Ok(session_id)` if session created successfully, `Err` otherwise
pub async fn create_session(
    config: Option<&OpenCodeConfig>,
) -> Result<String, OpenCodeError> {
    let config = config.cloned().unwrap_or_default();

    // Execute opencode with --new-session flag
    let args = vec!["--new-session"];
    let output = run_with_timeout(&config.binary_path, &args, 30)
        .await
        .map_err(OpenCodeError::from)?;

    if !output.success() {
        return Err(OpenCodeError::ExecutionFailed {
            message: output.stderr,
        });
    }

    // Parse session ID from output
    parse_session_from_output(&output.stdout).ok_or_else(|| {
        OpenCodeError::OutputParseFailed {
            message: "Failed to parse session ID from output".to_string(),
        }
    })
}

/// Run a mock OpenCode task for testing
///
/// This function simulates OpenCode execution without actually calling the CLI.
/// Useful for testing the integration without requiring OpenCode to be installed.
#[cfg(test)]
pub async fn run_mock_opencode_task(
    prompt: &str,
    session_id: Option<&str>,
    _timeout_secs: Option<u64>,
) -> Result<OpenCodeOutput, OpenCodeError> {
    // Simulate successful execution
    let result_session_id = session_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("sess_{}", Uuid::new_v4()));

    let stdout = format!(
        "Session: {}\nExecuted prompt: {}\nResult: Mock output",
        result_session_id, prompt
    );

    Ok(OpenCodeOutput {
        session_id: result_session_id,
        stdout,
        stderr: String::new(),
        success: true,
        timed_out: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert!(manager.get_session().is_none());
    }

    #[test]
    fn test_session_manager_create_session() {
        let mut manager = SessionManager::new();
        let session_id = manager.create_session();

        assert!(session_id.starts_with("sess_"));
        assert_eq!(manager.get_session(), Some(session_id.as_str()));
    }

    #[test]
    fn test_session_manager_set_session() {
        let mut manager = SessionManager::new();
        manager.set_session("test_session_123".to_string());

        assert_eq!(manager.get_session(), Some("test_session_123"));
    }

    #[test]
    fn test_session_manager_clear_session() {
        let mut manager = SessionManager::new();
        manager.create_session();
        manager.clear_session();

        assert!(manager.get_session().is_none());
    }

    #[test]
    fn test_session_manager_get_or_create() {
        let mut manager = SessionManager::new();

        // First call creates session
        let session1 = manager.get_or_create_session();
        assert!(session1.starts_with("sess_"));

        // Second call returns existing session
        let session2 = manager.get_or_create_session();
        assert_eq!(session1, session2);
    }

    #[test]
    fn test_parse_session_from_output() {
        let output = "Starting OpenCode...\nSession: sess_abc123\nProcessing...";
        let session_id = parse_session_from_output(output);
        assert_eq!(session_id, Some("sess_abc123".to_string()));
    }

    #[test]
    fn test_parse_session_from_output_lowercase() {
        let output = "session: sess_xyz789";
        let session_id = parse_session_from_output(output);
        assert_eq!(session_id, Some("sess_xyz789".to_string()));
    }

    #[test]
    fn test_parse_session_from_output_no_session() {
        let output = "Some output without session ID";
        let session_id = parse_session_from_output(output);
        assert!(session_id.is_none());
    }

    #[test]
    fn test_open_code_error_display() {
        let error = OpenCodeError::Timeout { timeout_secs: 60 };
        assert_eq!(
            error.to_string(),
            "OpenCode execution timed out after 60 seconds"
        );

        let error = OpenCodeError::InvalidSession {
            message: "session not found".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid session: session not found");
    }

    #[test]
    fn test_open_code_error_from_timeout_error() {
        let timeout_error = TimeoutError::Timeout { timeout_secs: 30 };
        let open_code_error: OpenCodeError = timeout_error.into();

        assert_eq!(
            open_code_error,
            OpenCodeError::Timeout { timeout_secs: 30 }
        );
    }

    #[tokio::test]
    async fn test_run_mock_opencode_task_new_session() {
        let output = run_mock_opencode_task("test prompt", None, Some(60))
            .await
            .unwrap();

        assert!(output.success);
        assert!(!output.timed_out);
        assert!(output.session_id.starts_with("sess_"));
        assert!(output.stdout.contains("test prompt"));
    }

    #[tokio::test]
    async fn test_run_mock_opencode_task_existing_session() {
        let output = run_mock_opencode_task(
            "test prompt",
            Some("sess_existing123"),
            Some(60),
        )
        .await
        .unwrap();

        assert!(output.success);
        assert_eq!(output.session_id, "sess_existing123");
    }

    #[test]
    fn test_open_code_config_default() {
        let config = OpenCodeConfig::default();
        assert_eq!(config.binary_path, "opencode");
        assert_eq!(config.default_timeout_secs, 300);
    }

    #[test]
    fn test_open_code_output_serialization() {
        let output = OpenCodeOutput {
            session_id: "sess_test".to_string(),
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            success: true,
            timed_out: false,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("sess_test"));
        assert!(json.contains("output"));

        let deserialized: OpenCodeOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "sess_test");
        assert_eq!(deserialized.stdout, "output");
    }

    // Integration test - only runs when opencode binary is available
    #[tokio::test]
    #[ignore = "requires opencode binary to be installed"]
    async fn test_run_opencode_task_integration() {
        // This test requires opencode to be installed
        let result = run_opencode_task(
            "echo hello",
            None,
            Some(30),
            None,
        )
        .await;

        // Should either succeed or fail with SpawnFailed (if opencode not installed)
        match result {
            Ok(output) => {
                assert!(output.success || output.timed_out);
            }
            Err(OpenCodeError::SpawnFailed { .. }) => {
                // Expected if opencode is not installed
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_open_code_output_success() {
        let output = OpenCodeOutput {
            session_id: "sess_test".to_string(),
            stdout: "output".to_string(),
            stderr: String::new(),
            success: true,
            timed_out: false,
        };

        assert!(output.success);
        assert!(!output.timed_out);
    }

    #[tokio::test]
    async fn test_open_code_output_timeout() {
        let output = OpenCodeOutput {
            session_id: "sess_test".to_string(),
            stdout: String::new(),
            stderr: String::new(),
            success: false,
            timed_out: true,
        };

        assert!(!output.success);
        assert!(output.timed_out);
    }

    // Test that we can reuse sessions
    #[tokio::test]
    async fn test_session_reuse() {
        let mut manager = SessionManager::new();

        // First execution creates session
        let session1 = manager.get_or_create_session();
        let output1 = run_mock_opencode_task("task 1", Some(&session1), Some(60))
            .await
            .unwrap();

        // Second execution reuses session
        let session2 = manager.get_or_create_session();
        assert_eq!(session1, session2);

        let output2 = run_mock_opencode_task("task 2", Some(&session2), Some(60))
            .await
            .unwrap();

        assert_eq!(output1.session_id, output2.session_id);
    }
}

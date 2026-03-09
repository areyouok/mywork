use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use super::process_tracker;

/// Timeout error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeoutError {
    /// Process execution timed out
    Timeout { timeout_secs: u64 },
    /// Failed to spawn process
    SpawnFailed { message: String },
    /// Failed to kill process
    KillFailed { pid: u32, message: String },
    /// Process execution failed
    ExecutionFailed { message: String },
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeoutError::Timeout { timeout_secs } => {
                write!(f, "Process execution timed out after {} seconds", timeout_secs)
            }
            TimeoutError::SpawnFailed { message } => {
                write!(f, "Failed to spawn process: {}", message)
            }
            TimeoutError::KillFailed { pid, message } => {
                write!(f, "Failed to kill process {}: {}", pid, message)
            }
            TimeoutError::ExecutionFailed { message } => {
                write!(f, "Process execution failed: {}", message)
            }
        }
    }
}

impl std::error::Error for TimeoutError {}

/// Output from a process execution
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessOutput {
    /// Exit status of the process
    pub status: ExitStatus,
    /// Standard output from the process
    pub stdout: String,
    /// Standard error from the process
    pub stderr: String,
    /// Whether the process timed out
    pub timed_out: bool,
}

impl ProcessOutput {
    /// Check if the process exited successfully
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Get the exit code if available
    pub fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

pub fn kill_process(pid: u32, kill_process_group: bool) -> Result<(), TimeoutError> {
    let target_pid = if kill_process_group {
        Pid::from_raw(-(pid as i32))
    } else {
        Pid::from_raw(pid as i32)
    };

    kill(target_pid, Signal::SIGKILL).map_err(|e| TimeoutError::KillFailed {
        pid,
        message: e.to_string(),
    })
}

/// Run a command with timeout control
///
/// # Arguments
/// * `program` - Program to execute
/// * `args` - Arguments for the program
/// * `timeout_secs` - Timeout in seconds
/// * `cwd` - Working directory for the process
///
/// # Returns
/// `Ok(ProcessOutput)` if the command completes (or times out), `Err` if spawn fails
pub async fn run_with_timeout(
    program: &str,
    args: &[&str],
    timeout_secs: u64,
    cwd: Option<&std::path::Path>,
) -> Result<ProcessOutput, TimeoutError> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .process_group(0);
    
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    
    let mut child = cmd
        .spawn()
        .map_err(|e| TimeoutError::SpawnFailed {
            message: e.to_string(),
        })?;
    
    // Get PID before moving child
    let pid = child.id().ok_or_else(|| TimeoutError::ExecutionFailed {
        message: "Failed to get process PID".to_string(),
    })?;
    
    process_tracker::register_pid(pid);
    
    let timeout_duration = Duration::from_secs(timeout_secs);
    let result = timeout(timeout_duration, async {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        
        let status = child.wait().await.map_err(|e| TimeoutError::ExecutionFailed {
            message: e.to_string(),
        })?;
        
        process_tracker::unregister_pid(pid);
        
        let stdout_str = if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            let mut lines = Vec::new();
            while let Some(line) = reader.next_line().await.map_err(|e| TimeoutError::ExecutionFailed {
                message: e.to_string(),
            })? {
                lines.push(line);
            }
            lines.join("\n")
        } else {
            String::new()
        };
        
        let stderr_str = if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            let mut lines = Vec::new();
            while let Some(line) = reader.next_line().await.map_err(|e| TimeoutError::ExecutionFailed {
                message: e.to_string(),
            })? {
                lines.push(line);
            }
            lines.join("\n")
        } else {
            String::new()
        };
        
        Ok::<ProcessOutput, TimeoutError>(ProcessOutput {
            status,
            stdout: stdout_str,
            stderr: stderr_str,
            timed_out: false,
        })
    })
    .await;
    
    match result {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => {
            process_tracker::unregister_pid(pid);
            Err(e)
        }
        Err(_) => {
            kill_process(pid, true)?;
            let _ = child.wait().await;
            process_tracker::unregister_pid(pid);
            
            Ok(ProcessOutput {
                status: std::process::ExitStatus::from_raw(137),
                stdout: String::new(),
                stderr: String::new(),
                timed_out: true,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[tokio::test]
    async fn test_run_with_timeout_success() {
        // Execute echo command successfully
        let output = run_with_timeout("echo", &["hello", "world"], 5, None).await.unwrap();
        
        assert!(output.success());
        assert!(!output.timed_out);
        assert!(output.stdout.contains("hello"));
        assert!(output.stdout.contains("world"));
        assert!(output.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_run_with_timeout_command_failure() {
        // Execute a command that will fail (exit with non-zero)
        let output = run_with_timeout("ls", &["/nonexistent_directory_12345"], 5, None).await.unwrap();
        
        assert!(!output.success());
        assert!(!output.timed_out);
        assert!(!output.stderr.is_empty() || !output.stdout.is_empty());
    }

    #[tokio::test]
    async fn test_run_with_timeout_short_running() {
        // A short-running command should complete before timeout
        let output = run_with_timeout("sleep", &["0.1"], 5, None).await.unwrap();
        
        assert!(output.success());
        assert!(!output.timed_out);
    }

    #[tokio::test]
    async fn test_run_with_timeout_times_out() {
        // A long-running command should timeout
        let start = std::time::Instant::now();
        let output = run_with_timeout("sleep", &["30"], 2, None).await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(!output.success());
        assert!(output.timed_out);
        assert!(elapsed < StdDuration::from_secs(5), "Should timeout within 5 seconds, took {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_run_with_timeout_captures_stdout() {
        // Test that stdout is captured correctly
        let output = run_with_timeout("printf", &["line1\\nline2\\nline3"], 5, None).await.unwrap();
        
        assert!(output.success());
        assert!(output.stdout.contains("line1"));
        assert!(output.stdout.contains("line2"));
        assert!(output.stdout.contains("line3"));
    }

    #[tokio::test]
    async fn test_run_with_timeout_captures_stderr() {
        // Test that stderr is captured correctly
        // Using bash to write to stderr
        let output = run_with_timeout("bash", &["-c", "echo 'error message' >&2"], 5, None).await.unwrap();
        
        assert!(output.success());
        assert!(output.stderr.contains("error message"));
    }

    #[tokio::test]
    async fn test_run_with_timeout_invalid_command() {
        // Invalid command should return error
        let result = run_with_timeout("nonexistent_command_12345", &[], 5, None).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TimeoutError::SpawnFailed { message } => {
                assert!(message.contains("nonexistent_command") || message.contains("No such file"));
            }
            _ => panic!("Expected SpawnFailed error"),
        }
    }

    #[tokio::test]
    async fn test_kill_process_success() {
        // Spawn a long-running process
        let mut child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("Failed to spawn sleep");
        
        let pid = child.id().expect("Failed to get PID");
        
        // Wait a bit for process to start
        thread::sleep(StdDuration::from_millis(100));
        
        let result = kill_process(pid, false);
        assert!(result.is_ok());
        
        let _ = child.wait().await;
        
        thread::sleep(StdDuration::from_millis(100));
        let _result2 = kill_process(pid, false);
        // This may or may not fail depending on whether PID has been recycled
        // Just verify first kill succeeded
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_kill_process_nonexistent() {
        // Try to kill a process that doesn't exist (use a very high PID)
        // This may or may not succeed depending on OS behavior
        let _result = kill_process(999999, false);
        // We don't assert the result because OS behavior varies
    }

    #[tokio::test]
    async fn test_process_output_success() {
        let output = ProcessOutput {
            status: std::process::ExitStatus::from_raw(0),
            stdout: "output".to_string(),
            stderr: String::new(),
            timed_out: false,
        };
        
        assert!(output.success());
        assert_eq!(output.code(), Some(0));
        assert!(!output.timed_out);
    }

    #[tokio::test]
    async fn test_process_output_failure() {
        let output = ProcessOutput {
            status: std::process::ExitStatus::from_raw(1),
            stdout: String::new(),
            stderr: "error".to_string(),
            timed_out: false,
        };
        
        assert!(!output.success());
        assert!(!output.timed_out);
    }

    #[tokio::test]
    async fn test_process_output_timeout() {
        let output = ProcessOutput {
            status: std::process::ExitStatus::from_raw(137),
            stdout: String::new(),
            stderr: String::new(),
            timed_out: true,
        };
        
        assert!(!output.success());
        assert!(output.timed_out);
    }

    #[tokio::test]
    async fn test_timeout_error_display() {
        let error = TimeoutError::Timeout { timeout_secs: 30 };
        assert_eq!(
            error.to_string(),
            "Process execution timed out after 30 seconds"
        );
        
        let error = TimeoutError::SpawnFailed {
            message: "command not found".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to spawn process: command not found"
        );
        
        let error = TimeoutError::KillFailed {
            pid: 1234,
            message: "no such process".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to kill process 1234: no such process"
        );
    }

    #[tokio::test]
    async fn test_timeout_zero_seconds() {
        // Even with 0 second timeout, process should timeout immediately
        let output = run_with_timeout("sleep", &["1"], 0, None).await.unwrap();
        
        assert!(output.timed_out);
        assert!(!output.success());
    }

    #[tokio::test]
    async fn test_run_with_timeout_concurrent_executions() {
        let task0 = run_with_timeout("echo", &["task-0"], 5, None);
        let task1 = run_with_timeout("echo", &["task-1"], 5, None);
        let task2 = run_with_timeout("echo", &["task-2"], 5, None);
        
        let (result0, result1, result2) = tokio::join!(task0, task1, task2);
        
        let output0 = result0.unwrap();
        let output1 = result1.unwrap();
        let output2 = result2.unwrap();
        
        assert!(output0.success());
        assert!(output0.stdout.contains("task-0"));
        
        assert!(output1.success());
        assert!(output1.stdout.contains("task-1"));
        
        assert!(output2.success());
        assert!(output2.stdout.contains("task-2"));
    }
}

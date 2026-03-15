use std::path::Path;
use std::sync::Arc;

use crate::executor::streaming_executor::{StreamLine, StreamingExecutor};
use crate::models::task;
use crate::opencode::executor::resolve_opencode_binary_path;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

#[cfg(target_os = "macos")]
fn is_system_sleeping() -> bool {
    crate::power_monitor::is_sleeping() || crate::power_monitor::is_clamshell_closed()
}

#[cfg(not(target_os = "macos"))]
fn is_system_sleeping() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputEvent {
    Output { text: String, source: String },
    Finished { exit_code: i32 },
}

#[tauri::command]
pub async fn execute_task_streaming(
    task_id: String,
    _prompt: String,
    cwd: Option<String>,
    channel: Channel<OutputEvent>,
    pool: State<'_, Arc<SqlitePool>>,
    _app: AppHandle,
) -> Result<(), String> {
    if is_system_sleeping() {
        return Err(
            "System is unavailable (sleeping or lid closed); streaming execution is paused"
                .to_string(),
        );
    }

    let task = task::get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Failed to get task: {}", e))?;

    let args: Vec<&str> = vec!["run", &task.prompt];
    let cwd_path = cwd.as_deref().map(Path::new);
    let opencode_binary = resolve_opencode_binary_path()
        .map_err(|e| format!("Failed to locate opencode binary: {}", e))?;

    if is_system_sleeping() {
        return Err(
            "System is unavailable (sleeping or lid closed); streaming execution is paused"
                .to_string(),
        );
    }

    let mut executor = StreamingExecutor::spawn(&opencode_binary, &args, cwd_path)
        .await
        .map_err(|e| format!("Failed to start opencode streaming: {}", e))?;

    let _ = stream_executor_to_events(&mut executor, |event| {
        channel.send(event).map_err(|e| e.to_string())
    })
    .await?;

    Ok(())
}

async fn stream_executor_to_events<S>(
    executor: &mut StreamingExecutor,
    mut send: S,
) -> Result<i32, String>
where
    S: FnMut(OutputEvent) -> Result<(), String>,
{
    while let Some(line) = executor.read_line().await {
        match line {
            StreamLine::Stdout(text) => {
                send(OutputEvent::Output {
                    text,
                    source: "stdout".to_string(),
                })?;
            }
            StreamLine::Stderr(text) => {
                send(OutputEvent::Output {
                    text,
                    source: "stderr".to_string(),
                })?;
            }
            StreamLine::Finished => {
                let exit_code = executor.exit_code().await.unwrap_or(-1);
                send(OutputEvent::Finished { exit_code })?;
                return Ok(exit_code);
            }
        }
    }

    let exit_code = executor.exit_code().await.unwrap_or(-1);
    send(OutputEvent::Finished { exit_code })?;
    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_system_sleeping_tracks_power_monitor_state() {
        crate::power_monitor::with_test_power_state_lock(|| {
            crate::power_monitor::set_sleeping(true);
            assert!(is_system_sleeping());

            crate::power_monitor::set_sleeping(false);
            assert!(!is_system_sleeping());
        });
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_is_system_sleeping_is_false_off_macos() {
        assert!(!is_system_sleeping());
    }

    #[tokio::test]
    async fn test_streaming_command_emits_stdout_stderr_and_finished() {
        let mut executor = StreamingExecutor::spawn(
            "bash",
            &["-c", "echo stdout-line; echo stderr-line >&2"],
            None,
        )
        .await
        .expect("failed to spawn executor");

        let mut events = Vec::new();
        let mut merged = String::new();
        let exit_code = stream_executor_to_events(&mut executor, |event| {
            if let OutputEvent::Output { text, source } = &event {
                let _ = source;
                merged.push_str(text);
                merged.push('\n');
            }
            events.push(event);
            Ok(())
        })
        .await
        .expect("failed to stream events");

        assert!(merged.contains("stdout-line"));
        assert!(merged.contains("stderr-line"));
        assert_eq!(exit_code, 0);

        assert!(events.iter().any(|event| {
            matches!(event, OutputEvent::Output { text, source } if source == "stdout" && text.contains("stdout-line"))
        }));
        assert!(events.iter().any(|event| {
            matches!(event, OutputEvent::Output { text, source } if source == "stderr" && text.contains("stderr-line"))
        }));
        assert!(matches!(
            events.last(),
            Some(OutputEvent::Finished { exit_code: 0 })
        ));
    }
}

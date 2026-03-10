use std::path::Path;

use crate::executor::streaming_executor::{StreamLine, StreamingExecutor};
use crate::storage::output;
use serde::Serialize;
use tauri::AppHandle;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputEvent {
    Stdout { text: String },
    Stderr { text: String },
    Finished { exit_code: i32 },
}

#[tauri::command]
pub async fn execute_task_streaming(
    task_id: String,
    prompt: String,
    cwd: Option<String>,
    channel: Channel<OutputEvent>,
    app: AppHandle,
) -> Result<(), String> {
    let args = ["--print-session-id", "--non-interactive", "-p", &prompt];
    let cwd_path = cwd.as_deref().map(Path::new);

    let mut executor = StreamingExecutor::spawn("opencode", &args, cwd_path)
        .await
        .map_err(|e| format!("Failed to start opencode streaming: {}", e))?;

    let (stdout, stderr, _) = stream_executor_to_events(&mut executor, |event| {
        channel.send(event).map_err(|e| e.to_string())
    })
    .await?;

    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let content = format!("=== STDOUT ===\n{}\n\n=== STDERR ===\n{}", stdout, stderr);
    output::write_output_file(&output_dir, &task_id, &content)
        .await
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

async fn stream_executor_to_events<S>(
    executor: &mut StreamingExecutor,
    mut send: S,
) -> Result<(String, String, i32), String>
where
    S: FnMut(OutputEvent) -> Result<(), String>,
{
    let mut stdout = String::new();
    let mut stderr = String::new();

    while let Some(line) = executor.read_line().await {
        match line {
            StreamLine::Stdout(text) => {
                stdout.push_str(&text);
                stdout.push('\n');
                send(OutputEvent::Stdout { text })?;
            }
            StreamLine::Stderr(text) => {
                stderr.push_str(&text);
                stderr.push('\n');
                send(OutputEvent::Stderr { text })?;
            }
            StreamLine::Finished => {
                let exit_code = executor.exit_code().await.unwrap_or(-1);
                send(OutputEvent::Finished { exit_code })?;
                return Ok((stdout, stderr, exit_code));
            }
        }
    }

    let exit_code = executor.exit_code().await.unwrap_or(-1);
    send(OutputEvent::Finished { exit_code })?;
    Ok((stdout, stderr, exit_code))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let (stdout, stderr, exit_code) = stream_executor_to_events(&mut executor, |event| {
            events.push(event);
            Ok(())
        })
        .await
        .expect("failed to stream events");

        assert!(stdout.contains("stdout-line"));
        assert!(stderr.contains("stderr-line"));
        assert_eq!(exit_code, 0);

        assert!(events.iter().any(|event| {
            matches!(event, OutputEvent::Stdout { text } if text.contains("stdout-line"))
        }));
        assert!(events.iter().any(|event| {
            matches!(event, OutputEvent::Stderr { text } if text.contains("stderr-line"))
        }));
        assert!(matches!(
            events.last(),
            Some(OutputEvent::Finished { exit_code: 0 })
        ));
    }
}

use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};

const MAX_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamLine {
    Stdout(String),
    Stderr(String),
    Finished,
}

#[derive(Debug)]
struct BufferState {
    lines: VecDeque<(usize, StreamLine)>,
    total_bytes: usize,
    next_id: usize,
}

impl BufferState {
    fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            total_bytes: 0,
            next_id: 0,
        }
    }

    fn push(&mut self, line: StreamLine) {
        let size = line_size(&line);
        let id = self.next_id;
        self.next_id += 1;
        self.lines.push_back((id, line));
        self.total_bytes += size;

        while self.total_bytes > MAX_BUFFER_BYTES {
            if let Some((_old_id, old_line)) = self.lines.pop_front() {
                self.total_bytes = self.total_bytes.saturating_sub(line_size(&old_line));
            } else {
                break;
            }
        }
    }

    fn clear_through(&mut self, id: usize) {
        loop {
            let should_remove = matches!(self.lines.front(), Some((front_id, _)) if *front_id <= id);
            if !should_remove {
                break;
            }

            if let Some((_old_id, old_line)) = self.lines.pop_front() {
                self.total_bytes = self.total_bytes.saturating_sub(line_size(&old_line));
            }
        }
    }
}

fn line_size(line: &StreamLine) -> usize {
    match line {
        StreamLine::Stdout(s) | StreamLine::Stderr(s) => s.len() + 1,
        StreamLine::Finished => 1,
    }
}

#[derive(Debug)]
pub struct StreamingExecutor {
    child: Arc<Mutex<Child>>,
    receiver: mpsc::Receiver<(usize, StreamLine)>,
    buffer: Arc<Mutex<BufferState>>,
    exit_code: Arc<Mutex<Option<i32>>>,
    last_read_id: Option<usize>,
}

impl StreamingExecutor {
    pub async fn spawn(
        program: &str,
        args: &[&str],
        cwd: Option<&Path>,
    ) -> std::io::Result<Self> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let child = Arc::new(Mutex::new(child));
        let buffer = Arc::new(Mutex::new(BufferState::new()));
        let exit_code = Arc::new(Mutex::new(None));
        let (tx, rx) = mpsc::channel(1024);

        let stdout_handle = if let Some(stdout_pipe) = stdout {
            let tx_stdout = tx.clone();
            let buffer_stdout = Arc::clone(&buffer);
            Some(tokio::spawn(async move {
                let mut lines = BufReader::new(stdout_pipe).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let stream_line = StreamLine::Stdout(line);
                    let id = {
                        let mut guard = buffer_stdout.lock().await;
                        let id = guard.next_id;
                        guard.push(stream_line.clone());
                        id
                    };
                    if tx_stdout.send((id, stream_line)).await.is_err() {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        let stderr_handle = if let Some(stderr_pipe) = stderr {
            let tx_stderr = tx.clone();
            let buffer_stderr = Arc::clone(&buffer);
            Some(tokio::spawn(async move {
                let mut lines = BufReader::new(stderr_pipe).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let stream_line = StreamLine::Stderr(line);
                    let id = {
                        let mut guard = buffer_stderr.lock().await;
                        let id = guard.next_id;
                        guard.push(stream_line.clone());
                        id
                    };
                    if tx_stderr.send((id, stream_line)).await.is_err() {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        let tx_finished = tx.clone();
        let child_wait = Arc::clone(&child);
        let buffer_finished = Arc::clone(&buffer);
        let exit_code_finished = Arc::clone(&exit_code);
        tokio::spawn(async move {
            let wait_result = child_wait.lock().await.wait().await;
            let status_code = wait_result.ok().and_then(|status| status.code()).unwrap_or(-1);
            *exit_code_finished.lock().await = Some(status_code);
            if let Some(handle) = stdout_handle {
                let _ = handle.await;
            }
            if let Some(handle) = stderr_handle {
                let _ = handle.await;
            }
            let id = {
                let mut guard = buffer_finished.lock().await;
                let id = guard.next_id;
                guard.push(StreamLine::Finished);
                id
            };
            let _ = tx_finished.send((id, StreamLine::Finished)).await;
        });

        Ok(Self {
            child,
            receiver: rx,
            buffer,
            exit_code,
            last_read_id: None,
        })
    }

    pub async fn read_line(&mut self) -> Option<StreamLine> {
        match self.receiver.recv().await {
            Some((id, line)) => {
                self.last_read_id = Some(id);
                let mut guard = self.buffer.lock().await;
                guard.clear_through(id);
                Some(line)
            }
            None => None,
        }
    }

    pub async fn is_running(&self) -> bool {
        let mut child = self.child.lock().await;
        match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }

    pub async fn buffer_size(&self) -> usize {
        self.buffer.lock().await.total_bytes
    }

    pub async fn exit_code(&self) -> Option<i32> {
        *self.exit_code.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_read_line() {
        let mut executor = StreamingExecutor::spawn(
            "bash",
            &["-c", "echo line-1; echo line-2"],
            None,
        )
        .await
        .expect("failed to spawn executor");

        let mut outputs = Vec::new();
        while let Some(line) = timeout(Duration::from_secs(2), executor.read_line())
            .await
            .expect("timed out waiting line")
        {
            eprintln!("test_read_line received: {:?}", line);
            let is_finished = matches!(line, StreamLine::Finished);
            outputs.push(line);
            if is_finished {
                break;
            }
        }

        let stdout_lines: Vec<String> = outputs
            .iter()
            .filter_map(|line| match line {
                StreamLine::Stdout(s) => Some(s.trim().to_string()),
                _ => None,
            })
            .collect();

        assert!(stdout_lines.iter().any(|s| s.contains("line-1")));
        assert!(stdout_lines.iter().any(|s| s.contains("line-2")));
        assert!(outputs.iter().any(|line| matches!(line, StreamLine::Finished)));
    }

    #[tokio::test]
    async fn test_stdout_stderr_separated() {
        let mut executor = StreamingExecutor::spawn(
            "bash",
            &["-c", "echo out-line; echo err-line >&2"],
            None,
        )
        .await
        .expect("failed to spawn executor");

        let mut saw_stdout = false;
        let mut saw_stderr = false;

        while let Some(line) = timeout(Duration::from_secs(2), executor.read_line())
            .await
            .expect("timed out waiting line")
        {
            eprintln!("test_stdout_stderr_separated received: {:?}", line);
            match line {
                StreamLine::Stdout(text) => {
                    if text.trim().contains("out-line") {
                        saw_stdout = true;
                    }
                }
                StreamLine::Stderr(text) => {
                    if text.trim().contains("err-line") {
                        saw_stderr = true;
                    }
                }
                StreamLine::Finished => break,
            }
        }

        assert!(saw_stdout);
        assert!(saw_stderr);
    }

    #[tokio::test]
    async fn test_backpressure() {
        let mut executor = StreamingExecutor::spawn(
            "bash",
            &[
                "-c",
                "python3 -c \"import sys; line='x'*512; [sys.stdout.write(line+'\\n') for _ in range(4096)]\"",
            ],
            None,
        )
        .await
        .expect("failed to spawn executor");

        let mut saw_output = false;
        while let Some(line) = timeout(Duration::from_secs(5), executor.read_line())
            .await
            .expect("timed out waiting line")
        {
            match line {
                StreamLine::Stdout(_) | StreamLine::Stderr(_) => {
                    saw_output = true;
                }
                StreamLine::Finished => break,
            }
        }

        assert!(saw_output);
        let size = executor.buffer_size().await;
        assert!(size <= MAX_BUFFER_BYTES, "buffer size exceeded limit: {size}");
    }

    #[tokio::test]
    async fn test_process_finished_signal() {
        let mut executor = StreamingExecutor::spawn("bash", &["-c", "echo done"], None)
            .await
            .expect("failed to spawn executor");

        let mut saw_finished = false;
        while let Some(line) = timeout(Duration::from_secs(2), executor.read_line())
            .await
            .expect("timed out waiting line")
        {
            if matches!(line, StreamLine::Finished) {
                saw_finished = true;
                break;
            }
        }

        assert!(saw_finished);
        assert!(!executor.is_running().await);
    }
}

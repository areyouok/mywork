# MyWork Backend Module

**Type:** Tauri 2 + Rust + SQLite

## OVERVIEW

Core backend: task scheduling, execution, persistence, OpenCode integration, and real-time streaming output.

## STRUCTURE

```
src-tauri/src/
├── commands/      # Tauri IPC handlers (7 files)
├── models/        # Data structs (Task, Execution)
├── scheduler/     # Job scheduling (cron + intervals + concurrency)
├── executor/      # Streaming process executor
├── task_executor/ # Task execution logic
├── opencode/      # OpenCode CLI executor + session parsing
├── db/            # SQLite connection pool + schema
├── storage/       # File I/O (output files)
├── lib.rs         # Tauri app setup + command registration
└── main.rs        # Entry point (calls lib.rs)
```

## WHERE TO LOOK

| Task              | Location                           | Notes                               |
| ----------------- | ---------------------------------- | ----------------------------------- |
| Task CRUD         | `commands/task_commands.rs`        | Create/update/delete tasks          |
| Execution queries | `commands/execution_commands.rs`   | History retrieval                   |
| Scheduler control | `commands/scheduler_commands.rs`   | Start/stop/reload + exec internal   |
| Manual execution  | `commands/task_runner_commands.rs` | run_task with streaming output      |
| Streaming exec    | `commands/streaming_commands.rs`   | Real-time output via IPC channels   |
| Output files      | `commands/output_commands.rs`      | Read/delete execution outputs       |
| Cron parsing      | `scheduler/cron_parser.rs`         | Parse cron expressions              |
| Simple schedules  | `scheduler/simple_schedule.rs`     | Interval parsing                    |
| Job scheduling    | `scheduler/job_scheduler.rs`       | tokio-cron-scheduler wrapper        |
| Concurrency       | `scheduler/task_queue.rs`          | Default: skip if running            |
| Timeout handling  | `scheduler/timeout.rs`             | Kill long-running tasks             |
| Process tracking  | `scheduler/process_tracker.rs`     | PID tracking, orphan cleanup        |
| Streaming I/O     | `executor/streaming_executor.rs`   | Async stdout/stderr streaming       |
| Task execution    | `task_executor/execute_task.rs`    | execute_task(), TaskExecutionResult |
| CLI execution     | `opencode/executor.rs`             | Run OpenCode commands               |
| Session parsing   | `opencode/session_parser.rs`       | Extract session ID from output      |
| DB connection     | `db/connection.rs`                 | SQLite pool setup                   |

## CONVENTIONS

- **Commands**: One file per domain, exported via `mod.rs`
- **Models**: Serde-serializable structs with UUID or timestamp-based IDs
- **Async**: All I/O is async via tokio
- **Errors**: Use `anyhow::Result` for command handlers
- **Streaming**: Use `StreamingExecutor` for real-time output via Tauri IPC channels

## ANTI-PATTERNS

- No blocking I/O in async context
- No panics in command handlers (return Result)

## NOTES

- Entry: `main.rs` → `mywork_lib::run()`
- Registration: Commands registered in `lib.rs` via `invoke_handler`
- Scheduler: tokio-cron-scheduler with PostgresJob-style cron
- Concurrency: Default behavior - skip execution if previous run still in progress
- Process management: Uses `nix` crate for signal handling
- Execution ID format: `{task_id}_{YYYYMMDD_HHMMSS_mmm}` (human-readable)
- Output files: `{execution_id}.txt` in app support directory

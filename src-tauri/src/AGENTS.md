# MyWork Backend Module

**Type:** Tauri 2 + Rust + SQLite

## OVERVIEW

Core backend: task scheduling, execution, persistence, and OpenCode integration.

## STRUCTURE

```
src-tauri/src/
├── commands/      # Tauri IPC handlers
├── models/        # Data structs (Task, Execution)
├── scheduler/     # Job scheduling (cron + intervals)
├── db/            # SQLite connection pool
├── storage/       # File I/O (output files)
├── opencode/      # OpenCode CLI executor
├── lib.rs         # Tauri app setup + command registration
└── main.rs        # Entry point (calls lib.rs)
```

## WHERE TO LOOK

| Task              | Location                         | Notes                        |
| ----------------- | -------------------------------- | ---------------------------- |
| Task CRUD         | `commands/task_commands.rs`      | Create/update/delete tasks   |
| Execution queries | `commands/execution_commands.rs` | History retrieval            |
| Scheduler control | `commands/scheduler_commands.rs` | Start/stop/reload            |
| Output files      | `commands/output_commands.rs`    | Read execution outputs       |
| Cron parsing      | `scheduler/cron_parser.rs`       | Parse cron expressions       |
| Simple schedules  | `scheduler/simple_schedule.rs`   | Interval parsing             |
| Job scheduling    | `scheduler/job_scheduler.rs`     | tokio-cron-scheduler wrapper |
| Concurrency       | `scheduler/task_queue.rs`        | Skip-if-running logic        |
| Timeout handling  | `scheduler/timeout.rs`           | Kill long-running tasks      |
| CLI execution     | `opencode/executor.rs`           | Run OpenCode commands        |
| DB connection     | `db/connection.rs`               | SQLite pool setup            |

## CONVENTIONS

- **Commands**: One file per domain, exported via `mod.rs`
- **Models**: Serde-serializable structs with UUID primary keys
- **Async**: All I/O is async via tokio
- **Errors**: Use `anyhow::Result` for command handlers

## ANTI-PATTERNS

- No blocking I/O in async context
- No panics in command handlers (return Result)

## NOTES

- Entry: `main.rs` → `mywork_lib::run()`
- Registration: Commands registered in `lib.rs` via `invoke_handler`
- Scheduler: tokio-cron-scheduler with PostgresJob-style cron
- Process management: Uses `nix` crate for signal handling

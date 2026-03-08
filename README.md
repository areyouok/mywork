# MyWork Scheduler

A Tauri-based desktop application for scheduling and managing AI task executions.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Database Schema

The application uses SQLite for data persistence. The schema is defined in `src-tauri/src/db/schema.sql`.

### Tables

**tasks** - Scheduled task configurations
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key (UUID) |
| name | TEXT | Task name |
| prompt | TEXT | AI prompt/task description |
| cron_expression | TEXT | Cron expression for scheduling (optional) |
| simple_schedule | TEXT | JSON for simple interval scheduling |
| enabled | INTEGER | Whether task is active (1=yes, 0=no) |
| timeout_seconds | INTEGER | Execution timeout in seconds (default: 300) |
| skip_if_running | INTEGER | Skip if previous execution is still running |
| created_at | TEXT | Creation timestamp (ISO 8601) |
| updated_at | TEXT | Last update timestamp (ISO 8601) |

**executions** - Task execution history
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key (UUID) |
| task_id | TEXT | Foreign key to tasks |
| session_id | TEXT | OpenCode session ID |
| status | TEXT | Execution status: pending, running, success, failed, timeout, skipped |
| started_at | TEXT | Start timestamp (ISO 8601) |
| finished_at | TEXT | End timestamp (ISO 8601) |
| output_file | TEXT | Path to output file |
| error_message | TEXT | Error message if failed |

### Indexes
- `idx_executions_task_id` - For querying executions by task
- `idx_executions_started_at` - For querying executions by time

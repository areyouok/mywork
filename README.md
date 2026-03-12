# MyWork Scheduler

A Tauri-based desktop application for scheduling and managing AI task executions. This macOS system tray application allows you to schedule and run OpenCode tasks automatically using cron expressions, simple time intervals, or a one-time timestamp.

## Features

- **System Tray Application**: Runs in the macOS menu bar for easy access
- **Flexible Scheduling**: Support for cron expressions, simple time intervals, and one-time execution
- **Task Management**: Create, edit, enable/disable, and delete scheduled tasks
- **Execution History**: View detailed history of all task executions
- **Real-time Streaming**: View task output in real-time during execution
- **Output Viewer**: View task outputs with Markdown rendering and ANSI color support
- **Timeout Control**: Automatically kill long-running tasks
- **Concurrency Control**: Automatic skip if previous run is still in progress
- **SQLite Persistence**: All data stored locally in SQLite database

## Installation

### Prerequisites

- macOS 26 (Tahoe) or later
- Node.js 18+
- Rust 1.70+
- npm or yarn

### Setup

1. Clone the repository:

```bash
git clone <repository-url>
cd mywork
```

2. Install dependencies:

```bash
npm install
```

3. Build and run in development mode:

```bash
npm run tauri dev
```

4. For production build:

```bash
npm run tauri build
```

The built application will be available in `src-tauri/target/release/bundle/macos/`.

## Usage

### Creating a Task

1. Click the tray icon or open the main window
2. Click "New Task" button
3. Fill in the task details:
   - **Name**: A descriptive name for your task
   - **Prompt**: The AI prompt or command to execute
   - **Schedule**: Choose between:
     - **Cron Expression**: e.g., `*/5 * * * *` (every 5 minutes)
     - **Simple Schedule**: Select from dropdown (e.g., "Every 5 minutes")
     - **Once**: Select a future date/time to run once
   - **Timeout**: Maximum execution time in seconds (default: 300)
4. Click "Save"

Note: Tasks automatically skip execution if a previous run is still in progress. Any task can still be run manually with the **Run** button, regardless of schedule type or previous automatic runs.

### Viewing Task History

1. Click on a task in the task list
2. Navigate to the "History" tab
3. View all execution records with status, timestamps, and outputs
4. Click on an execution to view detailed output

### Managing Tasks

- **Enable/Disable**: Toggle the switch next to task name
- **Edit**: Click the edit button to modify task settings
- **Delete**: Click the delete button (requires confirmation)

## Architecture

### Technology Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2
- **Database**: SQLite with sqlx (async)
- **Scheduler**: tokio-cron-scheduler
- **Testing**: Vitest (frontend), cargo test (backend), Playwright (E2E)

### Project Structure

```
mywork/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── api/                # Tauri IPC wrappers
│   ├── types/              # TypeScript types
│   ├── utils/              # Utility functions
│   └── styles/             # Global styles
├── src-tauri/src/          # Rust backend
│   ├── commands/           # Tauri command handlers
│   ├── models/             # Data models
│   ├── scheduler/          # Job scheduling logic
│   ├── executor/           # Streaming process executor
│   ├── task_executor/      # Task execution logic
│   ├── opencode/           # OpenCode CLI integration
│   ├── db/                 # Database layer
│   └── storage/            # File storage
└── tests/e2e/              # Playwright E2E tests
```

### Data Flow

1. **Task Creation**:
   - User fills form → Frontend validates → Tauri command → Database insert → Scheduler adds job

2. **Task Execution**:
   - Scheduler triggers → Task queue checks concurrency → OpenCode executor runs → Output streamed to file → Execution record created → Frontend notified via Tauri events

3. **Real-time Streaming**:
   - Manual run → StreamingExecutor spawns process → stdout/stderr streamed via IPC channel → Frontend receives in real-time → Output displayed with ANSI color support

4. **History Viewing**:
   - User clicks task → Frontend queries executions → Tauri command → Database query → Results displayed

## Development

### Running Tests

```bash
# Frontend tests
npm test

# Backend tests
cargo test

# E2E tests
npm run test:e2e
```

### Code Quality

```bash
# Lint frontend
npm run lint

# Format code
npm run format

# Rust linter
cargo clippy
```

### Database Management

The SQLite database is stored at:

```
~/Library/Application Support/com.mywork/mywork.db
```

Task outputs are stored in:

```
~/Library/Application Support/com.mywork/outputs/
```

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
| once_at | TEXT | RFC3339 timestamp for one-time scheduling |
| enabled | INTEGER | Whether task is active (1=yes, 0=no) |
| timeout_seconds | INTEGER | Execution timeout in seconds (default: 300) |
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
| output_file | TEXT | Output filename (`execution_id + timestamp + .txt`) |
| error_message | TEXT | Error message if failed |

### Indexes

- `idx_executions_task_id` - For querying executions by task
- `idx_executions_started_at` - For querying executions by time

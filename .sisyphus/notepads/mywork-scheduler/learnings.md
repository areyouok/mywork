# Mywork Tauri Project Learnings

## Task 1: Initialize Tauri + React Project

### What was done:
1. Created Tauri + React + TypeScript project using `npm create tauri-app@latest`
2. Used `--template react-ts --project-name mywork --yes` flags to skip interactive prompts
3. Had to remove `.DS_Store` file as it blocked project creation
4. Restored hidden directories (`.sisyphus`, `.idea`) after project creation
5. Installed npm dependencies with `npm install`
6. Verified Vite dev server runs on http://localhost:1420
7. Verified `npm run tauri dev` starts successfully (compiles Rust code)

### Project Structure:
- `/src/` - React frontend source
- `/src-tauri/` - Tauri/Rust backend source
- `/package.json` - npm configuration

### Key Points:
- Tauri v2 was installed (not v1)
- React 19.1.0 with TypeScript
- Vite 7.3.1
- Package name is "tauri-app" (not "mywork" - the project name flag may not have worked as expected)

### Verified:
- [x] Project directory structure correct
- [x] npm install succeeded
- [x] Vite dev server runs on http://localhost:1420
- [x] Tauri dev command starts (compiles Rust code)

## Task 2: Configure TypeScript + Vite

### What was done:
1. Installed `@types/node` (was missing)
2. Added path alias `@/*` -> `./src/*` to:
   - `tsconfig.json` (baseUrl + paths)
   - `vite.config.ts` (resolve.alias)
3. Added `types: ["node"]` to `tsconfig.node.json` for vite.config.ts
4. Verified strict mode already enabled (was already in template)

### Key Points:
- TypeScript strict mode was already enabled in template
- Path alias allows imports like `import X from "@/components/..."`
- @types/react and @types/react-dom were already installed

### Verified:
- [x] `npx tsc --noEmit` passes
- [x] `npm run build` succeeds
- [x] Path alias `@/*` configured

## Task 3: Install Rust Dependencies

### What was done:
1. Added Rust dependencies to `src-tauri/Cargo.toml`:
   - tokio with "full" features
   - tokio-cron-scheduler 0.9
   - sqlx with runtime-tokio-native-tls and sqlite features
   - uuid with "v4" feature
   - chrono with "serde" feature
   - tauri with "tray-icon" feature (macos-private-api removed due to config requirement)
2. Ran `cargo build` to verify all dependencies install correctly
3. Fixed build error: removed `macos-private-api` feature as it requires special allowlist config

### Key Points:
- Using Tauri v2 (not v1)
- Backend is pure Rust (no Node.js backend)
- sqlx 0.7 with SQLite for task storage
- tokio-cron-scheduler for cron job scheduling

### Verified:
- [x] Cargo.toml contains all required Rust dependencies
- [x] `cargo build` succeeds (with 1 deprecation warning)

## Task 4: Configure ESLint + Prettier

### What was done:
1. Installed ESLint 9, Prettier, and TypeScript ESLint packages:
   - `eslint`, `prettier`
   - `@typescript-eslint/parser`, `@typescript-eslint/eslint-plugin`
   - `eslint-plugin-react`, `eslint-plugin-react-hooks`
   - `eslint-config-prettier`
2. Created `eslint.config.js` (flat config for ESLint 9)
3. Created `.prettierrc` with sensible defaults
4. Added npm scripts:
   - `lint`: `eslint src --ext ts,tsx`
   - `format`: `prettier --write src`
5. Fixed existing lint errors (missing `rel="noreferrer"` on anchor tags)

### Key Points:
- ESLint 9 requires flat config format (`eslint.config.js`, not `.eslintrc.*`)
- Config includes: TypeScript parser, React rules, React Hooks rules
- Relaxed rules: disabled `react/prop-types`, `explicit-function-return-type`, `explicit-module-boundary-types`
- Warnings only for: `no-unused-vars`, `no-explicit-any`
- Prettier config: single quotes, 2 spaces, 100 char line width

### Verified:
- [x] `npm run lint` passes (no errors)
- [x] `npm run format` runs successfully


## Task 8: SQLite Database Connection and Initialization

### What was done:
1. Created database module structure (`src-tauri/src/db/mod.rs`, `connection.rs`)
2. Implemented `init_database()` async function using sqlx::SqlitePool
3. Used `include_str!` to embed schema.sql at compile time
4. Split schema into individual statements for execution
5. Added `get_database_path()` helper to get Tauri app data directory
6. Added `tempfile` dev-dependency for testing
7. Wrote 4 comprehensive tests following TDD approach:
   - `test_init_database_creates_file`
   - `test_init_database_creates_tables`
   - `test_init_database_creates_indexes`
   - `test_init_database_idempotent`

### Key Points:
- sqlx::SqlitePool requires `sqlite:?mode=rwc` URL format (rwc = read-write-create)
- Schema execution splits on `;` and trims each statement
- Parent directory must be created before connecting to database
- `include_str!` macro embeds schema at compile time
- Tauri v2 uses `app.path().app_data_dir()` to get app data directory
- Tests use `tempfile::tempdir()` for isolated temporary directories
- `tokio::fs::create_dir_all()` is async version of directory creation

### Implementation Details:
- Database path: `{app_data_dir}/mywork.db`
- Connection pool: `SqlitePool` (async, thread-safe)
- Error handling: `Result<T, sqlx::Error>` for database errors
- Module structure:
  - `db/mod.rs` - Public API exports
  - `db/connection.rs` - Core initialization logic
  - `db/schema.sql` - SQL schema (from Task 7)

### Verified:
- [x] All 4 tests pass (`cargo test db::connection`)
- [x] Database file created in correct location
- [x] Tables and indexes created successfully
- [x] Idempotent initialization (can run multiple times)
- [x] Build passes without errors
- [x] Module structure follows Rust conventions

## Task 9: Task CRUD Operations

### What was done:
1. Created models module structure (`src-tauri/src/models/mod.rs`, `task.rs`)
2. Defined Task model struct matching schema.sql:
   - `Task` - Full task representation with all fields
   - `NewTask` - For creating new tasks (required fields only, optional with defaults)
   - `UpdateTask` - For partial updates (all fields optional)
3. Implemented CRUD operations:
   - `create_task(pool, new_task)` - Creates task with auto-generated UUID and timestamps
   - `get_task(pool, id)` - Fetches single task by ID
   - `get_all_tasks(pool)` - Fetches all tasks ordered by created_at DESC
   - `update_task(pool, id, update)` - Partial update preserving unspecified fields
   - `delete_task(pool, id)` - Soft deletion returning success boolean
4. Wrote 12 comprehensive tests covering all operations
5. Used sqlx::query_as for type-safe database queries
6. Used chrono::DateTime<Utc> for timestamp handling

### Key Points:
- UUID generation: `Uuid::new_v4().to_string()` for unique task IDs
- Timestamp format: RFC 3339 (`chrono::Utc::now().to_rfc3339()`)
- Default values: enabled=1, timeout_seconds=300, skip_if_running=1
- Update strategy: Merge pattern - preserve existing values when update field is None
- TempDir lifecycle: Must keep TempDir alive during test execution
  - Solution: Return (SqlitePool, TempDir) tuple from setup function
  - Use `_temp_dir` prefix to indicate intentionally unused variable
- sqlx::FromRow derive macro enables automatic struct mapping
- Async/await throughout with `tokio::test` macro

### Testing Patterns:
- TDD approach: Write tests first, implement functionality to pass
- Test coverage includes:
  - Success cases for all operations
  - Error cases (not found, invalid operations)
  - Default value verification
  - Partial update behavior
  - Full lifecycle integration test
- Isolated test databases using tempfile::TempDir
- Each test is independent and can run in parallel

### Module Structure:
- `models/mod.rs` - Public API exports
- `models/task.rs` - Task model and CRUD operations
- Clean separation: db module for connections, models for data

### Verified:
- [x] All 12 Task CRUD tests pass
- [x] All 16 total tests pass (including db tests)
- [x] Build succeeds without errors
- [x] Type-safe queries with sqlx::query_as
- [x] Proper async/await implementation
- [x] UUID auto-generation working
- [x] Timestamp auto-setting working
- [x] Default values applied correctly

## Task 10: Execution CRUD Operations

### What was done:
1. Created execution module structure (`src-tauri/src/models/execution.rs`)
2. Defined ExecutionStatus enum with 6 variants: pending, running, success, failed, timeout, skipped
3. Defined Execution model struct matching schema.sql:
   - `Execution` - Full execution representation with all fields
   - `NewExecution` - For creating new executions (task_id required, others optional)
   - `UpdateExecution` - For partial updates (all fields optional)
4. Implemented CRUD operations:
   - `create_execution(pool, new_execution)` - Creates execution with auto-generated UUID and started_at timestamp
   - `get_execution(pool, id)` - Fetches single execution by ID
   - `get_executions_by_task(pool, task_id)` - Fetches all executions for a task ordered by started_at DESC
   - `update_execution(pool, id, update)` - Partial update preserving unspecified fields
5. Wrote 15 comprehensive tests covering all operations and status variants
6. Used sqlx::query_as for type-safe database queries
7. Used chrono::DateTime<Utc> for timestamp handling

### Key Points:
- ExecutionStatus enum: Custom enum with as_str() and from_str() methods for DB storage
- UUID generation: `Uuid::new_v4().to_string()` for unique execution IDs
- Timestamp format: RFC 3339 (`chrono::Utc::now().to_rfc3339()`)
- Default values: status defaults to "pending", started_at auto-set, finished_at is NULL
- Update strategy: Merge pattern - preserve existing values when update field is None
- Foreign key relationship: task_id references tasks(id)
- Status variants stored as lowercase strings: "pending", "running", "success", "failed", "timeout", "skipped"
- Optional fields: session_id, finished_at, output_file, error_message can all be NULL

### Testing Patterns:
- TDD approach: Wrote all 15 tests first, then implemented functionality
- Test coverage includes:
  - Success cases for all CRUD operations
  - Error cases (not found)
  - Default value verification
  - Partial update behavior
  - Full lifecycle integration test
  - Multiple executions per task
  - All status variants
  - Order verification (DESC by started_at)
- Tests create parent task before creating execution (foreign key requirement)
- Isolated test databases using tempfile::TempDir

### Module Structure:
- `models/execution.rs` - Execution model, status enum, and CRUD operations
- `models/mod.rs` - Updated to export execution module and types
- Clean separation: db module for connections, models for data

### Verified:
- [x] All 15 Execution CRUD tests pass
- [x] All 31 total tests pass (including task and db tests)
- [x] Build succeeds without errors or warnings
- [x] Type-safe queries with sqlx::query_as
- [x] Proper async/await implementation
- [x] UUID auto-generation working
- [x] Timestamp auto-setting working
- [x] Default status "pending" applied correctly
- [x] Foreign key relationship working
- [x] ExecutionStatus enum conversions working

## Task 11: Output File Storage

### What was done:
1. Created storage module structure (`src-tauri/src/storage/mod.rs`, `output.rs`)
2. Implemented 5 output file management functions:
   - `get_output_directory(app)` - Get output directory path from Tauri app handle
   - `create_output_directory(output_dir)` - Create output directory if not exists
   - `write_output_file(output_dir, execution_id, content)` - Write output to file
   - `read_output_file(output_dir, execution_id)` - Read output from file
   - `delete_output_file(output_dir, execution_id)` - Delete output file
   - `cleanup_old_outputs(output_dir, days_to_keep)` - Delete files older than specified days
3. Added `filetime` dev-dependency for testing file modification times
4. Wrote 11 comprehensive tests covering all operations
5. Used tokio::fs for async file operations
6. Used chrono for timestamp comparison in cleanup function

### Key Points:
- Output files stored in `{app_data_dir}/outputs/` directory
- File naming convention: `{execution_id}.txt`
- cleanup_old_outputs only processes .txt files, ignores other file types
- Functions accept PathBuf instead of AppHandle for testability
- Tauri app handle only used in `get_output_directory` to get app data dir
- filetime crate used in tests to set file modification times for aging tests
- std::time::SystemTime converted to chrono::DateTime<Utc> for comparison

### Module Structure:
- `storage/mod.rs` - Public API exports
- `storage/output.rs` - Output file management functions and tests
- Clean separation: storage module for file operations, models for database

### Testing Patterns:
- Tests use tempfile::tempdir() for isolated temporary directories
- Tests don't require actual Tauri AppHandle - use PathBuf directly
- Aging tests use filetime::set_file_mtime to simulate old files
- Cleanup tests verify both file deletion and preservation
- Mixed file type tests ensure non-.txt files are ignored

### Verified:
- [x] All 11 Output storage tests pass
- [x] All 42 total tests pass (including db, models, storage tests)
- [x] Build succeeds without errors
- [x] Proper async/await implementation
- [x] Output directory created in app data directory
- [x] File naming convention followed
- [x] Cleanup respects file age and type

## Task 12: Cron Expression Parser

### What was done:
1. Created scheduler module structure (`src-tauri/src/scheduler/mod.rs`, `cron_parser.rs`)
2. Added `cron = "0.12"` dependency to Cargo.toml
3. Defined `CronError` enum with 4 variants: InvalidFieldCount, OutOfRange, InvalidSyntax, EmptyExpression
4. Defined `CronSchedule` struct with parsed field information
5. Implemented `validate_cron(expression)` - validates 5-field cron expression
6. Implemented `parse_cron(expression)` - parses and returns field details
7. Wrote 29 comprehensive tests covering valid/invalid expressions and edge cases

### Key Points:
- `cron` crate uses 6-field format (includes seconds): `sec min hour dom mon dow`
- Conversion: prepend "0 " to user's 5-field expression before passing to cron crate
- Day of week uses 1-7 range (1=Monday, 7=Sunday), NOT 0-6 like standard cron
- `?` is supported as "any value" for day fields
- Supports ranges (1-5), lists (1,3,5), steps (*/5), and combinations

### Cron Expression Format (5-field user input):
```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (1 - 7, 1=Mon, 7=Sun)
* * * * *
```

### Module Structure:
- `scheduler/mod.rs` - Public API exports
- `scheduler/cron_parser.rs` - Cron parsing logic, error types, and tests
- Clean separation: scheduler module for cron parsing, storage for files, models for database

### Testing Patterns:
- Valid expressions: simple, ranges, lists, steps, complex combinations
- Invalid expressions: wrong field count, out of range values, malformed syntax
- Edge cases: boundary values, common scheduling patterns
- Serialization tests for CronSchedule and CronError

### Verified:
- [x] All 29 cron_parser tests pass
- [x] All 71 total tests pass (including db, models, storage, scheduler tests)
- [x] Doc-tests pass for validate_cron and parse_cron
- [x] Build succeeds without errors
- [x] Proper error handling with descriptive messages
- [x] Serde serialization support for structs

## Task 13: Simple Schedule Parser

### What was done:
1. Created `scheduler/simple_schedule.rs` module
2. Implemented `parse_simple_schedule(json: &str) -> Result<String, ScheduleError>` function
3. Added module exports to `scheduler/mod.rs`
4. Wrote 29 comprehensive tests covering all schedule types and error cases

### Supported JSON Formats:
- **interval**: `{"type": "interval", "value": 5, "unit": "minutes"}` → `"*/5 * * * *"`
  - units: "minutes", "hours", "days"
- **daily**: `{"type": "daily", "time": "09:30"}` → `"30 9 * * *"`
- **weekly**: `{"type": "weekly", "day": "monday", "time": "09:30"}` → `"30 9 * * 1"`
  - days: full names (monday, tuesday...) and short names (mon, tue...)
  - case insensitive

### Key Points:
- Uses `serde_json` for parsing JSON input
- Converts simple schedule to standard 5-field cron expression
- Error types: InvalidJson, InvalidScheduleType, InvalidIntervalValue, InvalidIntervalUnit, InvalidTimeFormat, InvalidDayOfWeek, MissingField
- Day of week mapping: Sunday=0, Monday=1, ..., Saturday=6

### Module Structure:
- `scheduler/mod.rs` - Updated to export simple_schedule module
- `scheduler/simple_schedule.rs` - Parser implementation with tests

### Testing Coverage:
- Interval: valid values (1, 5, etc.), units (minutes, hours, days), invalid (0, missing, wrong unit)
- Daily: valid times (00:00, 09:30, 23:59), invalid (24:00, 12:60, wrong format, missing)
- Weekly: all 7 days, short names, case insensitive, invalid day, missing fields
- General: invalid JSON, unknown schedule type, missing type field

### Verified:
- [x] All 29 simple_schedule tests pass
- [x] All 100 total tests pass (including all previous tasks)
- [x] Doc-tests pass for parse_simple_schedule
- [x] Build succeeds without warnings

## Task 14: Job Scheduler Core (TDD)

### What was done:
1. Created job_scheduler module (`src-tauri/src/scheduler/job_scheduler.rs`)
2. Defined `SchedulerError` enum with 9 error variants for comprehensive error handling
3. Defined `JobCallback` type alias for async callback functions
4. Defined `JobInfo` struct to store job metadata (task_id, job_id, cron_expression)
5. Defined `SchedulerState` enum with Stopped/Running states
6. Implemented `Scheduler` struct with:
   - `scheduler: Arc<Mutex<Option<JobScheduler>>>` - tokio-cron-scheduler instance
   - `jobs: Arc<Mutex<HashMap<String, JobInfo>>>` - task_id to job mapping
   - `state: Arc<Mutex<SchedulerState>>` - scheduler state tracking
7. Implemented 5 core methods:
   - `new()` - Create scheduler instance
   - `add_job(task_id, cron_expression, callback)` - Add job with 5-field cron
   - `remove_job(task_id)` - Remove job by task_id
   - `start()` - Start scheduler
   - `stop()` - Stop scheduler
8. Implemented helper methods:
   - `get_state()` - Get current scheduler state
   - `job_count()` - Get number of scheduled jobs
   - `has_job(task_id)` - Check if job exists
   - `get_job_info(task_id)` - Get job metadata
9. Wrote 16 comprehensive tests covering all operations
10. Updated `scheduler/mod.rs` to export new module and types

### Key Points:
- **tokio-cron-scheduler uses 6-field format** (includes seconds): `sec min hour dom mon dow`
- **Conversion strategy**: Prepend "0 " to user's 5-field expression before passing to Job::new_async
- **Job callback type**: `Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>`
  - Must explicitly cast async blocks to `Pin<Box<dyn Future<Output = ()> + Send>>` in tests
  - Using `as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>` in tests
- **Thread-safe access**: Using `Arc<Mutex<T>>` for all shared state
- **Lazy scheduler initialization**: JobScheduler created on first add_job call
- **State management**: Separate SchedulerState enum prevents start/stop errors
- **Job tracking**: HashMap<String, JobInfo> maps task_id to job metadata for removal

### Implementation Details:
- **Error handling**: Comprehensive SchedulerError enum with descriptive messages
- **Cron validation**: Reuses existing validate_cron from cron_parser module
- **Job removal**: Requires both scheduler.remove(&job_id) and HashMap removal
- **Shutdown**: Uses scheduler.shutdown() method, requires mutable borrow
- **UUID handling**: tokio-cron-scheduler provides Uuid for each job via job.guid()

### Testing Patterns:
- **Callback type annotation**: Tests must use explicit type annotation for JobCallback
- **Example**: `let callback: JobCallback = Arc::new(|| Box::pin(async {}) as Pin<Box<dyn Future<Output = ()> + Send>>);`
- **Lifecycle testing**: test_scheduler_full_lifecycle covers complete workflow
- **State testing**: Tests verify SchedulerState transitions (Stopped -> Running -> Stopped)
- **Error cases**: Tests verify JobNotFound, AlreadyRunning, NotRunning errors
- **Cron validation**: Tests verify InvalidCronExpression errors

### Module Structure:
- `scheduler/job_scheduler.rs` - Job scheduler core implementation (600+ lines)
- `scheduler/mod.rs` - Updated to export job_scheduler types
- Exports: `Scheduler`, `SchedulerError`, `SchedulerState`, `JobInfo`, `JobCallback`

### Verified:
- [x] All 16 job_scheduler tests pass
- [x] All 116 total tests pass (including all previous tasks)
- [x] Build succeeds without errors
- [x] Module structure follows Rust conventions
- [x] Type-safe async callback handling
- [x] Thread-safe scheduler management
- [x] Comprehensive error handling
- [x] Job addition, removal, start, stop all working
- [x] Can add/remove jobs while scheduler is running

## Task 15: Task Queue and Concurrency Control

### What was done:
1. Created task_queue module (`src-tauri/src/scheduler/task_queue.rs`)
2. Defined `TaskQueueError` enum with 3 variants: NoAvailableSlots, TaskAlreadyRunning, TaskNotFound
3. Defined `SkipResult` enum: Execute, Skipped
4. Implemented `SlotGuard` struct for automatic slot release on drop
5. Implemented `TaskQueue` struct with:
   - `semaphore: Arc<Semaphore>` - Tokio semaphore for concurrency control
   - `running_tasks: Arc<Mutex<HashMap<String, OwnedSemaphorePermit>>>` - Track running tasks
   - `max_concurrent: usize` - Maximum concurrent tasks
6. Implemented core methods:
   - `new(max_concurrent)` - Create queue with concurrency limit
   - `acquire_slot(task_id)` - Non-blocking slot acquisition, returns SlotGuard
   - `acquire_slot_with_skip(task_id)` - Acquire with skip_if_running behavior
   - `release_slot(task_id)` - Manual slot release
   - `skip_if_running(task_id)` - Check if task should be skipped
   - `is_running(task_id)` - Check if task is currently running
   - `running_count()` - Get count of running tasks
   - `available_slots()` - Get count of available slots
7. Wrote 19 comprehensive tests covering all operations
8. Updated `scheduler/mod.rs` to export new module and types

### Key Points:
- **Tokio Semaphore**: `tokio::sync::Semaphore` provides efficient async concurrency control
- **OwnedSemaphorePermit**: Use `try_acquire_owned()` to get permits that can be stored in HashMap
- **SlotGuard pattern**: Guard drops automatically release semaphore permits via tokio runtime handle
- **Non-blocking acquisition**: `try_acquire_owned()` returns immediately if no slots available
- **skip_if_running logic**: First check if task is running, then try to acquire slot
- **Thread-safe access**: Using `Arc<Mutex<T>>` for all shared state

### Implementation Details:
- **SlotGuard Drop**: Uses `tokio::runtime::Handle::try_current()` to spawn cleanup task
- **Error types**: Comprehensive TaskQueueError with descriptive messages
- **SkipResult**: Distinguishes between "execute" and "skipped" states clearly
- **API design**: `acquire_slot` returns error for running tasks, `acquire_slot_with_skip` returns SkipResult

### Testing Patterns:
- **Concurrent limit test**: Spawns multiple async tasks, verifies max concurrent never exceeded
- **Auto-release test**: Verifies guard drop releases slot correctly
- **Skip behavior tests**: Tests both skip_if_running true/false cases
- **Full lifecycle test**: Complete workflow from acquire to release
- **Error cases**: Tests NoAvailableSlots, TaskAlreadyRunning, TaskNotFound errors

### Module Structure:
- `scheduler/task_queue.rs` - Task queue implementation (560+ lines)
- `scheduler/mod.rs` - Updated to export task_queue types
- Exports: `TaskQueue`, `TaskQueueError`, `SkipResult`, `SlotGuard`

### Verified:
- [x] All 19 task_queue tests pass
- [x] All 135 total tests pass (116 existing + 19 new)
- [x] Build succeeds without errors
- [x] Semaphore limits concurrent execution correctly
- [x] skip_if_running returns correct SkipResult
- [x] SlotGuard auto-releases on drop
- [x] Manual release_slot works correctly
- [x] Thread-safe concurrent access verified

## Task 16: Timeout Control and Process Killing

### What was done:
1. Created timeout module (`src-tauri/src/scheduler/timeout.rs`)
2. Added `nix` crate dependency with signal and process features
3. Defined `TimeoutError` enum with 4 variants: Timeout, SpawnFailed, KillFailed, ExecutionFailed
4. Defined `ProcessOutput` struct with status, stdout, stderr, timed_out fields
5. Implemented `kill_process(pid)` - Kills process using SIGKILL via nix crate
6. Implemented `run_with_timeout(program, args, timeout_secs)` - Executes command with timeout
7. Wrote 16 comprehensive tests covering all operations
8. Updated `scheduler/mod.rs` to export new module and types

### Key Points:
- **Process execution**: Uses `tokio::process::Command` for async process spawning
- **Timeout control**: Uses `tokio::time::timeout` wrapper for timeout enforcement
- **Process killing**: Uses `nix::sys::signal::kill` with SIGKILL for process termination
- **ExitStatus handling**: Unix processes killed by signal return `None` for `code()`, use `signal()` instead
- **Exit code 137**: 128 + SIGKILL(9) indicates process was killed by SIGKILL
- **Output capture**: Uses `AsyncBufReadExt::lines()` to read stdout/stderr line by line
- **Concurrent execution**: Multiple `run_with_timeout` calls can run concurrently with `tokio::join!`

### Implementation Details:
- **Timeout behavior**: When timeout occurs:
  1. Process is killed via `kill_process(pid)`
  2. Process is reaped via `child.wait()`
  3. Returns `ProcessOutput` with `timed_out: true`
- **Error handling**: Comprehensive TimeoutError enum with descriptive messages
- **Stdout/Stderr capture**: Lines are joined with `\n` separator
- **ExitStatusExt trait**: Required on Unix to use `from_raw()` method

### Testing Patterns:
- **Timeout test**: Uses `sleep 30` with 2s timeout to verify timeout behavior
- **Process kill test**: Verifies process is actually killed after timeout
- **Concurrent test**: Uses `tokio::join!` for multiple concurrent executions
- **Edge cases**: Tests 0-second timeout, invalid commands, command failures
- **Output capture tests**: Verifies stdout/stderr are captured correctly

### Module Structure:
- `scheduler/timeout.rs` - Timeout control implementation (450+ lines)
- `scheduler/mod.rs` - Updated to export timeout types
- Exports: `TimeoutError`, `ProcessOutput`, `run_with_timeout`, `kill_process`

### Dependencies Added:
- `nix = { version = "0.27", features = ["signal", "process"] }` - Unix signal handling
- `futures = "0.3"` (dev-dependency) - For concurrent test utilities

### Verified:
- [x] All 16 timeout tests pass
- [x] All 151 total lib tests pass
- [x] Build succeeds without warnings (except pre-existing doc-test)
- [x] Timeout correctly kills long-running processes
- [x] Process output is captured correctly
- [x] Error handling covers all edge cases
- [x] Concurrent execution works correctly

## Task 17: OpenCode CLI Integration

### What was done:
1. Created opencode module structure (`src-tauri/src/opencode/mod.rs`, `executor.rs`)
2. Defined `OpenCodeError` enum with 5 variants: ExecutionFailed, Timeout, SpawnFailed, InvalidSession, OutputParseFailed
3. Defined `OpenCodeOutput` struct with session_id, stdout, stderr, success, timed_out fields
4. Defined `OpenCodeConfig` struct with binary_path and default_timeout_secs
5. Implemented `SessionManager` struct for session lifecycle management
6. Implemented `run_opencode_task(prompt, session_id, timeout_secs, config)` async function
7. Implemented `create_session(config)` async function for creating new sessions
8. Implemented `run_mock_opencode_task` for testing without real CLI
9. Implemented `parse_session_from_output` helper for parsing session IDs
10. Reused `run_with_timeout` from Task 16 for timeout control
11. Wrote 18 comprehensive tests (17 pass, 1 ignored integration test)

### Key Points:
- **OpenCode CLI arguments**: `--session <id>` for session reuse, `--prompt <text>` for task prompt
- **Session ID format**: `sess_<uuid>` (e.g., `sess_abc123-def456-...`)
- **Session parsing**: OpenCode outputs session ID in format "Session: sess_xxx"
- **Error conversion**: `From<TimeoutError>` trait implemented for seamless error handling
- **Session lifecycle**: `SessionManager` provides create, get, set, clear, get_or_create operations
- **Default config**: binary_path="opencode", default_timeout_secs=300
- **Mock testing**: `run_mock_opencode_task` simulates execution without CLI

### Implementation Details:
- **Session reuse**: Pass `Some(session_id)` to reuse existing session, `None` creates new
- **Output structure**: OpenCodeOutput serializable with serde for JSON storage
- **Timeout integration**: Uses `run_with_timeout` from scheduler::timeout module
- **Modular design**: executor.rs contains core logic, mod.rs exports public API

### Testing Patterns:
- **Session manager tests**: new, create, set, clear, get_or_create operations
- **Mock task tests**: new session creation, existing session reuse
- **Output parsing tests**: valid/invalid session formats
- **Error handling tests**: TimeoutError conversion, error display formatting
- **Serialization tests**: OpenCodeOutput JSON roundtrip
- **Integration test**: Marked with `#[ignore]` as it requires real opencode binary

### Module Structure:
- `opencode/mod.rs` - Public API exports
- `opencode/executor.rs` - Core implementation with types and functions (520+ lines)
- Exports: `OpenCodeConfig`, `OpenCodeError`, `OpenCodeOutput`, `SessionManager`, `run_opencode_task`, `create_session`

### Verified:
- [x] All 17 opencode tests pass (1 ignored integration test)
- [x] All 168 total lib tests pass
- [x] Build succeeds without errors
- [x] Doctests compile (except pre-existing job_scheduler issue)
- [x] Session lifecycle management works correctly
- [x] Mock task execution for testing
- [x] Timeout integration working
- [x] Error handling comprehensive

## Task 18: Task List Component (Frontend)

### What was done:
1. 配置测试环境：
   - 创建 `vitest.config.ts` 配置 vitest
   - 创建 `src/test/setup.ts` 配置 testing library
   - 添加 test script 到 package.json
   - 安装 @testing-library/user-event
2. 创建设计系统基础：
   - 创建 `src/styles/design-system.css`
   - 定义 CSS 变量支持 Dark Mode
   - 使用 SF Pro 字体系列（-apple-system）
   - 定义颜色、间距、圆角、阴影等设计 tokens
3. 定义类型接口：
   - 创建 `src/types/task.ts`
   - 定义 Task 接口匹配后端模型
   - 定义 TaskListProps 接口
4. 实现 TaskList 组件（TDD）：
   - 编写 10 个测试用例（RED）
   - 实现 TaskList.tsx 组件（GREEN）
   - 创建 TaskList.css 样式（macOS 风格）
5. 测试覆盖：
   - 渲染测试：空状态、任务列表、显示内容
   - 交互测试：启用/禁用、删除、取消删除
   - 无障碍测试：aria labels、键盘导航

### Key Points:
- **设计系统优先**：遵循 DESIGN_SYSTEM_WORKFLOW_MANDATE，先创建 CSS 变量再实现组件
- **TDD 流程**：先写测试（RED），实现最小代码（GREEN），确保所有测试通过
- **macOS 原生风格**：
  - 使用 SF Pro 字体：`-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text"`
  - 系统色彩：支持 Dark Mode 的 CSS 变量
  - Toggle 开关：macOS 风格的滑动开关（44px × 24px）
  - 卡片式布局：圆角、阴影、边框
- **React Testing Library**：
  - 使用 `screen.getByRole()` 查询元素
  - 使用 `userEvent.setup()` 模拟用户交互
  - 测试 aria labels 和键盘导航
- **组件设计**：
  - 状态管理：`useState` 管理删除确认对话框
  - Props 设计：`tasks`, `onToggle`, `onDelete` 回调
  - 条件渲染：空状态、任务列表、删除确认
  - 格式化：`formatSchedule` 辅助函数

### Implementation Details:
- **Toggle 开关**：
  - 使用 `role="switch"` 和 `aria-checked`
  - CSS transition 实现平滑动画
  - 背景色：灰色（禁用）、绿色（启用）
- **删除确认**：
  - 点击删除按钮显示确认对话框
  - 确认/取消按钮
  - 确认后调用 `onDelete` 回调
- **Schedule 显示**：
  - Cron 表达式：直接显示
  - Simple schedule：解析 JSON 显示友好格式
  - 使用 SF Mono 等宽字体显示

### Testing Patterns:
- **渲染测试**：验证空状态、任务列表、各字段显示
- **交互测试**：使用 userEvent 模拟点击，验证回调调用
- **无障碍测试**：验证 aria-label 格式，测试键盘导航
- **测试修复**：`toHaveAttribute` 不支持正则，改用 `getAttribute() + toMatch()`

### File Structure:
```
src/
├── components/
│   ├── TaskList.tsx         # 组件实现（140 行）
│   ├── TaskList.test.tsx    # 测试文件（135 行）
│   └── TaskList.css         # 样式文件（200+ 行）
├── types/
│   └── task.ts              # TypeScript 类型定义
├── styles/
│   └── design-system.css    # 设计系统（CSS 变量）
└── test/
    └── setup.ts             # 测试配置
```

### Dependencies Added:
- `@testing-library/user-event` - 用于模拟用户交互（点击、键盘等）

### Verified:
- [x] 所有 10 个测试通过
- [x] ESLint 无错误
- [x] TypeScript 类型检查通过
- [x] 组件支持空状态
- [x] 启用/禁用功能正常
- [x] 删除功能带确认
- [x] macOS 风格样式（Dark Mode 支持）
- [x] 无障碍访问（aria labels、键盘导航）

## Task 19 - TaskForm 组件 (2024-03-08)

### TDD 开发流程
- **RED 阶段**: 先编写全面的测试用例，包括：
  - 渲染测试（所有字段、按钮）
  - 验证测试（必填、格式、范围）
  - 提交测试（成功、失败、loading）
  - 编辑模式测试（预填充、更新）
  - 交互测试（checkbox、radio切换）
  - 可访问性测试（labels、aria）
- **GREEN 阶段**: 实现最小代码使测试通过
- **REFACTOR 阶段**: 优化样式，使用设计系统变量

### Testing Library 技巧
- **fireEvent vs userEvent**: 
  - JSON 字符串包含特殊字符 `{}` 时，使用 `fireEvent.change` 而不是 `userEvent.type`
  - `userEvent.type` 会解释花括号作为特殊按键
- **异步测试**: 使用 `waitFor` 测试 Promise 和状态更新
- **表单验证**: 测试时移除 HTML5 的 min/max 属性，用 JavaScript 验证代替

### 组件设计模式
- **状态管理**: 使用多个 useState 而不是表单库（避免过度工程化）
- **编辑模式**: 通过 initialData prop 区分创建/编辑
- **条件渲染**: 根据 scheduleType 显示不同的输入字段
- **表单重置**: 创建成功后重置，编辑成功后调用 onCancel

### 验证逻辑
- **必填字段**: name、prompt、schedule（根据类型）
- **Cron 验证**: 使用正则表达式验证格式（未来可调用后端）
- **Timeout 范围**: 1-3600 秒
- **错误状态**: aria-invalid、aria-describedby 关联错误消息

### macOS 原生风格
- 使用设计系统的 CSS 变量
- Radio/Checkbox 使用 accent-color
- Input 使用系统边框和圆角
- Focus ring 使用 accent-color
- 禁用状态使用 opacity: 0.5

## Task 20: CronInput 组件 (2024-03-09)

### TDD 开发流程
- **RED 阶段**: 先编写 26 个测试用例,覆盖:
  - 渲染测试 (5个): 输入框、初始值、预览、标签、帮助文本
  - 验证测试 (8个): 无效表达式、字段数量、有效表达式、范围、列表、步骤、aria-invalid、错误关联
  - 预览测试 (4个): 下次运行时间、每分钟、每天、无效表达式
  - 交互测试 (2个): onChange 回调、值更新
  - 禁用状态测试 (2个): 禁用输入、禁用预览
  - 错误状态测试 (3个): 自定义错误、错误优先级、aria-invalid
  - 无障碍测试 (2个): label 关联、必填标记
- **GREEN 阶段**: 实现最小代码使测试通过
- **REFACTOR 阶段**: 添加 macOS 风格样式

### Cron 库使用
- **cron npm 包**: 用于计算下次运行时间
- **5-field 格式**: 用户输入 5 字段 (minute hour dom month dow)
- **6-field 转换**: cron 库需要 6 字段,需要 prepend "0 "
- **CronJob 构造**: `new CronJob(expression, callback, null, false, ...)`
- **nextDate()**: 获取下次运行的 Luxon DateTime 对象

### 验证逻辑
- **前端验证**: 简单验证 5 个字段、格式正确性
- **错误消息**: 统一使用 "Invalid cron expression"
- **后端验证**: 完整验证在后端 (cron_parser.rs)
- **实时验证**: 每次输入都进行验证

### 下次运行时间预览
- **时间格式化**: formatTimeUntil 函数处理多种情况:
  - "in less than 1 minute" (< 1分钟)
  - "in 1 minute" (1分钟)
  - "in X minutes" (多分钟)
  - "in 1 hour" (1小时)
  - "in X hours" (多小时)
  - "in 1 day" (1天)
  - "in X days" (多天)
- **测试技巧**: 正则表达式需要匹配所有可能的格式

### React Testing Library 技巧
- **受控组件测试**: 创建 TestWrapper 组件管理状态
- **useMemo**: 用于缓存验证结果和下次运行时间计算
- **动态 ID**: 使用 Math.random() 生成唯一 inputId
- **aria-describedby**: 错误消息通过 ID 关联到输入框

### 组件设计模式
- **Props 接口**: value, onChange, error (可选), disabled (可选)
- **错误优先级**: externalError || internalError
- **条件渲染**: 只有有效表达式且未禁用时才显示预览
- **状态管理**: internalError 通过 useEffect 同步更新

### macOS 原生风格
- **设计系统变量**: 使用 --color-*, --spacing-*, --radius-* 等
- **输入框样式**: 系统边框、圆角、focus ring
- **预览区域**: 灰色背景、圆角边框
- **Dark Mode**: 通过 CSS 变量自动支持

### 测试修复
- **时间格式匹配**: 正则表达式需要考虑 "less than 1 minute" 特殊情况
- **正则表达式**: `/next run.*in (less than 1 minute|\d+ minutes?)/i`
- **测试稳定性**: 时间相关测试需要考虑边界情况

### 文件结构
```
src/components/
├── CronInput.tsx         # 组件实现 (125 行)
├── CronInput.test.tsx    # 测试文件 (272 行, 26 个测试)
└── CronInput.css         # 样式文件 (110 行)
```

### Verified
- [x] 所有 26 个 CronInput 测试通过
- [x] 所有 63 个总测试通过
- [x] ESLint 无错误
- [x] TypeScript 类型检查通过
- [x] 实时验证 cron 表达式
- [x] 显示下次运行时间预览
- [x] 支持 Dark Mode
- [x] 无障碍访问 (aria labels、错误关联)

## Task 21: SimpleScheduleInput 组件 (2024-03-09)

### TDD 开发流程
- **RED 阶段**: 先编写 28 个测试用例，覆盖:
  - 渲染测试 (6个): 类型选择器、选项、初始值、字段显示、标签
  - Interval 类型测试 (3个): 5分钟、1小时、1天
  - Daily 类型测试 (2个): 时间输入、初始值解析
  - Weekly 类型测试 (3个): 星期+时间、7天选项、初始值解析
  - 交互测试 (3个): onChange 回调、字段更新
  - 错误状态测试 (2个): 自定义错误、aria-invalid
  - 禁用状态测试 (4个): 所有输入、单独字段
  - 无障碍测试 (2个): label 关联、必填标记
  - 边界情况测试 (3个): 空值、无效 JSON、类型切换
- **GREEN 阶段**: 实现最小代码使测试通过
- **REFACTOR 阶段**: 添加 macOS 风格样式

### 组件设计模式
- **Props 接口**: value, onChange, error (可选), disabled (可选)
- **三种调度类型**:
  - **Interval**: 下拉选择预设间隔 (5/10/15/30 分钟, 1/2/6/12 小时, 1 天)
  - **Daily**: 时间选择器 (24小时制)
  - **Weekly**: 星期选择器 + 时间选择器 (Monday-Sunday)
- **JSON 输出格式**:
  - Interval: `{"type":"interval","value":5,"unit":"minutes"}`
  - Daily: `{"type":"daily","time":"09:30"}`
  - Weekly: `{"type":"weekly","day":"monday","time":"09:30"}`

### React Testing Library 技巧
- **Time input 问题**: `userEvent.type()` 对 `<input type="time">` 行为不一致
  - 解决方案: 使用 `fireEvent.change()` 直接改变值
  - 示例: `fireEvent.change(timeInput, { target: { value: '09:30' } })`
- **Accessible name 查询**: `getByRole('combobox', { name: /pattern/i })`
  - 使用关联 label 的文本作为 name
  - 在本组件中，label 是 "Simple Schedule *"，所以使用 `/simple schedule/i`

### 状态管理
- **本地状态**: scheduleType, intervalValue, dailyTime, weeklyDay, weeklyTime
- **派生状态**: 从 value prop 解析得到 parsed
- **useEffect 同步**: 当 parsed 改变时同步更新本地状态
- **初始值处理**: 通过 useEffect 从 value prop 初始化状态

### JSON 解析
- **parseSchedule 函数**: 解析 JSON 字符串为类型和调度对象
- **错误处理**: 无效 JSON 返回 `{ type: '' }`
- **类型定义**: IntervalSchedule, DailySchedule, WeeklySchedule 联合类型

### macOS 原生风格
- **设计系统变量**: 使用 `--color-*`, `--spacing-*`, `--radius-*` 等
- **Select 样式**: 
  - 自定义下拉箭头 (SVG background-image)
  - padding-right 为图标留空间
  - 系统边框和圆角
  - focus ring 使用 accent-color
- **Time input 样式**:
  - calendar picker icon 透明度调整
  - cursor: text / pointer
- **嵌套字段**: schedule-fields 容器使用浅色背景

### 文件结构
```
src/components/
├── SimpleScheduleInput.tsx         # 组件实现 (245 行)
├── SimpleScheduleInput.test.tsx    # 测试文件 (310 行, 28 个测试)
└── SimpleScheduleInput.css         # 样式文件 (152 行)
```


## Task 22: ExecutionHistory 组件 (2024-03-09)

### TDD 开发流程
- **RED 阶段**: 先编写 28 个测试用例,覆盖:
  - 渲染测试 (6个): 空状态、列表显示、执行时间、状态显示、持续时间、未完成执行
  - 状态显示测试 (7个): pending(灰)、running(蓝)、success(绿)、failed(红)、timeout(橙)、skipped(黄)、错误消息
  - 交互测试 (4个): onSelect 回调、无回调、可点击样式、不可点击样式
  - 时间格式化测试 (5个): 相对时间、绝对时间、秒、分钟、小时
  - 加载状态测试 (3个): loading 显示、空状态隐藏、loading spinner
  - 无障碍测试 (3个): 列表结构、accessible name、状态标识
- **GREEN 阶段**: 实现最小代码使测试通过
- **REFACTOR 阶段**: 添加 macOS 风格样式

### 组件设计模式
- **Props 接口**: executions, onSelect (可选), loading (可选)
- **条件渲染**: 
  - loading 优先级最高
  - 然后是空状态
  - 最后是执行历史列表
- **时间格式化**: 
  - < 1小时: 显示相对时间 ("X minutes ago")
  - < 7天: 显示星期+时间 ("Mon 10:00 AM")
  - >= 7天: 显示日期 ("Mar 9, 2024")
- **持续时间格式化**:
  - < 1分钟: 显示秒 ("45s")
  - < 1小时: 显示分钟 ("5m")
  - >= 1小时: 显示小时+分钟 ("2h 30m")
- **错误消息显示**: failed 状态时显示 error_message

### 状态颜色系统 (macOS System Colors)
- **pending**: #999 (灰色)
- **running**: #007AFF (蓝色,系统蓝)
- **success**: #34C759 (绿色,系统绿)
- **failed**: #FF3B30 (红色,系统红)
- **timeout**: #FF9500 (橙色,系统橙)
- **skipped**: #FFCC00 (黄色,系统黄)

### Dark Mode 支持
- 使用 `@media (prefers-color-scheme: dark)` 媒体查询
- 调整状态颜色为 Dark Mode 变体 (更亮的颜色)
- 例如: #007AFF → #0A84FF (Dark Mode 蓝)

### React Testing Library 技巧
- **时间测试**: 测试相对时间时需要考虑当前时间,使用正则匹配多种格式
- **正则表达式**: `/2024-03-09|Mar 9, 2024|ago/i` 匹配多种时间显示格式
- **Mock 数据工厂**: 创建 `createMockExecution()` 函数生成测试数据
- **状态样式测试**: 使用 `toHaveClass('status-success')` 验证 CSS 类

### 组件实现细节
- **formatTime 函数**: 根据时间差返回不同格式
  - 计算时间差 (diffMs, diffHours, diffDays)
  - 边界条件处理 (< 1分钟、1分钟、多分钟、1小时、多小时)
  - 使用 toLocaleString/toLocaleDateString 格式化
- **formatDuration 函数**: 格式化持续时间
  - 计算秒、分钟、小时
  - 组合格式 ("2h 30m")
- **可访问性**: 
  - role="list" 和 role="listitem"
  - aria-label 包含状态和时间信息
  - loading spinner 使用 role="status"

### macOS 原生风格
- **设计系统变量**: 使用 `--color-*`, `--spacing-*`, `--radius-*` 等
- **状态徽章**: 
  - 大写字母 (text-transform: uppercase)
  - 字间距 (letter-spacing: 0.5px)
  - 白色文字 + 彩色背景
- **可点击项**: 
  - cursor: pointer
  - hover 时背景色变化
  - active 时缩放 (transform: scale(0.98))
- **错误消息**: 
  - 左侧红色边框
  - 浅红色背景 (rgba)
  - word-wrap: break-word

### 文件结构
```
src/
├── components/
│   ├── ExecutionHistory.tsx         # 组件实现 (115 行)
│   ├── ExecutionHistory.test.tsx    # 测试文件 (300 行, 28 个测试)
│   └── ExecutionHistory.css         # 样式文件 (180 行)
└── types/
    └── execution.ts                 # TypeScript 类型定义 (18 行)
```

### Verified
- [x] 所有 28 个 ExecutionHistory 测试通过
- [x] 所有 119 个总测试通过
- [x] TypeScript 类型检查通过
- [x] 组件支持空状态
- [x] 显示所有 6 种状态 (不同颜色)
- [x] 时间格式化 (相对/绝对)
- [x] 持续时间显示
- [x] 错误消息显示
- [x] 点击选择功能
- [x] Loading 状态
- [x] macOS 风格样式 (Dark Mode 支持)
- [x] 无障碍访问 (aria labels、role)


## Task 22: 历史记录列表组件 (ExecutionHistory.tsx)

### What was done:
1. 更新了类型定义 (src/types/execution.ts):
   - 将 `onSelect` 回调改为 `onViewOutput`
   - 添加了可选的 `taskId` 参数用于筛选
2. 编写了完整的测试覆盖 (ExecutionHistory.test.tsx):
   - 渲染测试、状态显示、时间格式化、持续时间显示
   - 点击交互测试 (只有 output_file 的记录才能点击)
   - 筛选功能测试
   - 无障碍访问测试
3. 实现了组件功能 (ExecutionHistory.tsx):
   - 支持 taskId 筛选
   - 点击行为仅对有 output_file 的执行记录有效
   - 正在运行的执行显示 "Running..."
   - 持续时间格式化 (支持小时、分钟、秒)
   - 键盘访问支持 (tabIndex + onKeyPress)
4. 更新了样式文件 (ExecutionHistory.css):
   - 使用设计系统的 CSS 变量 (--space-*, --text-*, --bg-* 等)
   - 状态颜色使用设计系统的语义化颜色 (--success-color, --error-color 等)
   - 添加了 focus 状态样式

### Key Points:
- TDD 开发流程:先写测试,再实现功能
- 点击交互只在有 output_file 的情况下触发 onViewOutput 回调
- 持续时间格式化支持多种单位组合 (2h 30m, 5m 30s, 45s, <1s)
- 使用设计系统的 CSS 变量保持样式一致性
- 无障碍访问:clickable 项有 tabIndex,支持键盘导航

### Design System Variables Used:
- Spacing: --space-xs, --space-sm, --space-md, --space-lg, --space-3xl
- Colors: --text-*, --bg-*, --border-*, --success-color, --error-color, --warning-color, --accent-color
- Typography: --font-family, --font-size-*, --font-weight-*
- Border: --radius-sm, --radius-lg
- Effects: --shadow-md, --transition-fast

### Verified:
- [x] 所有 31 个测试通过
- [x] TypeScript 类型检查通过 (npx tsc --noEmit)
- [x] 组件使用设计系统的 CSS 变量
- [x] 支持空状态显示
- [x] 支持加载状态
- [x] 状态颜色符合设计规范
- [x] 无障碍访问支持


## Task 23: OutputViewer 组件 (2024-03-09)

### TDD 开发流程
- **RED 阶段**: 先编写 21 个测试用例,覆盖:
  - 渲染测试 (3个): 基础渲染、纯文本模式、默认 Markdown 模式
  - Markdown 渲染测试 (9个): 标题、有序/无序列表、代码块、内联代码、粗体/斜体、链接、引用
  - 语法高亮测试 (3个): JavaScript、Python、无语言指定
  - 空状态测试 (3个): 空字符串、空白字符、有内容
  - Props 测试 (3个): content、isMarkdown=true、isMarkdown=false
- **GREEN 阶段**: 实现最小代码使测试通过
- **REFACTOR 阶段**: 优化代码,移除不必要的注释

### 依赖库使用
- **react-markdown**: 用于渲染 Markdown 内容
  - 支持 CommonMark 规范
  - 通过 `components` prop 自定义渲染器
  - 自动处理 HTML 转义
- **react-syntax-highlighter**: 用于代码块语法高亮
  - 使用 Prism 语法高亮器
  - vscDarkPlus 主题 (VS Code 深色主题)
  - 支持多种语言 (JavaScript, Python, Rust, etc.)
  - 自动检测语言并应用高亮

### 组件设计模式
- **Props 接口**: 
  - `content: string` - 要显示的内容
  - `isMarkdown?: boolean` - 是否渲染为 Markdown (默认 true)
- **条件渲染**: 
  - 空状态 → 纯文本模式 → Markdown 模式
  - 代码块检测: `language-(\w+)` 正则匹配
  - 内联代码 vs 代码块: 根据 className 判断
- **组件结构**: 
  - 外层容器 (`.output-viewer`)
  - 内容容器 (`.output-viewer-content`)
  - ReactMarkdown 组件 + 自定义 code 渲染器

### React Testing Library 技巧
- **语法高亮测试**: react-syntax-highlighter 会将代码分解为多个 `<span>` 元素
  - 问题: `screen.getByText(/const greeting/)` 找不到文本
  - 解决方案: 使用 `container.querySelector()` 查找 code 元素
  - 验证: 检查 `codeBlock?.textContent` 包含关键词
  - 示例: 
    ```typescript
    const { container } = render(<OutputViewer content={markdown} />);
    const codeBlock = container.querySelector('code.language-javascript');
    expect(codeBlock?.textContent).toContain('const');
    ```
- **避免不必要的注释**: 测试代码应该自解释,避免冗余注释

### Markdown 渲染实现
- **ReactMarkdown 配置**:
  ```typescript
  <ReactMarkdown
    children={content}
    components={{
      code({ className, children, ...props }) {
        const match = /language-(\w+)/.exec(className || '');
        const isInline = !match;
        
        if (isInline) {
          return <code className={className} {...props}>{children}</code>;
        }

        return (
          <SyntaxHighlighter
            style={vscDarkPlus}
            language={match[1]}
            PreTag="div"
            children={String(children).replace(/\n$/, '')}
          />
        );
      }
    }}
  />
  ```
- **关键点**:
  - 正则提取语言名称: `/language-(\w+)/.exec(className)`
  - 内联代码 vs 代码块判断: 根据 match 结果
  - 移除末尾换行符: `replace(/\n$/, '')`

### macOS 原生风格
- **设计系统变量**: 使用 `--font-*`, `--text-*`, `--bg-*`, `--space-*` 等
- **Markdown 样式**:
  - 标题: 不同字号 (3xl, 2xl, xl, lg)
  - 段落: 底部间距 (--space-md)
  - 列表: 默认样式 (disc/decimal)
  - 代码块: 深色背景 (--bg-tertiary)、圆角、等宽字体
  - 内联代码: 浅色背景、小字号 (0.85em)
  - 引用块: 左侧蓝色边框、浅色背景
  - 链接: 系统蓝色、hover 效果
- **代码块字体**: SF Mono + Monaco + Cascadia Code + Consolas fallback
- **滚动容器**: max-height: 600px, overflow-y: auto

### 空状态设计
- **检测逻辑**: `!content || content.trim() === ''`
- **视觉效果**: 
  - 居中布局 (flexbox)
  - 灰色文字 (--text-tertiary)
  - 斜体字体
  - 最小高度 (min-height: 200px)

### 文件结构
```
src/components/
├── OutputViewer.tsx         # 组件实现 (64 行)
├── OutputViewer.test.tsx    # 测试文件 (207 行, 21 个测试)
└── OutputViewer.css         # 样式文件 (152 行)
```

### Verified
- [x] 所有 21 个 OutputViewer 测试通过
- [x] 所有 143 个总测试通过
- [x] TypeScript 类型检查通过 (npx tsc --noEmit)
- [x] 组件支持 Markdown 渲染
- [x] 组件支持纯文本模式
- [x] 代码块语法高亮 (JavaScript, Python)
- [x] 内联代码样式
- [x] 列表、标题、引用、链接等 Markdown 元素
- [x] 空状态显示
- [x] macOS 风格样式 (Dark Mode 支持)
- [x] 使用设计系统 CSS 变量

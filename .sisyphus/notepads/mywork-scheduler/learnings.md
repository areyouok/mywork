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

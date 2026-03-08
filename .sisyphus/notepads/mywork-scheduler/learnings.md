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

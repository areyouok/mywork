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

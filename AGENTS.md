# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-09
**Type:** Tauri 2 Desktop App

## OVERVIEW

macOS system tray app for scheduling and managing OpenCode task executions. React 19 + TypeScript frontend, Rust + Tauri 2 backend, SQLite persistence.

## STRUCTURE

```
mywork/
├── src/              # React frontend (see src/AGENTS.md)
├── src-tauri/src/    # Rust backend (see src-tauri/src/AGENTS.md)
├── tests/e2e/        # Playwright E2E tests
└── .sisyphus/        # AI dev tool config
```

## WHERE TO LOOK

| Task                  | Location                   | Notes                         |
| --------------------- | -------------------------- | ----------------------------- |
| Add new UI component  | `src/components/`          | Colocated .tsx/.test.tsx/.css |
| Add Tauri command     | `src-tauri/src/commands/`  | Register in lib.rs            |
| Modify task scheduler | `src-tauri/src/scheduler/` | tokio-cron-scheduler          |
| Database operations   | `src-tauri/src/db/`        | SQLite via sqlx               |
| Frontend types        | `src/types/`               | Shared TS interfaces          |
| E2E tests             | `tests/e2e/*.spec.ts`      | Playwright                    |

## CONVENTIONS

- **Path alias**: `@/` maps to `./src/` (tsconfig + vite)
- **Line width**: 100 chars (prettier)
- **Underscore params**: `_foo` allowed in unused args (eslint)
- **React imports**: Not required in JSX scope (React 19)
- **Dev server port**: 1420 (fixed, strictPort)
- **Test globals**: Vitest globals enabled (no imports needed)

## ANTI-PATTERNS (THIS PROJECT)

- No `as any` type assertions
- No `@ts-ignore` comments
- No empty catch blocks
- No TODO/FIXME/HACK comments (project rule)
- No `no_explicit_any` error → warn only

## COMMANDS

```bash
npm run dev           # Vite dev server
npm run tauri dev     # Full app development
npm run tauri build   # Production build
npm test              # Vitest unit tests
npm run test:e2e      # Playwright E2E tests
cargo test            # Rust backend tests
npm run lint          # ESLint
npm run format        # Prettier
```

## DATA STORAGE

- DB: `~/Library/Application Support/com.mywork/mywork.db`
- Outputs: `~/Library/Application Support/com.mywork/outputs/`

## NOTES

- macOS only (system tray app)
- No CI/CD configured (manual builds)
- Tauri IPC: frontend calls Rust commands via `@tauri-apps/api`
- Scheduler uses tokio-cron-scheduler with cron expressions or simple intervals

# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-12
**Type:** Tauri 2 Desktop App

## OVERVIEW

macOS system tray app for scheduling and managing OpenCode task executions. React 19 + TypeScript frontend, Rust + Tauri 2 backend, SQLite persistence.

## STRUCTURE

```
mywork/
├── src/              # React frontend (see src/AGENTS.md)
├── src-tauri/src/    # Rust backend (see src-tauri/src/AGENTS.md)
└── tests/e2e/        # Playwright E2E tests
```

## WHERE TO LOOK

| Task                  | Location                   | Notes                         |
| --------------------- | -------------------------- | ----------------------------- |
| Add new UI component  | `src/components/`          | Colocated .tsx/.test.tsx/.css |
| Add Tauri command     | `src-tauri/src/commands/`  | Register in lib.rs            |
| Modify task scheduler | `src-tauri/src/scheduler/` | tokio-cron-scheduler          |
| Database operations   | `src-tauri/src/db/`        | SQLite via sqlx               |
| Execution cleanup     | `src-tauri/src/`           | execution_retention.rs        |
| Frontend types        | `src/types/`               | Shared TS interfaces          |
| E2E tests             | `tests/e2e/*.spec.ts`      | Playwright                    |

## CONVENTIONS

- **Path alias**: `@/` maps to `./src/` (tsconfig + vite)
- **Line width**: 100 chars (prettier)
- **Underscore params**: `_foo` allowed in unused args (eslint)
- **React imports**: Not required in JSX scope (React 19)
- **Dev server port**: 1420 (fixed, strictPort)
- **Test globals**: Vitest globals enabled (no imports needed)

## TDD PRINCIPLES

This project follows Test-Driven Development (TDD) methodology strictly.

### Core Rules

1. **Red-Green-Refactor Cycle**
   - Write a failing test first (Red)
   - Write minimal code to pass the test (Green)
   - Refactor while keeping tests green (Refactor)

2. **Test First, Always**
   - No production code without a failing test
   - Tests drive the design, not just verify it
   - Write tests before implementing features

3. **Minimal Implementation**
   - Write only enough code to make the test pass
   - Don't implement features not covered by tests
   - Avoid speculative generalization

4. **Refactor Mercilessly**
   - After tests pass, improve code quality
   - Remove duplication ruthlessly
   - Simplify complex logic
   - Keep tests green during refactoring

5. **Fast Feedback Loop**
   - Unit tests must run in milliseconds
   - Use mocks/stubs for external dependencies
   - E2E tests for critical user journeys only

### Test Organization

- **Unit tests**: Colocated with source files (`.test.ts`, `.test.tsx`)
- **Integration tests**: Test module interactions
- **E2E tests**: `tests/e2e/` for user workflows
- **Rust tests**: `#[cfg(test)]` modules in source files

### Test Quality Standards

- One assertion per test when possible
- Descriptive test names (e.g., "should return error when input is empty")
- Use Arrange-Act-Assert pattern
- Avoid test interdependencies
- No shared mutable state between tests

### Verification Checklist

Before committing:

- [ ] All new code has corresponding tests
- [ ] All tests pass locally
- [ ] Test coverage maintained or improved
- [ ] No skipped tests without justification
- [ ] Refactoring done while keeping tests green

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

## GIT WORKFLOW

- Review changes with `git diff` before committing
- Never commit: `dist/`, `build/`, `target/`, `*.log`, `*.tmp`, `node_modules/`, `.env`
- Use English for commit messages

## DATA STORAGE

- DB: `~/Library/Application Support/com.mywork/mywork.db`
- Outputs: `~/Library/Application Support/com.mywork/outputs/`

## NOTES

- macOS only (system tray app)
- No CI/CD configured (manual builds)
- Tauri IPC: frontend calls Rust commands via `@tauri-apps/api`
- Scheduler uses tokio-cron-scheduler with cron expressions or simple intervals

# MyWork Frontend Module

**Type:** React 19 + TypeScript + Vite

## OVERVIEW

UI components for task management, scheduling, and execution history viewing.

## STRUCTURE

```
src/
├── components/    # UI components (*.tsx + *.test.tsx + *.css)
├── types/         # TypeScript interfaces (Task, Execution, Schedule)
├── api/           # Tauri IPC wrappers
├── test/          # Test setup (jsdom + jest-dom)
├── styles/        # Global CSS
└── assets/        # Static files
```

## WHERE TO LOOK

| Task             | Location                              | Notes                   |
| ---------------- | ------------------------------------- | ----------------------- |
| Task list UI     | `components/TaskList.tsx`             | Main list display       |
| Task form        | `components/TaskForm.tsx`             | Create/edit tasks       |
| Cron input       | `components/CronInput.tsx`            | Cron expression builder |
| Simple schedule  | `components/SimpleScheduleInput.tsx`  | Dropdown intervals      |
| History view     | `components/ExecutionHistory.tsx`     | Execution records       |
| Output viewer    | `components/OutputViewer.tsx`         | Markdown rendering      |
| API calls        | `api/tauri.ts`                        | Invoke Tauri commands   |
| Type definitions | `types/task.ts`, `types/execution.ts` | Shared interfaces       |

## CONVENTIONS

- **Component colocation**: `.tsx`, `.test.tsx`, `.css` in same directory
- **Imports**: Use `@/` alias for src paths
- **Tests**: Unit tests use `.test.tsx` suffix, colocated with components
- **No React imports**: React 19 automatic JSX transform

## ANTI-PATTERNS

- No `as any` (warn only per project config)
- No `@ts-ignore`
- No inline styles (use CSS files)

## NOTES

- Entry: `main.tsx` → `App.tsx`
- API: All backend calls via `@tauri-apps/api` invoke
- Output viewer supports Markdown + syntax highlighting

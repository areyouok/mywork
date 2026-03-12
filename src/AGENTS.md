# MyWork Frontend Module

**Type:** React 19 + TypeScript + Vite

## OVERVIEW

UI components for task management, scheduling, and execution history viewing with real-time streaming output support.

## STRUCTURE

```
src/
├── components/    # UI components (*.tsx + *.test.tsx + *.css)
├── hooks/         # Custom React hooks for state management
├── types/         # TypeScript interfaces (Task, Execution)
├── api/           # Tauri IPC wrappers
├── utils/         # Utility functions (formatting, etc.)
├── styles/        # Global CSS (design system, theming)
├── test/          # Test setup (jsdom + jest-dom)
└── assets/        # Static files
```

## WHERE TO LOOK

| Task             | Location                              | Notes                          |
| ---------------- | ------------------------------------- | ------------------------------ |
| Task list UI     | `components/TaskList.tsx`             | Main list display              |
| Task form        | `components/TaskForm.tsx`             | Create/edit tasks              |
| Cron input       | `components/CronInput.tsx`            | Cron expression builder        |
| Simple schedule  | `components/SimpleScheduleInput.tsx`  | Dropdown intervals             |
| History view     | `components/ExecutionHistory.tsx`     | Execution records              |
| Output viewer    | `components/OutputViewer.tsx`         | Markdown + syntax highlighting |
| ANSI rendering   | `components/AnsiRenderer.tsx`         | ANSI escape → HTML converter   |
| API calls        | `api/tasks.ts`                        | All Tauri IPC wrappers         |
| Type definitions | `types/task.ts`, `types/execution.ts` | Task, Execution interfaces     |
| Task state       | `hooks/useTasks.ts`                   | Task CRUD + scheduler reload   |
| Scheduler state  | `hooks/useScheduler.ts`               | Scheduler status + running IDs |
| Execution state  | `hooks/useExecutions.ts`              | Execution list loading         |
| Output loading   | `hooks/useOutput.ts`                  | Load execution output          |
| Streaming output | `hooks/useStreamingOutput.ts`         | Real-time output streaming     |
| Action handlers  | `hooks/useTaskActions.ts`             | Toggle/delete/run handlers     |
| Time formatting  | `utils/format.ts`                     | Relative time, duration        |
| Design system    | `styles/design-system.css`            | CSS variables, dark mode       |
| Channel test     | `components/ChannelTest.tsx`          | IPC channel testing (dev only) |

## CONVENTIONS

- **Component colocation**: `.tsx`, `.test.tsx`, `.css` in same directory
- **Imports**: Use `@/` alias for src paths
- **Tests**: Unit tests use `.test.tsx` suffix, colocated with components
- **No React imports**: React 19 automatic JSX transform
- **Custom hooks**: All state management via custom hooks (no Redux/Zustand)
- **Theming**: CSS variables in design-system.css, supports light/dark mode

## ANTI-PATTERNS

- No `as any` (warn only per project config)
- No `@ts-ignore`
- No inline styles (use CSS files)

## NOTES

- Entry: `main.tsx` → `App.tsx`
- API: All backend calls via `@tauri-apps/api` invoke
- Output viewer supports Markdown + syntax highlighting + ANSI colors
- Real-time updates via Tauri events (`execution-started`, `execution-finished`)
- View routing: `list | form | history | output`

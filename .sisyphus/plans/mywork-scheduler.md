# mywork - opencode 定时调度应用

## TL;DR

> **Quick Summary**: 创建一个基于 Tauri + React 的 macOS 系统托盘应用，用于定时调度和执行 opencode 任务。支持 cron 表达式和简单时间选择器，具备超时控制、并发控制、历史记录查看等功能。
> 
 > **Deliverables**:
> - macOS 系统托盘应用（Tauri + React）
> - 定时任务调度系统（Rust: tokio-cron-scheduler + 内置队列）
> - 任务管理界面（创建、编辑、查看）
> - 执行历史和输出查看
> - SQLite 数据持久化（Rust: sqlx）
> - opencode CLI 集成（Rust 调用 opencode CLI binary）
> - 自动化测试（TDD）
> - Git 仓库管理
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 5 waves
> **Critical Path**: Setup → Data Layer → Scheduler → Integration → UI → Testing

---

## Context

### Original Request
创建一个基于 opencode 的调度、编排 GUI 应用，用于定时执行 opencode 任务。要求：
- 仅支持 macOS
- 常驻任务栏，自调度（不用 macOS 系统调度）
- 定时任务管理（创建、查看、历史记录）
- 超时控制和并发控制
- 原生 macOS 设计风格
- 高代码质量和测试覆盖率
- CLI-only 环境，无 Xcode

### Interview Summary

**Key Discussions**:
- **技术栈**: Tauri + React 18 (内存占用小，80MB vs Electron 200-500MB)
- **后端架构**: 纯 Rust（单一技术栈，性能最优）
  - 调度器: `tokio-cron-scheduler` (async, production-ready)
  - 并发控制: Tokio semaphore (原生支持)
  - 存储: `sqlx` (SQLite + WAL 模式, async)
  - 进程管理: `tokio::process::Command` (超时 + kill)
- **opencode 集成**: 直接调用 opencode CLI binary（通过 Tauri shell）
- **定时规则**: 同时支持 cron 表达式和简单时间选择器
- **测试策略**: TDD (测试驱动开发) - `cargo test` for Rust, `vitest` for React
- **设计风格**: 原生 macOS 风格（系统字体、Dark Mode、HIG）
- **Git 管理**: 自动初始化，维护 .gitignore，定期提交

**Research Findings**:
- **opencode CLI**: 通过 `opencode serve` 启动本地服务器，SDK 通过 HTTP API 调用
- **Session 模型**: 每个 session 可以处理多个 prompts（有状态），建议每个任务创建独立 session
- **输出流**: 支持 SSE (Server-Sent Events) 用于实时流式输出
- **Tauri 优势**: 内存小、启动快、打包小、安全性高、无需 Xcode
- **node-cron**: 纯 Node.js，无外部依赖，API 简洁
- **better-sqlite3**: 性能最佳，支持 WAL 模式

### Metis Review

**Identified Gaps** (已解决):
- ✅ **opencode SDK session 模型**: 确认每个任务创建独立 session
- ✅ **Task 数据模型**: 定义了完整的 schema (id, name, prompt, cron, timeout, enabled, etc.)
- ✅ **数据保留策略**: 默认保留 30 天执行历史，输出存储为文件
- ✅ **输出格式**: 文本格式，支持 Markdown 渲染（v1）
- ✅ **原生 macOS 设计**: 使用系统字体 SF Pro、支持 Dark Mode、遵循 HIG
- ✅ **托盘菜单**: 显示任务数量、最近执行状态、Open/Quit 选项
- ✅ **测试策略**: 使用时间模拟库测试调度器

**Guardrails Applied**:
- MUST NOT: 多平台支持、实时协作、云同步、用户认证、插件系统
- MUST: TDD、获取 SDK 文档、定义 schema、设置数据保留策略

---

## Work Objectives

### Core Objective
构建一个生产级的 macOS 系统托盘应用，用于管理和定时执行 opencode 任务，具备完善的任务管理、历史记录、超时控制等功能，并保持高质量的代码和测试覆盖率。

### Concrete Deliverables

**后端 (Rust + Node.js)**:
- `src-tauri/` - Tauri 主进程
  - 系统托盘管理
  - SQLite 数据库操作
  - opencode CLI 进程管理
  - 调度器逻辑
- `src-tauri/src/scheduler/` - 调度模块
- `src-tauri/src/db/` - 数据库模块
- `src-tauri/src/opencode/` - opencode 集成模块

**前端 (React + TypeScript)**:
- `src/` - React 应用
  - 任务列表界面
  - 任务创建/编辑表单
  - 历史记录查看
  - 输出查看器
  - 托盘菜单
- `src/components/` - UI 组件
- `src/hooks/` - React hooks
- `src/utils/` - 工具函数

**测试**:
- `src-tauri/src/**/*.rs` - Rust 单元测试
- `src/**/*.test.ts` - React 组件测试
- `tests/` - E2E 测试 (Playwright)

**配置和文档**:
- `package.json` - 依赖配置
- `src-tauri/Cargo.toml` - Rust 依赖
- `tsconfig.json` - TypeScript 配置
- `vite.config.ts` - Vite 配置
- `.gitignore` - Git 忽略规则
- `README.md` - 项目文档

### Definition of Done

- [ ] 应用可以成功构建为 macOS .app
- [ ] 系统托盘图标可见，菜单可用
- [ ] 可以创建、编辑、删除定时任务
- [ ] 任务按调度规则执行（cron 和简单选择器）
- [ ] 超时任务被正确终止
- [ ] 重叠任务被正确跳过
- [ ] 执行历史可查看
- [ ] 输出可查看（文本 + Markdown）
- [ ] 所有测试通过（单元 + E2E）
- [ ] 代码覆盖率 ≥ 80%
- [ ] Git 仓库管理良好（.gitignore 正确）
- [ ] README 文档完整

### Must Have

- ✅ macOS 系统托盘常驻
- ✅ 定时任务调度（cron + 简单选择器）
- ✅ 任务 CRUD 操作
- ✅ 超时控制和并发控制
- ✅ 执行历史记录
- ✅ 输出查看
- ✅ TDD 测试覆盖
- ✅ 原生 macOS 设计
- ✅ SQLite 数据持久化
- ✅ Git 仓库管理

### Must NOT Have (Guardrails)

- ❌ 非 macOS 平台支持
- ❌ macOS 系统级调度（launchd）
- ❌ Xcode 相关依赖
- ❌ 云端同步
- ❌ 用户认证/账户
- ❌ 实时协作
- ❌ 插件/扩展系统
- ❌ 内置 prompt 模板市场
- ❌ WebSocket 实时流式输出（v1 使用轮询）
- ❌ 多平台支持（Windows/Linux）

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (需要从零搭建)
- **Automated tests**: TDD (测试驱动开发)
- **Framework**: 
  - Rust: `cargo test` (内置)
  - React: `vitest` + `@testing-library/react`
  - E2E: `playwright` + `@playwright/test`
- **TDD**: 每个任务遵循 RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Frontend/UI**: Use Playwright (playwright skill) — Navigate, interact, assert DOM, screenshot
- **TUI/CLI**: Use interactive_bash (tmux) — Run command, send keystrokes, validate output
- **API/Backend**: Use Bash (curl) — Send requests, assert status + response fields
- **Library/Module**: Use Bash (bun/node REPL) — Import, call functions, compare output

---

## Execution Strategy

### Parallel Execution Waves

> Maximize throughput by grouping independent tasks into parallel waves.
> Target: 5-8 tasks per wave. Fewer than 3 per wave = under-splitting.

```
Wave 1 (Foundation — 项目搭建和基础设施):
├── Task 1: 初始化 Tauri + React 项目 [quick]
├── Task 2: 配置 TypeScript + Vite [quick]
├── Task 3: 配置 ESLint + Prettier [quick]
├── Task 4: 初始化 Git 仓库 + .gitignore [quick]
├── Task 5: 安装依赖 (node-cron, p-queue, better-sqlite3, @opencode-ai/sdk) [quick]
└── Task 6: 配置 Tauri 系统托盘 [quick]

Wave 2 (Data Layer — 数据模型和存储):
├── Task 7: 设计数据库 schema (tasks, executions) [quick]
├── Task 8: 实现 SQLite 数据库连接和初始化 (TDD) [unspecified-high]
├── Task 9: 实现 Task CRUD 操作 (TDD) [deep]
├── Task 10: 实现 Execution 记录操作 (TDD) [unspecified-high]
└── Task 11: 实现输出文件存储 (TDD) [unspecified-high]

Wave 3 (Backend Core — 调度和进程管理):
├── Task 12: 实现 Cron 表达式解析和验证 (TDD) [deep]
├── Task 13: 实现简单时间选择器逻辑 (TDD) [quick]
├── Task 14: 实现调度器核心 (node-cron) (TDD) [deep]
├── Task 15: 实现任务队列和并发控制 (p-queue) (TDD) [unspecified-high]
├── Task 16: 实现超时控制和进程杀死 (TDD) [unspecified-high]
└── Task 17: 实现 opencode CLI 集成 (TDD) [deep]

Wave 4 (UI Components — 前端组件):
├── Task 18: 实现任务列表组件 (TDD) [visual-engineering]
├── Task 19: 实现任务创建/编辑表单 (TDD) [visual-engineering]
├── Task 20: 实现 Cron 表达式输入组件 (TDD) [visual-engineering]
├── Task 21: 实现简单时间选择器组件 (TDD) [visual-engineering]
├── Task 22: 实现历史记录列表组件 (TDD) [visual-engineering]
├── Task 23: 实现输出查看器组件 (TDD) [visual-engineering]
└── Task 24: 实现托盘菜单和主窗口 [visual-engineering]

Wave 5 (Integration & Testing — 集成和测试):
├── Task 25: 集成前后端 - Tauri commands [deep]
├── Task 26: E2E 测试 - 任务创建流程 [unspecified-high]
├── Task 27: E2E 测试 - 任务执行和超时 [unspecified-high]
├── Task 28: E2E 测试 - 历史记录查看 [unspecified-high]
├── Task 29: 性能测试和优化 [deep]
└── Task 30: 文档编写和 Git 清理 [writing]

Wave FINAL (After ALL tasks — Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high + playwright)
└── Task F4: Scope fidelity check (deep)

Critical Path: Task 1-6 → Task 7-11 → Task 12-17 → Task 18-24 → Task 25-30 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 7 (Waves 1 & 2)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1-6 | — | 7-30 |
| 7-11 | 1-6 | 12-17, 18-24 |
| 12-17 | 7-11 | 25-30 |
| 18-24 | 7-11 | 25-30 |
| 25-30 | 12-17, 18-24 | F1-F4 |
| F1-F4 | 25-30 | — |

### Agent Dispatch Summary

- **Wave 1**: **6** — T1-T4 → `quick`, T5 → `quick`, T6 → `quick`
- **Wave 2**: **5** — T7 → `quick`, T8-T11 → `unspecified-high`, T9 → `deep`
- **Wave 3**: **6** — T12-T14 → `deep`, T15-T16 → `unspecified-high`, T17 → `deep`
- **Wave 4**: **7** — T18-T24 → `visual-engineering`
- **Wave 5**: **6** — T25, T29 → `deep`, T26-T28 → `unspecified-high`, T30 → `writing`
- **FINAL**: **4** — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

### Wave 1: Foundation (Project Setup)

- [x] 1. 初始化 Tauri + React 项目

  **What to do**:
  - 使用 `npm create tauri-app@latest` 创建项目
  - 选择 React + TypeScript 模板
  - 配置项目名称为 "mywork"
  - 验证项目可以成功运行 (`npm run tauri dev`)
  
  **Must NOT do**:
  - 不要修改默认的 Vite 配置（留给 Task 2）
  - 不要添加额外依赖（留给 Task 5）
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 标准的项目初始化，遵循官方模板
  - **Skills**: []
    - 无需特殊技能
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2-6)
  - **Blocks**: All subsequent tasks
  - **Blocked By**: None
  
  **References**:
  - Official docs: `https://tauri.app/v1/guides/getting-started/setup-vite/` - Tauri + Vite 设置指南
  
  **Acceptance Criteria**:
  - [ ] 项目目录结构正确 (`src-tauri/`, `src/`, `package.json`)
  - [ ] `npm run tauri dev` 成功启动应用
  - [ ] React 应用在 http://localhost:1420 运行
  
  **QA Scenarios**:
  ```
  Scenario: Project initialization succeeds
    Tool: Bash
    Preconditions: Empty directory
    Steps:
      1. Run `npm create tauri-app@latest mywork -- --template react-ts`
      2. `cd mywork && npm install`
      3. `npm run tauri dev --exit-on-panic` (wait 10s)
      4. Check process is running: `pgrep -f "mywork"`
    Expected Result: Process running, no errors in output
    Evidence: .sisyphus/evidence/task-01-init.log
  
  Scenario: React dev server starts
    Tool: Bash
    Steps:
      1. `npm run dev` (background)
      2. `curl http://localhost:1420` (after 5s)
    Expected Result: HTML response with React root element
    Evidence: .sisyphus/evidence/task-01-react-server.txt
  ```

  **Commit**: YES
  - Message: `chore: initialize Tauri + React project`
  - Files: All initial project files
  - Pre-commit: None

- [x] 2. 配置 TypeScript + Vite

  **What to do**:
  - 配置 `tsconfig.json`（strict mode, paths）
  - 配置 `vite.config.ts`（alias, build options）
  - 添加 `@types` 包（node, react, react-dom）
  - 验证 TypeScript 编译通过 (`tsc --noEmit`)
  
  **Must NOT do**:
  - 不要添加路径别名（如果会导致复杂性）
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 配置文件修改，简单快速
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None (独立配置)
  - **Blocked By**: Task 1
  
  **References**:
  - Vite docs: `https://vitejs.dev/config/` - Vite 配置选项
  
  **Acceptance Criteria**:
  - [ ] `tsc --noEmit` 无错误
  - [ ] `npm run build` 成功
  - [ ] Path alias `@/*` 可用
  
  **QA Scenarios**:
  ```
  Scenario: TypeScript compilation succeeds
    Tool: Bash
    Steps:
      1. `tsc --noEmit`
    Expected Result: Exit code 0, no errors
    Evidence: .sisyphus/evidence/task-02-tsc.log
  ```
  
  **Commit**: YES (with Task 1)

- [x] 3. 配置 ESLint + Prettier

  **What to do**:
  - 安装 `eslint`, `prettier`, `@typescript-eslint/*`
  - 配置 `.eslintrc.js`（TypeScript rules）
  - 配置 `.prettierrc`（格式化规则）
  - 添加 `lint` 和 `format` npm scripts
  - 验证 `npm run lint` 通过
  
  **Must NOT do**:
  - 不要添加过于严格的规则（影响开发效率）
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: Task 1
  
  **Acceptance Criteria**:
  - [ ] `npm run lint` 通过
  - [ ] `npm run format` 成功
  
  **QA Scenarios**:
  ```
  Scenario: Linting passes
    Tool: Bash
    Steps:
      1. `npm run lint`
    Expected Result: Exit code 0
    Evidence: .sisyphus/evidence/task-03-lint.log
  ```
  
  **Commit**: YES (with Task 1)

- [x] 4. 初始化 Git 仓库 + .gitignore

  **What to do**:
  - 初始化 git: `git init`
  - 创建 `.gitignore`，排除：
    - `node_modules/`
    - `dist/`
    - `target/` (Rust)
    - `*.log`
    - `.DS_Store`
    - `.sisyphus/evidence/` (QA evidence)
    - `*.db` (SQLite 数据库文件)
    - `outputs/` (任务输出文件)
  - 初始提交: `git add . && git commit -m "Initial commit"`
  
  **Must NOT do**:
  - 不要提交敏感文件
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`git-master`]
    - Git 操作最佳实践
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: Task 1
  
  **Acceptance Criteria**:
  - [ ] Git 仓库已初始化
  - [ ] `.gitignore` 包含所有必要规则
  - [ ] 初始提交存在
  
  **QA Scenarios**:
  ```
  Scenario: Git repository initialized
    Tool: Bash
    Steps:
      1. `git status`
    Expected Result: Shows "On branch main", initial commit exists
    Evidence: .sisyphus/evidence/task-04-git-status.txt
  
  Scenario: .gitignore excludes build artifacts
    Tool: Bash
    Steps:
      1. `mkdir -p dist target node_modules`
      2. `touch dist/test.js target/test node_modules/test`
      3. `git status --porcelain`
    Expected Result: No files from dist/, target/, node_modules/ shown
    Evidence: .sisyphus/evidence/task-04-gitignore.txt
  ```
  
  **Commit**: YES
  - Message: `chore: initialize git repository with .gitignore`
  - Files: `.gitignore`

- [x] 5. 安装依赖

  **What to do**:
  - 安装前端依赖:
    ```bash
    npm install -D @types/node
    ```
  - 在 `src-tauri/Cargo.toml` 添加 Rust 依赖:
    ```toml
    [dependencies]
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    tokio = { version = "1", features = ["full"] }
    tokio-cron-scheduler = "0.9"
    sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite"] }
    uuid = { version = "1.0", features = ["v4"] }
    chrono = { version = "0.4", features = ["serde"] }
    tauri = { version = "1.5", features = ["shell-open", "system-tray"] }
    ```
  - 验证所有依赖安装成功: `npm install && cargo build`
  
  **Must NOT do**:
  - 不要安装 Node.js 后端依赖（如 node-cron, better-sqlite3） - 后端纯 Rust
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Wave 2-5
  - **Blocked By**: Task 1
  
  **References**:
  - tokio-cron-scheduler: `https://docs.rs/tokio-cron-scheduler/`
  - sqlx: `https://docs.rs/sqlx/`
  - Tauri docs: `https://tauri.app/v1/guides/`
  
  **Acceptance Criteria**:
  - [ ] `Cargo.toml` 包含所有 Rust 依赖
  - [ ] `npm install` 无错误
  - [ ] `cargo build` 无错误
  
  **QA Scenarios**:
  ```
  Scenario: Rust dependencies compiled successfully
    Tool: Bash
    Steps:
      1. `cargo build --manifest-path=src-tauri/Cargo.toml`
    Expected Result: Exit code 0, no compilation errors
    Evidence: .sisyphus/evidence/task-05-cargo-build.log
  
  Scenario: All crates are available
    Tool: Bash
    Steps:
      1. `cargo tree --manifest-path=src-tauri/Cargo.toml | grep -E "tokio-cron-scheduler|sqlx|chrono"`
    Expected Result: Shows dependency tree with versions
    Evidence: .sisyphus/evidence/task-05-deps-tree.txt
  ```
  
  **Commit**: YES (with Task 1)

- [x] 6. 配置 Tauri 系统托盘

  **What to do**:
  - 在 `src-tauri/tauri.conf.json` 配置系统托盘:
    ```json
    {
      "tauri": {
        "systemTray": {
          "iconPath": "icons/icon.png",
          "iconAsTemplate": true
        },
        "windows": [
          {
            "title": "mywork",
            "width": 800,
            "height": 600,
            "resizable": true,
            "fullscreen": false
          }
        ]
      }
    }
    ```
  - 创建托盘图标（32x32 PNG）
  - 在 `src-tauri/src/main.rs` 添加托盘逻辑:
    ```rust
    use tauri::{Manager, SystemTray};
    
    fn main() {
      let tray = SystemTray::new();
      
      tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| {
          // Handle tray events
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    }
    ```
  - 验证托盘图标显示
  
  **Must NOT do**:
  - 不要添加复杂的托盘菜单（留给后续）
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: Task 1
  
  **References**:
  - Tauri tray docs: `https://tauri.app/v1/guides/features/system-tray/`
  
  **Acceptance Criteria**:
  - [ ] 托盘图标在 macOS 菜单栏可见
  - [ ] 点击图标可以打开/关闭窗口
  
  **QA Scenarios**:
  ```
  Scenario: System tray icon appears in menu bar
    Tool: interactive_bash (tmux)
    Steps:
      1. Start app: `npm run tauri dev &` (background)
      2. Wait 15 seconds for app to initialize
      3. Check tray via osascript:
         `osascript -e 'tell application "System Events" to get name of every process whose background only is true' | grep -q "mywork"`
      4. Take screenshot: `screencapture -x .sisyphus/evidence/task-06-menubar.png`
    Expected Result: osascript finds "mywork" process, screenshot shows tray icon
    Evidence: .sisyphus/evidence/task-06-menubar.png
  
  Scenario: Clicking tray icon opens window
    Tool: interactive_bash (tmux)
    Steps:
      1. App running in background
      2. Simulate click via osascript:
         `osascript -e 'tell application "System Events" to click menu bar item 1 of menu bar 2'`
      3. Check window visible: `osascript -e 'tell application "mywork" to get visible of window 1'`
    Expected Result: Window visible = true
    Evidence: .sisyphus/evidence/task-06-window-open.txt
  ```
  
  **Commit**: YES
  - Message: `feat: add system tray support`
  - Files: `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/icons/`

---

### Wave 2: Data Layer (Storage & Models)

- [x] 7. 设计数据库 schema

  **What to do**:
  - 设计并文档化数据库 schema:
    ```sql
    CREATE TABLE tasks (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      prompt TEXT NOT NULL,
      cron_expression TEXT,
      simple_schedule TEXT, -- JSON: {"type": "interval", "value": 5, "unit": "minutes"}
      enabled INTEGER DEFAULT 1,
      timeout_seconds INTEGER DEFAULT 300,
      skip_if_running INTEGER DEFAULT 1,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL
    );
    
    CREATE TABLE executions (
      id TEXT PRIMARY KEY,
      task_id TEXT NOT NULL,
      session_id TEXT, -- opencode session ID
      status TEXT NOT NULL, -- pending, running, success, failed, timeout, skipped
      started_at TEXT NOT NULL,
      finished_at TEXT,
      output_file TEXT, -- Path to output file
      error_message TEXT,
      FOREIGN KEY (task_id) REFERENCES tasks(id)
    );
    
    CREATE INDEX idx_executions_task_id ON executions(task_id);
    CREATE INDEX idx_executions_started_at ON executions(started_at);
    ```
  - 创建 `src-tauri/src/db/schema.sql` 文件
  - 添加 schema 文档到 README
  
  **Must NOT do**:
  - 不要实现数据库操作（留给 Task 8-11）
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 8-11
  - **Blocked By**: Task 1-6
  
  **Acceptance Criteria**:
  - [ ] `schema.sql` 文件存在且 SQL 语法正确
  - [ ] README 包含 schema 说明
  
  **QA Scenarios**:
  ```
  Scenario: Schema is valid SQL
    Tool: Bash
    Steps:
      1. `sqlite3 :memory: < src-tauri/src/db/schema.sql`
    Expected Result: Exit code 0, no syntax errors
    Evidence: .sisyphus/evidence/task-07-schema-valid.log
  ```
  
  **Commit**: YES
  - Message: `docs: add database schema design`
  - Files: `src-tauri/src/db/schema.sql`, `README.md`

- [x] 8-11. 数据层实现 (数据库、Task CRUD、Execution CRUD、输出存储)

  **What to do**: 实现完整的数据持久化层，包括数据库连接、任务和执行记录的 CRUD 操作、输出文件存储。使用 TDD 开发。
  
  **详细任务**:
  8. SQLite 数据库连接和初始化 (TDD)
  9. Task CRUD 操作 (TDD)
  10. Execution CRUD 操作 (TDD)
  11. 输出文件存储 (TDD)
  
  **Must NOT do**:
  - 不要在数据库中存储完整输出（仅存路径）
  - 不要跳过测试
  
  **Recommended Agent Profile**:
  - **Category**: `deep` (T9), `unspecified-high` (T8, T10, T11)
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Wave 3-5
  - **Blocked By**: Task 1-6, Task 7
  
  **Acceptance Criteria**:
  - [ ] 所有数据层测试通过 (`cargo test db::`)
  - [ ] 数据库文件正确创建在 app data directory
  - [ ] CRUD 操作全部实现且测试覆盖
  - [ ] `.gitignore` 包含 `*.db` 和 `outputs/`
  
  **QA Scenarios**:
  ```
  Scenario: All database tests pass
    Tool: Bash
    Steps:
      1. `cargo test db::`
    Expected Result: All tests pass, 0 failures
    Evidence: .sisyphus/evidence/task-08-11-db-tests.log
  
  Scenario: Database file created in correct location
    Tool: interactive_bash
    Steps:
      1. Start app: `npm run tauri dev`
      2. Create a task via Tauri command
      3. Check database exists: `ls ~/Library/Application\ Support/com.mywork.app/`
    Expected Result: mywork.db file exists
    Evidence: .sisyphus/evidence/task-08-11-db-file.txt
  ```
  
  **Commit**: YES
  - Message: `feat: implement data layer with SQLite and TDD`
  - Files: `src-tauri/src/db/**/*.rs`, `src-tauri/src/models/**/*.rs`, `src-tauri/src/storage/**/*.rs`
  - Pre-commit: `cargo test`

---

### Wave 3: Backend Core (Scheduler & Process Management)

- [x] 12. 实现 Cron 表达式解析和验证 (TDD)
  - **What**: 使用 `tokio-cron-scheduler` 的 JobScheduler 解析和验证 cron 表达式
  - **TDD**: 测试用例覆盖合法/非法 cron 表达式
  - **Category**: `deep`
  - **Commit**: `feat: implement cron expression parser`
  - **QA Scenarios**:
    ```
    Scenario: Valid cron expressions are accepted
      Tool: Bash
      Steps:
        1. `cargo test cron_parser::test_valid_cron -- --nocapture`
      Expected Result: All valid expressions ("*/5 * * * *", "0 9 * * 1-5") pass
      Evidence: .sisyphus/evidence/task-12-valid-cron.log
    
    Scenario: Invalid cron expressions are rejected
      Tool: Bash
      Steps:
        1. `cargo test cron_parser::test_invalid_cron -- --nocapture`
      Expected Result: Invalid expressions ("invalid", "25 * * * * *") return errors
      Evidence: .sisyphus/evidence/task-12-invalid-cron.log
    ```

- [x] 13. 实现简单时间选择器逻辑 (TDD)
  - **What**: 解析 JSON 格式的简单调度规则（每N分钟、每天HH:MM）
  - **TDD**: 测试各种时间格式
  - **Category**: `quick`
  - **Commit**: `feat: implement simple time schedule parser`
  - **QA Scenarios**:
    ```
    Scenario: Simple interval schedules parse correctly
      Tool: Bash
      Steps:
        1. `cargo test simple_schedule::test_interval -- --nocapture`
      Expected Result: JSON {"type":"interval","value":5,"unit":"minutes"} parses to cron
      Evidence: .sisyphus/evidence/task-13-interval.log
    ```

- [x] 14. 实现调度器核心 (tokio-cron-scheduler) (TDD)
  - **What**: 使用 `tokio-cron-scheduler` 实现定时触发
  - **TDD**: 使用 tokio::time::pause 测试调度逻辑
  - **Category**: `deep`
  - **Commit**: `feat: implement job scheduler with cron`
  - **QA Scenarios**:
    ```
    Scenario: Scheduler triggers jobs at correct times
      Tool: Bash
      Steps:
        1. `cargo test scheduler::test_job_trigger -- --nocapture`
      Expected Result: Job scheduled for "*/1 * * * *" fires within 60 seconds
      Evidence: .sisyphus/evidence/task-14-scheduler.log
    ```

- [x] 15. 实现任务队列和并发控制 (TDD)
  - **What**: 使用 Tokio Semaphore 实现任务队列，确保 skip_if_running 生效
  - **TDD**: 测试并发场景
  - **Category**: `unspecified-high`
  - **Commit**: `feat: implement task queue with concurrency control`
  - **QA Scenarios**:
    ```
    Scenario: Running task prevents duplicate execution
      Tool: Bash
      Steps:
        1. `cargo test task_queue::test_skip_if_running -- --nocapture`
      Expected Result: Second trigger while first running returns skipped status
      Evidence: .sisyphus/evidence/task-15-skip-if-running.log
    ```

- [x] 16. 实现超时控制和进程杀死 (TDD)
  - **What**: 使用 `tokio::time::timeout` + `kill` 实现超时控制
  - **TDD**: 测试超时场景
  - **Category**: `unspecified-high`
  - **Commit**: `feat: implement timeout and process killing`
  - **QA Scenarios**:
    ```
    Scenario: Long-running task is killed after timeout
      Tool: Bash
      Steps:
        1. `cargo test timeout::test_kill_process -- --nocapture`
      Expected Result: Task exceeding 5s timeout is killed, exit code recorded
      Evidence: .sisyphus/evidence/task-16-timeout.log
    ```

- [x] 17. 实现 opencode CLI 集成 (TDD)
  - **What**: 通过 Tauri shell 执行 opencode CLI，管理 session 生命周期
  - **TDD**: Mock CLI 测试
  - **Category**: `deep`
  - **References**: `/Users/huangli/gh/opencode/packages/sdk/js/example/example.ts`
  - **Commit**: `feat: integrate opencode CLI`
  - **QA Scenarios**:
    ```
    Scenario: opencode CLI executes successfully
      Tool: Bash
      Steps:
        1. `cargo test opencode::test_cli_execution -- --nocapture`
      Expected Result: Mock opencode command runs, returns output
      Evidence: .sisyphus/evidence/task-17-opencode-cli.log
    ```

---

### Wave 4: UI Components (React Frontend)

- [ ] 18. 实现任务列表组件 (TDD)
  - **What**: 创建 `TaskList.tsx` 显示所有任务，支持启用/禁用、删除
  - **TDD**: React Testing Library 测试组件渲染和交互
  - **Category**: `visual-engineering`
  - **Design**: 原生 macOS 风格（SF Pro 字体、系统色彩）
  - **Commit**: `feat: implement task list component`
  - **QA Scenarios**:
    ```
    Scenario: Task list renders with correct macOS styling
      Tool: Bash + Playwright
      Steps:
        1. `npm run tauri dev &` (start app)
        2. Wait 10s
        3. `npx playwright test tests/task-list.spec.ts`
      Expected Result: Component uses SF Pro font, system colors, no generic styling
      Evidence: .sisyphus/evidence/task-18-tasklist-screenshot.png
    
    Scenario: User can toggle task enable/disable
      Tool: interactive_bash
      Steps:
        1. App running, task list visible
        2. Click toggle button on first task
        3. Check database: `sqlite3 ~/Library/Application\ Support/com.mywork.app/mywork.db "SELECT enabled FROM tasks LIMIT 1"`
      Expected Result: Database shows enabled=0 after toggle
      Evidence: .sisyphus/evidence/task-18-toggle.txt
    ```

- [ ] 19. 实现任务创建/编辑表单 (TDD)
  - **What**: 创建 `TaskForm.tsx` 支持 prompt、cron/schedule、timeout 等字段
  - **TDD**: 测试表单验证和提交
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement task form component`
  - **QA Scenarios**:
    ```
    Scenario: Form validates required fields
      Tool: Playwright
      Steps:
        1. Navigate to "New Task" form
        2. Submit with empty fields
        3. Check error messages visible
      Expected Result: Shows "Name required", "Prompt required" errors
      Evidence: .sisyphus/evidence/task-19-form-validation.png
    
    Scenario: Form creates task in database
      Tool: interactive_bash
      Steps:
        1. Fill form: name="Test", prompt="Hello", schedule="*/5 * * * *"
        2. Submit form
        3. Query: `sqlite3 ~/Library/Application\ Support/com.mywork.app/mywork.db "SELECT name FROM tasks WHERE name='Test'"`
      Expected Result: Returns "Test"
      Evidence: .sisyphus/evidence/task-19-create-task.txt
    ```

- [ ] 20. 实现 Cron 表达式输入组件 (TDD)
  - **What**: 创建 `CronInput.tsx` 支持 cron 表达式输入和预览
  - **TDD**: 测试输入验证
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement cron input component`
  - **QA Scenarios**:
    ```
    Scenario: Valid cron shows next run time
      Tool: Playwright
      Steps:
        1. Type "*/5 * * * *" in cron input
        2. Check preview text
      Expected Result: Shows "Next run: in ~5 minutes"
      Evidence: .sisyphus/evidence/task-20-cron-preview.png
    
    Scenario: Invalid cron shows error
      Tool: Playwright
      Steps:
        1. Type "invalid" in cron input
        2. Check error state
      Expected Result: Shows "Invalid cron expression"
      Evidence: .sisyphus/evidence/task-20-cron-error.png
    ```

- [ ] 21. 实现简单时间选择器组件 (TDD)
  - **What**: 创建 `SimpleScheduleInput.tsx` 支持下拉选择时间间隔
  - **TDD**: 测试选择器交互
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement simple schedule selector`
  - **QA Scenarios**:
    ```
    Scenario: User can select interval from dropdown
      Tool: Playwright
      Steps:
        1. Click schedule type dropdown, select "Interval"
        2. Select "Every 5 minutes"
        3. Check generated schedule JSON
      Expected Result: JSON shows {"type":"interval","value":5,"unit":"minutes"}
      Evidence: .sisyphus/evidence/task-21-interval-select.txt
    ```

- [ ] 22. 实现历史记录列表组件 (TDD)
  - **What**: 创建 `ExecutionHistory.tsx` 显示任务执行历史
  - **TDD**: 测试列表渲染
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement execution history component`
  - **QA Scenarios**:
    ```
    Scenario: History shows all executions for task
      Tool: interactive_bash
      Steps:
        1. Create task, trigger execution
        2. Navigate to task details, click "History" tab
        3. Check list count
      Expected Result: Shows at least 1 execution record
      Evidence: .sisyphus/evidence/task-22-history-list.png
    ```

- [ ] 23. 实现输出查看器组件 (TDD)
  - **What**: 创建 `OutputViewer.tsx` 显示任务输出，支持 Markdown 渲染
  - **TDD**: 测试文本和 Markdown 显示
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement output viewer component`
  - **QA Scenarios**:
    ```
    Scenario: Output viewer displays markdown correctly
      Tool: Playwright
      Steps:
        1. Navigate to execution with markdown output
        2. Check heading rendered with <h1> tag
        3. Check code block syntax highlighted
      Expected Result: Markdown rendered as HTML, not raw text
      Evidence: .sisyphus/evidence/task-23-markdown-viewer.png
    ```

- [ ] 24. 实现托盘菜单和主窗口
  - **What**: 集成托盘菜单（显示任务数量、Open/Quit）和主窗口布局
  - **TDD**: 测试菜单交互
  - **Category**: `visual-engineering`
  - **Commit**: `feat: implement tray menu and main window`
  - **QA Scenarios**:
    ```
    Scenario: Tray menu shows task count
      Tool: interactive_bash
      Steps:
        1. App running, create 3 tasks
        2. Check tray menu item: `osascript -e 'tell application "System Events" to get title of menu bar item 1 of menu bar 2'`
      Expected Result: Menu shows "3 Tasks"
      Evidence: .sisyphus/evidence/task-24-tray-count.txt
    
    Scenario: Clicking tray icon opens window
      Tool: interactive_bash
      Steps:
        1. Close window
        2. Click tray icon via osascript
        3. Check window visible
      Expected Result: Window becomes visible
      Evidence: .sisyphus/evidence/task-24-tray-click.txt
    ```

---

### Wave 5: Integration & Testing

- [ ] 25. 集成前后端 - Tauri commands
  - **What**: 实现所有 Tauri commands 连接前端和后端
  - **TDD**: 测试 command 调用
  - **Category**: `deep`
  - **Commit**: `feat: integrate frontend and backend`
  - **QA Scenarios**:
    ```
    Scenario: Frontend can call backend commands
      Tool: interactive_bash
      Steps:
        1. App running
        2. Call command via tauri API: `window.__TAURI__.tauri.invoke('get_tasks')`
        3. Check response in browser console
      Expected Result: Returns JSON array of tasks
      Evidence: .sisyphus/evidence/task-25-tauri-commands.txt
    ```

- [ ] 26. E2E 测试 - 任务创建流程
  - **What**: Playwright 测试：创建任务 → 验证保存 → 检查列表
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for task creation`
  - **QA Scenarios**:
    ```
    Scenario: Complete task creation flow
      Tool: Playwright
      Steps:
        1. Navigate to "New Task"
        2. Fill: name="E2E Test", prompt="Test prompt", schedule="*/5 * * * *"
        3. Submit
        4. Check task list contains "E2E Test"
        5. Check database has task
      Expected Result: Task visible in UI and database
      Evidence: .sisyphus/evidence/task-26-e2e-create.png
    ```

- [ ] 27. E2E 测试 - 任务执行和超时
  - **What**: Playwright 测试：触发任务 → 验证执行 → 测试超时
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for task execution`
  - **QA Scenarios**:
    ```
    Scenario: Task executes and shows output
      Tool: interactive_bash + Playwright
      Steps:
        1. Create task with immediate trigger
        2. Wait 5 seconds
        3. Navigate to execution history
        4. Check latest execution status = "success"
        5. Click to view output
      Expected Result: Output visible, status=success
      Evidence: .sisyphus/evidence/task-27-e2e-exec.png
    
    Scenario: Task timeout kills process
      Tool: interactive_bash
      Steps:
        1. Create task with 2s timeout, slow prompt
        2. Wait 3 seconds
        3. Check execution status = "timeout"
      Expected Result: Status=timeout, process killed
      Evidence: .sisyphus/evidence/task-27-timeout.txt
    ```

- [ ] 28. E2E 测试 - 历史记录查看
  - **What**: Playwright 测试：查看历史 → 验证输出显示
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for history view`
  - **QA Scenarios**:
    ```
    Scenario: User can view execution history
      Tool: Playwright
      Steps:
        1. Task has 3+ executions
        2. Navigate to task details
        3. Click "History" tab
        4. Check all 3 executions listed
      Expected Result: Shows 3 records with timestamps
      Evidence: .sisyphus/evidence/task-28-e2e-history.png
    ```

- [ ] 29. 性能测试和优化
  - **What**: 测试内存占用（<100MB）、启动时间（<1s）、数据库性能
  - **Category**: `deep`
  - **Commit**: `perf: optimize application performance`
  - **QA Scenarios**:
    ```
    Scenario: Memory usage under limit
      Tool: Bash
      Steps:
        1. Start app, wait 30s
        2. Check memory: `ps -o rss= -p $(pgrep -f mywork) | awk '{print $1/1024 "MB"}'`
      Expected Result: < 100MB
      Evidence: .sisyphus/evidence/task-29-memory.txt
    
    Scenario: Startup time under limit
      Tool: Bash
      Steps:
        1. `time npm run tauri dev` until window visible
      Expected Result: < 1 second to window show
      Evidence: .sisyphus/evidence/task-29-startup.txt
    ```

- [ ] 30. 文档编写和 Git 清理
  - **What**: 完善 README（安装、使用、架构）、检查 .gitignore、清理无用文件
  - **Category**: `writing`
  - **Commit**: `docs: add comprehensive README and cleanup`
  - **QA Scenarios**:
    ```
    Scenario: README contains all sections
      Tool: Bash
      Steps:
        1. Check README has: Installation, Usage, Architecture, Development
        2. All code blocks are valid
      Expected Result: All sections present, no broken links
      Evidence: .sisyphus/evidence/task-30-readme-check.txt
    
    Scenario: Gitignore excludes all artifacts
      Tool: Bash
      Steps:
        1. Create test files in dist/, target/, outputs/
        2. `git status --porcelain`
      Expected Result: No files from ignored directories shown
      Evidence: .sisyphus/evidence/task-30-gitignore-verify.txt
    ```

---

### Wave 5: Integration & Testing

- [ ] 25. 集成前后端 - Tauri commands
  - **What**: 实现所有 Tauri commands 连接前端和后端
  - **TDD**: 测试 command 调用
  - **Category**: `deep`
  - **Commit**: `feat: integrate frontend and backend`
  - **QA Scenarios**:
    ```
    Scenario: Frontend can call backend commands
      Tool: Playwright
      Steps:
        1. Start app
        2. Call `invoke('get_tasks')` from frontend
        3. Verify response is array
      Expected Result: Returns task list from database
      Evidence: .sisyphus/evidence/task-25-commands.log
    ```

- [ ] 26. E2E 测试 - 任务创建流程
  - **What**: Playwright 测试：创建任务 → 验证保存 → 检查列表
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for task creation`
  - **QA Scenarios**:
    ```
    Scenario: User creates task end-to-end
      Tool: Playwright
      Steps:
        1. Open app
        2. Click "New Task"
        3. Fill form: name="E2E Test", prompt="echo hello"
        4. Set schedule="*/1 * * * *"
        5. Save
        6. Check task appears in list
        7. Verify database: `sqlite3 ... "SELECT name FROM tasks WHERE name='E2E Test'"`
      Expected Result: Task created, appears in UI and database
      Evidence: .sisyphus/evidence/task-26-e2e-create.png
    ```

- [ ] 27. E2E 测试 - 任务执行和超时
  - **What**: Playwright 测试：触发任务 → 验证执行 → 测试超时
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for task execution`
  - **QA Scenarios**:
    ```
    Scenario: Task executes on schedule
      Tool: interactive_bash
      Steps:
        1. Create task with 1-minute interval
        2. Wait 65 seconds
        3. Check execution history
      Expected Result: Execution record appears with "success" status
      Evidence: .sisyphus/evidence/task-27-execution.txt
    
    Scenario: Task timeout kills process
      Tool: interactive_bash
      Steps:
        1. Create task with 5s timeout, prompt="sleep 30"
        2. Wait 10 seconds
        3. Check execution status
      Expected Result: Status is "timeout", process killed
      Evidence: .sisyphus/evidence/task-27-timeout.txt
    ```

- [ ] 28. E2E 测试 - 历史记录查看
  - **What**: Playwright 测试：查看历史 → 验证输出显示
  - **Category**: `unspecified-high`
  - **Commit**: `test: add E2E test for history view`
  - **QA Scenarios**:
    ```
    Scenario: User views execution history
      Tool: Playwright
      Steps:
        1. Task has executed at least once
        2. Click task to view history
        3. Click specific execution
        4. View output
      Expected Result: Output displayed correctly (text/markdown)
      Evidence: .sisyphus/evidence/task-28-history-view.png
    ```

- [ ] 29. 性能测试和优化
  - **What**: 测试内存占用（<100MB）、启动时间（<1s）、数据库性能
  - **Category**: `deep`
  - **Commit**: `perf: optimize application performance`
  - **QA Scenarios**:
    ```
    Scenario: Memory usage under 100MB
      Tool: Bash
      Steps:
        1. Start app
        2. Wait 30s
        3. Check memory: `ps -o rss= -p $(pgrep -f mywork)`
      Expected Result: RSS < 100000 (100MB in KB)
      Evidence: .sisyphus/evidence/task-29-memory.txt
    
    Scenario: App startup under 1 second
      Tool: Bash
      Steps:
        1. `time npm run tauri dev &`
        2. Measure until window visible
      Expected Result: <1.0s user+sys time
      Evidence: .sisyphus/evidence/task-29-startup-time.txt
    ```

- [ ] 30. 文档编写和 Git 清理
  - **What**: 完善 README（安装、使用、架构）、检查 .gitignore、清理无用文件
  - **Category**: `writing`
  - **Commit**: `docs: add comprehensive README and cleanup`
  - **QA Scenarios**:
    ```
    Scenario: README contains required sections
      Tool: Bash
      Steps:
        1. `grep -E "## Installation|## Usage|## Architecture" README.md`
      Expected Result: All 3 sections found
      Evidence: .sisyphus/evidence/task-30-readme-check.txt
    
    Scenario: No build artifacts in git status
      Tool: Bash
      Steps:
        1. `git status --porcelain`
        2. Verify no .db, .log, node_modules, target files
      Expected Result: Empty output (clean repo)
      Evidence: .sisyphus/evidence/task-30-git-clean.txt
    ```

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo test` + `npm test` + `cargo clippy` + `npm run lint`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill)
  Start from clean state. Execute EVERY QA scenario from EVERY task. Test cross-task integration. Test edge cases: empty state, invalid input, rapid actions. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Detect cross-task contamination.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **Wave 1 Complete**: `feat: initialize project with Tauri + React + dependencies`
  - Files: All project setup files
  - Pre-commit: `npm run lint && cargo clippy`

- **Wave 2 Complete**: `feat: implement data layer with SQLite`
  - Files: `src-tauri/src/db/**/*.rs`, `src-tauri/src/models/**/*.rs`
  - Pre-commit: `cargo test`

- **Wave 3 Complete**: `feat: implement scheduler and opencode integration`
  - Files: `src-tauri/src/scheduler/**/*.rs`, `src-tauri/src/opencode/**/*.rs`
  - Pre-commit: `cargo test`

- **Wave 4 Complete**: `feat: implement UI components`
  - Files: `src/**/*.tsx`, `src/**/*.css`
  - Pre-commit: `npm test`

- **Wave 5 Complete**: `feat: integrate frontend and backend, add E2E tests`
  - Files: `src-tauri/src/cmd/**/*.rs`, `tests/**/*.ts`
  - Pre-commit: `npm run test:e2e`

- **Final**: `docs: add README and cleanup`
  - Files: `README.md`, `.gitignore`
  - Pre-commit: All tests

---

## Success Criteria

### Verification Commands
```bash
# Build
npm run tauri build  # Expected: dist/bundle/macos/mywork.app

# Tests
cargo test           # Expected: All Rust tests pass
npm test            # Expected: All React tests pass
npm run test:e2e    # Expected: All E2E tests pass

# Code Quality
cargo clippy        # Expected: No warnings
npm run lint        # Expected: No errors

# Functionality
# 1. App launches and shows tray icon
# 2. Can create task "Test" with cron "*/1 * * * *"
# 3. Task executes within 60 seconds
# 4. Can view execution output
# 5. History shows execution record
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (cargo test + npm test + e2e)
- [ ] Code coverage ≥ 80%
- [ ] App builds successfully
- [ ] Tray icon visible and functional
- [ ] All QA scenarios pass
- [ ] README documentation complete
- [ ] Git repository clean (proper .gitignore)

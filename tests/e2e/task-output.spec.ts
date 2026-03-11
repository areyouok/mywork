import { test, expect } from '@playwright/test';

async function createTask(page: import('@playwright/test').Page, name: string, prompt: string) {
  await page.click('button:has-text("+ New Task")');
  await expect(page.locator('h2:has-text("Create New Task")')).toBeVisible();
  await page.fill('#task-name', name);
  await page.fill('#prompt', prompt);
  await page.check('input[value="cron"]');
  await page.getByLabel('Cron Expression *').fill('0 9 * * *');
  await page.click('button:has-text("Create Task")');

  await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
  await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });
}

async function runTaskAndOpenHistory(page: import('@playwright/test').Page, taskName: string) {
  await page.click(`.sidebar-task-item:has-text("${taskName}")`);
  await page.click('button:has-text("Run")');
  await page.click('button:has-text("History")');
  await expect(page.locator('.execution-item')).toBeVisible();
}

test.describe('Task Output E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const state = {
        tasks: [] as Array<{
          id: string;
          name: string;
          prompt: string;
          cron_expression: string | null;
          simple_schedule: string | null;
          enabled: number;
          timeout_seconds: number;
          skip_if_running: number;
          created_at: string;
          updated_at: string;
        }>,
        executionsByTaskId: {} as Record<string, any[]>,
        executionPollCount: {} as Record<string, number>,
      };

      const buildOutput = (status: string) => {
        const ansiHeader =
          status === 'running'
            ? '\u001b[33mRUNNING\u001b[0m\n\u001b[31mERROR\u001b[0m\n'
            : '\u001b[32mSUCCESS\u001b[0m\n\u001b[31mERROR\u001b[0m\n';
        const longLines = Array.from({ length: 240 })
          .map((_, i) => `line-${i + 1} lorem ipsum dolor sit amet`)
          .join('\n');
        return `${ansiHeader}${longLines}`;
      };

      const nowIso = () => new Date().toISOString();

      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          if (cmd === 'get_tasks') return state.tasks;
          if (cmd === 'reload_scheduler') return 'scheduler reloaded';
          if (cmd === 'get_scheduler_status') return 'running';
          if (cmd === 'start_scheduler') return 'scheduler started';

          if (cmd === 'get_running_executions') {
            return Object.values(state.executionsByTaskId)
              .flat()
              .filter((e) => e.status === 'running')
              .map((e) => e.task_id);
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            const task = {
              id: `task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled ?? 1,
              timeout_seconds: newTask.timeout_seconds ?? 300,
              skip_if_running: newTask.skip_if_running ?? 0,
              created_at: nowIso(),
              updated_at: nowIso(),
            };
            state.tasks.push(task);
            return task;
          }

          if (cmd === 'run_task') {
            const taskId = args?.taskId;
            const execution = {
              id: `exec-${Date.now()}`,
              task_id: taskId,
              session_id: 'session-task-output-e2e',
              status: 'running',
              started_at: nowIso(),
              finished_at: null,
              output_file: `/tmp/${taskId}.txt`,
              error_message: null,
            };
            state.executionsByTaskId[taskId] = [execution];
            state.executionPollCount[taskId] = 0;
            return execution.id;
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const executions = state.executionsByTaskId[taskId] ?? [];
            state.executionPollCount[taskId] = (state.executionPollCount[taskId] ?? 0) + 1;

            if (executions.length > 0 && state.executionPollCount[taskId] >= 3) {
              executions[0] = {
                ...executions[0],
                status: 'success',
                finished_at: nowIso(),
              };
              state.executionsByTaskId[taskId] = executions;
            }

            return state.executionsByTaskId[taskId] ?? [];
          }

          if (cmd === 'get_output') {
            const executionId = args?.executionId;
            const execution = Object.values(state.executionsByTaskId)
              .flat()
              .find((e) => e.id === executionId);
            return buildOutput(execution?.status ?? 'running');
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');
  });

  test('完整流程：创建任务 -> 执行 -> 实时查看 -> 完成', async ({ page }) => {
    await createTask(page, 'Task Output Full Flow', 'echo streaming');
    await runTaskAndOpenHistory(page, 'Task Output Full Flow');

    await expect(page.locator('.execution-status.status-running')).toBeVisible();
    await expect(page.locator('.execution-duration')).toContainText('Running...');

    await page.click('.execution-item.clickable');
    await expect(page.locator('.output-viewer .ansi-renderer')).toBeVisible();

    await page.click('button:has-text("Back to History")');
    await page.click('button:has-text("Back to Task")');
    await page.click('button:has-text("History")');

    await expect(page.locator('.execution-status.status-success')).toBeVisible();
  });

  test('ANSI 颜色渲染验证（.ansi-renderer）', async ({ page }) => {
    await createTask(page, 'Task Output ANSI', 'echo ansi');
    await runTaskAndOpenHistory(page, 'Task Output ANSI');

    await page.click('.execution-item.clickable');

    const ansiRenderer = page.locator('.ansi-renderer');
    await expect(ansiRenderer).toBeVisible();
    await expect(ansiRenderer).toContainText('ERROR');

    const coloredSegment = ansiRenderer.locator('span[style*="color"]');
    await expect(coloredSegment.first()).toBeVisible();
  });

  test('双滚动条验证（应为单滚动条）', async ({ page }) => {
    await createTask(page, 'Task Output Scroll', 'echo scroll');
    await runTaskAndOpenHistory(page, 'Task Output Scroll');
    await page.click('.execution-item.clickable');

    const outerOverflowY = await page
      .locator('.output-viewer')
      .evaluate((el) => window.getComputedStyle(el).overflowY);
    const innerOverflowY = await page
      .locator('.output-viewer-content')
      .evaluate((el) => window.getComputedStyle(el).overflowY);

    expect(outerOverflowY).not.toBe('auto');
    expect(outerOverflowY).not.toBe('scroll');
    expect(innerOverflowY).toBe('auto');

    const contentScrollInfo = await page.locator('.output-viewer-content').evaluate((el) => ({
      scrollHeight: el.scrollHeight,
      clientHeight: el.clientHeight,
    }));
    expect(contentScrollInfo.scrollHeight).toBeGreaterThan(contentScrollInfo.clientHeight);
  });

  test('状态徽章验证（运行中到完成）', async ({ page }) => {
    await createTask(page, 'Task Output Badge', 'echo badge');
    await runTaskAndOpenHistory(page, 'Task Output Badge');

    const runningBadge = page.locator('.execution-status.status-running');
    await expect(runningBadge).toBeVisible();
    await expect(runningBadge).toContainText('running');

    await page.click('button:has-text("Back to Task")');
    await page.click('button:has-text("History")');

    const finishedBadge = page.locator('.execution-status.status-success');
    await expect(finishedBadge).toBeVisible();
    await expect(finishedBadge).toContainText('success');
  });
});

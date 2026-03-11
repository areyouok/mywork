import { test, expect } from '@playwright/test';

test.describe('History View E2E', () => {
  test('should display execution history with clickable output items', async ({ page }) => {
    await page.addInitScript(() => {
      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);

          if (cmd === 'get_tasks') {
            return [];
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            return {
              id: `test-task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              skip_if_running: newTask.skip_if_running || 0,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();
            const startedAt = new Date(now.getTime() - 5 * 60 * 1000);
            const finishedAt = new Date(now.getTime() - 4 * 60 * 1000);

            return [
              {
                id: `exec-${Date.now()}`,
                task_id: taskId,
                session_id: 'session-123',
                status: 'success',
                started_at: startedAt.toISOString(),
                finished_at: finishedAt.toISOString(),
                output_file: '/path/to/output.txt',
                error_message: null,
              },
            ];
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');
    await page.fill('#task-name', 'Task With Output');
    await page.fill('#prompt', 'echo hello');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Output")');
    await page.click('button:has-text("History")');

    await expect(page.locator('.execution-history')).toBeVisible();
    await expect(page.locator('.execution-item')).toBeVisible();

    const executionItem = page.locator('.execution-item.clickable');
    await expect(executionItem).toBeVisible();
    await expect(executionItem).toHaveAttribute('tabindex', '0');

    await page.screenshot({ path: '.sisyphus/evidence/task-28-clickable-execution.png' });
  });

  test('should not allow clicking execution items without output', async ({ page }) => {
    await page.addInitScript(() => {
      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);

          if (cmd === 'get_tasks') {
            return [];
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            return {
              id: `test-task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              skip_if_running: newTask.skip_if_running || 0,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();
            const startedAt = new Date(now.getTime() - 2 * 60 * 1000);
            const finishedAt = new Date(now.getTime() - 1 * 60 * 1000);

            return [
              {
                id: `exec-${Date.now()}`,
                task_id: taskId,
                session_id: 'session-456',
                status: 'failed',
                started_at: startedAt.toISOString(),
                finished_at: finishedAt.toISOString(),
                output_file: null,
                error_message: 'Command failed with exit code 1',
              },
            ];
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');
    await page.fill('#task-name', 'Task Without Output');
    await page.fill('#prompt', 'exit 1');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 10 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task Without Output")');
    await page.click('button:has-text("History")');

    await expect(page.locator('.execution-item')).toBeVisible();

    const nonClickableItem = page.locator('.execution-item:not(.clickable)');
    await expect(nonClickableItem).toBeVisible();
    await expect(nonClickableItem).not.toHaveAttribute('tabindex', '0');

    await page.screenshot({ path: '.sisyphus/evidence/task-28-non-clickable-execution.png' });
  });

  test('should display execution history with multiple executions', async ({ page }) => {
    await page.addInitScript(() => {
      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);

          if (cmd === 'get_tasks') {
            return [];
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            return {
              id: `test-task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              skip_if_running: newTask.skip_if_running || 0,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();

            return [
              {
                id: 'exec-3',
                task_id: taskId,
                session_id: 'session-3',
                status: 'success',
                started_at: new Date(now.getTime() - 5 * 60 * 1000).toISOString(),
                finished_at: new Date(now.getTime() - 4 * 60 * 1000).toISOString(),
                output_file: '/path/to/output3.md',
                error_message: null,
              },
              {
                id: 'exec-2',
                task_id: taskId,
                session_id: 'session-2',
                status: 'success',
                started_at: new Date(now.getTime() - 30 * 60 * 1000).toISOString(),
                finished_at: new Date(now.getTime() - 29 * 60 * 1000).toISOString(),
                output_file: '/path/to/output2.txt',
                error_message: null,
              },
              {
                id: 'exec-1',
                task_id: taskId,
                session_id: 'session-1',
                status: 'timeout',
                started_at: new Date(now.getTime() - 60 * 60 * 1000).toISOString(),
                finished_at: new Date(now.getTime() - 59 * 60 * 1000).toISOString(),
                output_file: null,
                error_message: 'Timeout exceeded',
              },
            ];
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');
    await page.fill('#task-name', 'Task With Multiple Runs');
    await page.fill('#prompt', 'multiple executions');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 * * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Multiple Runs")');
    await page.click('button:has-text("History")');

    const executionItems = page.locator('.execution-item');
    await expect(executionItems).toHaveCount(3);

    const clickableItems = page.locator('.execution-item.clickable');
    const nonClickableItems = page.locator('.execution-item:not(.clickable)');
    await expect(clickableItems).toHaveCount(2);
    await expect(nonClickableItems).toHaveCount(1);

    await page.screenshot({ path: '.sisyphus/evidence/task-28-multiple-executions.png' });
  });

  test('should handle keyboard navigation for output viewing', async ({ page }) => {
    await page.addInitScript(() => {
      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);

          if (cmd === 'get_tasks') {
            return [];
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            return {
              id: `test-task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              skip_if_running: newTask.skip_if_running || 0,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();
            const startedAt = new Date(now.getTime() - 5 * 60 * 1000);
            const finishedAt = new Date(now.getTime() - 4 * 60 * 1000);

            return [
              {
                id: `exec-${Date.now()}`,
                task_id: taskId,
                session_id: 'session-789',
                status: 'success',
                started_at: startedAt.toISOString(),
                finished_at: finishedAt.toISOString(),
                output_file: '/path/to/output.txt',
                error_message: null,
              },
            ];
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');
    await page.fill('#task-name', 'Keyboard Navigation Task');
    await page.fill('#prompt', 'echo test');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Keyboard Navigation Task")');
    await page.click('button:has-text("History")');

    const executionItem = page.locator('.execution-item.clickable');
    await expect(executionItem).toBeVisible();

    await executionItem.focus();
    await expect(executionItem).toBeFocused();

    await executionItem.press('Enter');

    await page.screenshot({ path: '.sisyphus/evidence/task-28-keyboard-nav.png' });
  });

  test('should verify execution history displays correct information', async ({ page }) => {
    await page.addInitScript(() => {
      Reflect.set(window, '__TAURI_INTERNALS__', {
        invoke: async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);

          if (cmd === 'get_tasks') {
            return [];
          }

          if (cmd === 'create_task') {
            const newTask = args?.newTask;
            return {
              id: `test-task-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              skip_if_running: newTask.skip_if_running || 0,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();
            const startedAt = new Date(now.getTime() - 10 * 60 * 1000);
            const finishedAt = new Date(now.getTime() - 5 * 60 * 1000);

            return [
              {
                id: `exec-${Date.now()}`,
                task_id: taskId,
                session_id: 'session-999',
                status: 'success',
                started_at: startedAt.toISOString(),
                finished_at: finishedAt.toISOString(),
                output_file: '/path/to/output.md',
                error_message: null,
              },
            ];
          }

          return null;
        },
      });
    });

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');
    await page.fill('#task-name', 'Info Display Task');
    await page.fill('#prompt', 'info test');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Info Display Task")');
    await page.click('button:has-text("History")');

    await expect(page.locator('.execution-status.status-success')).toBeVisible();
    await expect(page.locator('.execution-status.status-success')).toContainText('success');

    await expect(page.locator('.execution-time')).toBeVisible();

    await expect(page.locator('.execution-duration')).toContainText('Duration:');
    await expect(page.locator('.execution-duration')).toContainText('5 minutes');

    await page.screenshot({ path: '.sisyphus/evidence/task-28-info-display.png' });
  });
});

import { test, expect } from '@playwright/test';

test.describe('Task Execution E2E', () => {
  test('should display empty execution history for new task', async ({ page }) => {
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
              id: `test-${Date.now()}`,
              name: newTask.name,
              prompt: newTask.prompt,
              cron_expression: newTask.cron_expression || null,
              simple_schedule: newTask.simple_schedule || null,
              enabled: newTask.enabled || 1,
              timeout_seconds: newTask.timeout_seconds || 300,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            return [];
          }

          return null;
        },
      });
    });

    await page.goto('/');

    // Wait for app to load
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    await page.click('button:has-text("+ New Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).toBeVisible();

    await page.fill('#task-name', 'Test Task with No History');
    await page.fill('#prompt', 'echo test');

    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');

    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();

    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await expect(page.locator('.sidebar-task-item')).toContainText('Test Task with No History');

    await page.click('.sidebar-task-item:has-text("Test Task with No History")');

    await page.click('button:has-text("History")');

    // Verify empty state is shown
    await expect(page.locator('.execution-history-empty')).toBeVisible();
    await expect(page.locator('h3:has-text("No execution history")')).toBeVisible();
    await expect(
      page.locator('p:has-text("Run this task to see execution history")')
    ).toBeVisible();

    // Take screenshot
    await page.screenshot({ path: '.sisyphus/evidence/task-27-empty-history.png' });
  });

  test('should display execution history with success status', async ({ page }) => {
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
    await page.fill('#task-name', 'Task With Multiple Executions');
    await page.fill('#prompt', 'multiple runs');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 12 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Multiple Executions")');
    await page.click('button:has-text("History")');

    // Verify execution history is shown
    await expect(page.locator('.execution-history')).toBeVisible();
    await expect(page.locator('.execution-item')).toBeVisible();

    // Verify success status
    const statusElement = page.locator('.execution-status.status-success');
    await expect(statusElement).toBeVisible();
    await expect(statusElement).toContainText('success');

    // Verify time is displayed
    await expect(page.locator('.execution-time')).toBeVisible();

    // Verify duration is displayed
    await expect(page.locator('.execution-duration')).toContainText('Duration:');

    // Take screenshot
    await page.screenshot({ path: '.sisyphus/evidence/task-27-success-execution.png' });
  });

  test('should display execution history with timeout status', async ({ page }) => {
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

              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
            };
          }

          if (cmd === 'get_executions') {
            const taskId = args?.taskId;
            const now = new Date();
            const startedAt = new Date(now.getTime() - 10 * 60 * 1000);
            const finishedAt = new Date(now.getTime() - 9 * 60 * 1000 - 50 * 1000);

            return [
              {
                id: `exec-${Date.now()}`,
                task_id: taskId,
                session_id: 'session-456',
                status: 'timeout',
                started_at: startedAt.toISOString(),
                finished_at: finishedAt.toISOString(),
                output_file: null,
                error_message: 'Task execution timed out after 300 seconds',
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
    await page.fill('#task-name', 'Task With Success Execution');
    await page.fill('#prompt', 'echo success');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Success Execution")');
    await page.click('button:has-text("History")');

    // Verify timeout status
    const statusElement = page.locator('.execution-status.status-timeout');
    await expect(statusElement).toBeVisible();
    await expect(statusElement).toContainText('timeout');

    await page.screenshot({ path: '.sisyphus/evidence/task-27-timeout-execution.png' });
  });

  test('should display execution history with failed status', async ({ page }) => {
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
                session_id: 'session-789',
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
    await page.fill('#task-name', 'Task With Timeout');
    await page.fill('#prompt', 'sleep 600');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 10 * * *');
    await page.fill('#timeout', '5');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Timeout")');
    await page.click('button:has-text("History")');

    // Verify failed status
    const statusElement = page.locator('.execution-status.status-failed');
    await expect(statusElement).toBeVisible();
    await expect(statusElement).toContainText('failed');

    // Verify error message is shown
    await expect(page.locator('.execution-error')).toBeVisible();
    await expect(page.locator('.execution-error')).toContainText('exit code 1');

    // Take screenshot
    await page.screenshot({ path: '.sisyphus/evidence/task-27-failed-execution.png' });
  });

  test('should display multiple executions in history', async ({ page }) => {
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
                output_file: '/path/to/output3.txt',
                error_message: null,
              },
              {
                id: 'exec-2',
                task_id: taskId,
                session_id: 'session-2',
                status: 'failed',
                started_at: new Date(now.getTime() - 30 * 60 * 1000).toISOString(),
                finished_at: new Date(now.getTime() - 29 * 60 * 1000).toISOString(),
                output_file: null,
                error_message: 'Command failed',
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
    await page.fill('#task-name', 'Task With Failed Execution');
    await page.fill('#prompt', 'exit 1');
    await page.click('input[value="cron"]');
    await page.getByLabel('Cron Expression *').fill('0 11 * * *');
    await page.click('button:has-text("Create Task")');

    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    await page.click('.sidebar-task-item:has-text("Task With Failed Execution")');
    await page.click('button:has-text("History")');

    // Verify multiple executions are shown
    const executionItems = page.locator('.execution-item');
    await expect(executionItems).toHaveCount(3);

    // Verify different statuses
    await expect(page.locator('.execution-status.status-success')).toBeVisible();
    await expect(page.locator('.execution-status.status-failed')).toBeVisible();
    await expect(page.locator('.execution-status.status-timeout')).toBeVisible();

    // Take screenshot
    await page.screenshot({ path: '.sisyphus/evidence/task-27-multiple-executions.png' });
  });
});

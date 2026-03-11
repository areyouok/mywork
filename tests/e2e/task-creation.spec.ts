import { test, expect } from '@playwright/test';

test.describe('Task Creation E2E', () => {
  test.beforeEach(async ({ page }) => {
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

          return null;
        },
      });
    });
  });

  test('should create a new task end-to-end', async ({ page }) => {
    // Step 1: Open app
    await page.goto('/');

    // Wait for app to load
    await expect(page.locator('h1')).toContainText('MyWork Scheduler');

    // Verify initial state - no tasks
    await expect(page.locator('.task-count')).toContainText('0');

    // Step 2: Click "New Task" button
    await page.click('button:has-text("+ New Task")');

    // Verify form is displayed
    await expect(page.locator('h2:has-text("Create New Task")')).toBeVisible();

    // Step 3: Fill form
    // Task Name
    await page.fill('#task-name', 'E2E Test Task');

    // Prompt
    await page.fill('#prompt', 'echo hello');

    // Select Cron schedule type
    await page.click('input[value="cron"]');

    // Fill cron expression
    await page.getByLabel('Cron Expression *').fill('0 9 * * *');

    // Set timeout
    await page.fill('#timeout', '600');

    // Check "Skip if running"
    // Take screenshot before submission
    await page.screenshot({ path: '.sisyphus/evidence/task-26-form-filled.png' });

    // Listen for console messages for debugging
    page.on('console', (msg) => console.log('BROWSER LOG:', msg.text()));

    // Step 4: Submit form
    await page.click('button:has-text("Create Task")');

    // Wait for form to close (viewMode changes to 'list')
    await expect(page.locator('h2:has-text("Create New Task")')).not.toBeVisible();

    // Step 5: Verify task appears in list
    await expect(page.locator('.task-count')).toContainText('1', { timeout: 10000 });

    // Verify task is in sidebar
    await expect(page.locator('.sidebar-task-item')).toContainText('E2E Test Task');

    // Take screenshot of final state
    await page.screenshot({ path: '.sisyphus/evidence/task-26-e2e-create.png' });
  });

  test('should validate required fields', async ({ page }) => {
    await page.goto('/');

    // Click "New Task"
    await page.click('button:has-text("+ New Task")');

    // Try to submit without filling required fields
    await page.click('button:has-text("Create Task")');

    // Verify error messages are shown
    await expect(page.locator('#name-error')).toContainText('Name is required');
    await expect(page.locator('#prompt-error')).toContainText('Prompt is required');
  });

  test('should support simple schedule type', async ({ page }) => {
    await page.goto('/');

    // Click "New Task"
    await page.click('button:has-text("+ New Task")');

    // Fill form with simple schedule
    await page.fill('#task-name', 'Simple Schedule Task');
    await page.fill('#prompt', 'test prompt');

    // Select Simple schedule type (should be default)
    await page.click('input[value="simple"]');

    // Fill simple schedule
    await page.getByLabel('Simple Schedule *').selectOption('daily');
    await page.getByLabel('Time (24h)').fill('09:30');

    // Submit
    await page.click('button:has-text("Create Task")');

    // Verify task created
    await expect(page.locator('.task-count')).toContainText('1');
    await expect(page.locator('.sidebar-task-item')).toContainText('Simple Schedule Task');
  });
});

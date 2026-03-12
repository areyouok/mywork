import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import App from './App';
import * as api from './api/tasks';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

const mockTasks = [
  {
    id: '1',
    name: 'Daily Code Review',
    prompt: 'Review code',
    enabled: true,
    timeout_seconds: 300,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    name: 'Weekly Report',
    prompt: 'Generate report',
    enabled: false,
    timeout_seconds: 600,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
];

const mockExecutions = [
  {
    id: 'exec-1',
    task_id: '1',
    status: 'success' as const,
    started_at: '2024-01-01T10:00:00Z',
    finished_at: '2024-01-01T10:05:00Z',
  },
];

const mockGetExecution = vi.hoisted(() => vi.fn());
const mockGetOutput = vi.hoisted(() => vi.fn());

vi.mock('./api/tasks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./api/tasks')>();
  return {
    ...actual,
    getTasks: vi.fn(),
    createTask: vi.fn(),
    updateTask: vi.fn(),
    deleteTask: vi.fn(),
    getExecutions: vi.fn(),
    getRunningExecutions: vi.fn(),
    getSchedulerStatus: vi.fn(),
    startScheduler: vi.fn(),
    reloadScheduler: vi.fn(),
    runTask: vi.fn(),
    stopScheduler: vi.fn(),
    getExecution: mockGetExecution,
    getOutput: mockGetOutput,
  };
});

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getTasks).mockResolvedValue(mockTasks);
    vi.mocked(api.getExecutions).mockResolvedValue(mockExecutions);
    vi.mocked(api.updateTask).mockResolvedValue(mockTasks[0]);
    vi.mocked(api.deleteTask).mockResolvedValue(true);
    vi.mocked(api.getRunningExecutions).mockResolvedValue([]);
    vi.mocked(api.getSchedulerStatus).mockResolvedValue('running');
    vi.mocked(api.startScheduler).mockResolvedValue('Scheduler started successfully');
    vi.mocked(api.reloadScheduler).mockResolvedValue('Scheduler reloaded');
    mockGetExecution.mockResolvedValue(mockExecutions[0]);
    mockGetOutput.mockResolvedValue('');
  });

  afterEach(() => {
    vi.useRealTimers();
  });
  it('should render app header with title', () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: /mywork scheduler/i })).toBeInTheDocument();
  });

  it('should render new task button', () => {
    render(<App />);

    expect(screen.getByRole('button', { name: /\+ new task/i })).toBeInTheDocument();
  });

  it('should render sidebar with task count', async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Tasks')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
    });
  });

  it('should render all tasks in sidebar', async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
      expect(screen.getByText('Weekly Report')).toBeInTheDocument();
    });
  });

  it('should show empty state when no task is selected', () => {
    render(<App />);

    expect(screen.getByText('Select a Task')).toBeInTheDocument();
    expect(screen.getByText('Choose a task from the sidebar to view details')).toBeInTheDocument();
  });

  it('should select task when clicking on sidebar item', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));

    expect(
      screen.getByRole('heading', { level: 2, name: /daily code review/i })
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /history/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /delete/i })).toBeInTheDocument();
  });

  it('should switch to form view when clicking new task button', async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole('button', { name: /\+ new task/i }));

    expect(screen.getByRole('heading', { name: /create new task/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/task name/i)).toBeInTheDocument();
  });

  it('should switch to form view when clicking edit button', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));
    await user.click(screen.getByRole('button', { name: /edit/i }));

    expect(screen.getByRole('heading', { name: /edit task/i })).toBeInTheDocument();
    expect(screen.getByDisplayValue('Daily Code Review')).toBeInTheDocument();
  });

  it('should switch to history view when clicking history button', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));
    await user.click(screen.getByRole('button', { name: /history/i }));

    expect(screen.getByRole('heading', { name: /execution history/i })).toBeInTheDocument();
    expect(screen.getByText('Back to Task')).toBeInTheDocument();
  });

  it('should render execution status badge in output header', async () => {
    const user = userEvent.setup();
    const runningExecution = {
      id: 'exec-running',
      task_id: '1',
      status: 'running' as const,
      started_at: '2024-01-01T10:00:00Z',
      output_file: 'exec-running',
    };

    vi.mocked(api.getExecutions).mockResolvedValue([runningExecution]);
    vi.mocked(api.getOutput).mockResolvedValue('stream output');
    mockGetExecution.mockResolvedValue({
      ...runningExecution,
      status: 'success',
      finished_at: '2024-01-01T10:01:00Z',
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));
    await user.click(screen.getByRole('button', { name: /history/i }));

    await waitFor(() => {
      expect(screen.getByText('running')).toBeInTheDocument();
    });

    await user.click(screen.getByText('running'));

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /output -/i })).toBeInTheDocument();
      expect(screen.getByText('running')).toBeInTheDocument();
    });
  });

  it('should update output status badge when execution finishes while staying on output page', async () => {
    const user = userEvent.setup();
    const listeners: Record<string, (event: { payload: string }) => void> = {};
    Reflect.set(window, '__TAURI_INTERNALS__', {});

    const runningExecution = {
      id: 'exec-live',
      task_id: '1',
      status: 'running' as const,
      started_at: '2024-01-01T10:00:00Z',
      output_file: 'exec-live',
    };
    const finishedExecution = {
      ...runningExecution,
      status: 'success' as const,
      finished_at: '2024-01-01T10:05:00Z',
    };

    let shouldReturnFinished = false;
    vi.mocked(api.getExecutions).mockImplementation(async (taskId: string) => {
      if (!taskId) {
        return [];
      }
      return shouldReturnFinished ? [finishedExecution] : [runningExecution];
    });
    vi.mocked(api.getOutput).mockResolvedValue('live output');
    vi.mocked(listen).mockImplementation(async (eventName, handler) => {
      listeners[eventName] = handler as (event: { payload: string }) => void;
      return () => {
        delete listeners[eventName];
      };
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));
    await user.click(screen.getByRole('button', { name: /history/i }));

    await waitFor(() => {
      expect(screen.getByText('running')).toBeInTheDocument();
    });

    await user.click(screen.getByText('running'));

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /output -/i })).toBeInTheDocument();
      expect(screen.getByText('running')).toBeInTheDocument();
    });

    shouldReturnFinished = true;
    listeners['execution-finished']?.({ payload: '1' });

    await waitFor(() => {
      expect(screen.getByText('success')).toBeInTheDocument();
    });

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  });

  it('should show run button as running when scheduler emits execution-started and reset on execution-finished', async () => {
    const user = userEvent.setup();
    const listeners: Record<string, (event: { payload: string }) => void> = {};
    Reflect.set(window, '__TAURI_INTERNALS__', {});

    vi.mocked(listen).mockImplementation(async (eventName, handler) => {
      listeners[eventName] = handler as (event: { payload: string }) => void;
      return () => {
        delete listeners[eventName];
      };
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));

    await waitFor(() => {
      expect(listeners['execution-started']).toBeDefined();
      expect(listeners['execution-finished']).toBeDefined();
    });

    const runButton = screen.getByRole('button', { name: /run daily code review/i });
    expect(runButton).toHaveTextContent('Run');
    expect(runButton).not.toBeDisabled();

    listeners['execution-started']?.({ payload: '1' });

    await waitFor(() => {
      expect(runButton).toHaveTextContent('Running...');
      expect(runButton).toBeDisabled();
    });

    listeners['execution-finished']?.({ payload: '1' });

    await waitFor(() => {
      expect(runButton).toHaveTextContent('Run');
      expect(runButton).not.toBeDisabled();
    });

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  });

  it('should toggle task enabled status', async () => {
    const user = userEvent.setup();

    const updatedTasks = [{ ...mockTasks[0], enabled: false }, mockTasks[1]];

    vi.mocked(api.getTasks).mockResolvedValueOnce(mockTasks).mockResolvedValueOnce(updatedTasks);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));

    const toggleButton = screen.getByRole('switch', { name: /toggle daily code review/i });
    expect(toggleButton).toHaveAttribute('aria-checked', 'true');

    await user.click(toggleButton);

    await waitFor(() => {
      expect(toggleButton).toHaveAttribute('aria-checked', 'false');
    });
  });

  it('should delete task after confirmation', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Weekly Report')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Weekly Report'));

    const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
    await user.click(deleteButtons[0]);

    expect(screen.getByText('Are you sure?')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /confirm/i }));

    expect(screen.queryByText('Weekly Report')).not.toBeInTheDocument();
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('should cancel task deletion', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Weekly Report')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Weekly Report'));

    const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
    await user.click(deleteButtons[0]);

    expect(screen.getByText('Are you sure?')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /cancel/i }));

    expect(screen.queryByText('Are you sure?')).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 2, name: /weekly report/i })).toBeInTheDocument();
  });

  it('should show task status indicators', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));

    const taskItems = screen.getAllByRole('listitem');
    expect(taskItems.length).toBeGreaterThan(0);
  });

  it('should switch back to list view from history', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));
    await user.click(screen.getByRole('button', { name: /history/i }));

    expect(screen.getByRole('heading', { name: /execution history/i })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /back to task/i }));

    expect(
      screen.getByRole('heading', { level: 2, name: /daily code review/i })
    ).toBeInTheDocument();
  });

  it('should apply selected class to active task in sidebar', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    const dailyReviewItem = screen.getByText('Daily Code Review').closest('.sidebar-task-item');
    expect(dailyReviewItem).not.toHaveClass('selected');

    await user.click(screen.getByText('Daily Code Review'));
    expect(dailyReviewItem).toHaveClass('selected');
  });

  it('should move running task to top when tasks reload with newer updated_at', async () => {
    const user = userEvent.setup();
    const listeners: Record<string, (event: { payload: string }) => void> = {};
    Reflect.set(window, '__TAURI_INTERNALS__', {});

    const initialTasks = [
      {
        id: '2',
        name: 'Weekly Report',
        prompt: 'Generate report',
        enabled: true,
        timeout_seconds: 300,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      },
      {
        id: '1',
        name: 'Daily Code Review',
        prompt: 'Review code',
        enabled: true,
        timeout_seconds: 300,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      },
    ];

    const reorderedTasks = [
      {
        ...initialTasks[1],
        updated_at: '2024-01-03T00:00:00Z',
      },
      initialTasks[0],
    ];

    let shouldReturnReordered = false;
    vi.mocked(api.getTasks).mockImplementation(async () =>
      shouldReturnReordered ? reorderedTasks : initialTasks
    );

    vi.mocked(listen).mockImplementation(async (eventName, handler) => {
      listeners[eventName] = handler as (event: { payload: string }) => void;
      return () => {
        delete listeners[eventName];
      };
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
      expect(screen.getByText('Weekly Report')).toBeInTheDocument();
    });

    await waitFor(() => {
      const names = Array.from(document.querySelectorAll('.sidebar-task-item .task-item-name')).map(
        (element) => element.textContent?.trim()
      );
      expect(names).toEqual(['Weekly Report', 'Daily Code Review']);
    });

    await user.click(screen.getByText('Daily Code Review'));
    await waitFor(() => {
      expect(listeners['execution-started']).toBeDefined();
    });

    shouldReturnReordered = true;
    listeners['execution-started']?.({ payload: '1' });

    await waitFor(() => {
      const names = Array.from(document.querySelectorAll('.sidebar-task-item .task-item-name')).map(
        (element) => element.textContent?.trim()
      );
      expect(names).toEqual(['Daily Code Review', 'Weekly Report']);
    });

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  });
});

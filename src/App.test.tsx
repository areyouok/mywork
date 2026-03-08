import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import App from './App';
import * as api from './api/tasks';

vi.mock('./api/tasks', () => ({
  getTasks: vi.fn(),
  createTask: vi.fn(),
  updateTask: vi.fn(),
  deleteTask: vi.fn(),
  getExecutions: vi.fn(),
}));

const mockTasks = [
  {
    id: '1',
    name: 'Daily Code Review',
    prompt: 'Review code',
    enabled: true,
    timeout_seconds: 300,
    skip_if_running: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    name: 'Weekly Report',
    prompt: 'Generate report',
    enabled: false,
    timeout_seconds: 600,
    skip_if_running: false,
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

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getTasks).mockResolvedValue(mockTasks);
    vi.mocked(api.getExecutions).mockResolvedValue(mockExecutions);
    vi.mocked(api.updateTask).mockResolvedValue(mockTasks[0]);
    vi.mocked(api.deleteTask).mockResolvedValue(true);
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

  it('should toggle task enabled status', async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Daily Code Review')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Daily Code Review'));

    const toggleButton = screen.getByRole('switch', { name: /toggle daily code review/i });
    expect(toggleButton).toHaveAttribute('aria-checked', 'true');

    await user.click(toggleButton);
    expect(toggleButton).toHaveAttribute('aria-checked', 'false');
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
});

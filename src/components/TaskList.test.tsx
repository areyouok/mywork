import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TaskList } from './TaskList';
import type { Task } from '@/types/task';

const mockTasks: Task[] = [
  {
    id: '1',
    name: 'Daily Report',
    prompt: 'Generate daily report',
    cron_expression: '0 9 * * *',
    enabled: true,
    timeout_seconds: 300,
    skip_if_running: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    name: 'Weekly Summary',
    prompt: 'Create weekly summary',
    simple_schedule: '{"type":"weekly","day":"monday","hour":10}',
    enabled: false,
    timeout_seconds: 600,
    skip_if_running: false,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
];

describe('TaskList', () => {
  describe('Rendering', () => {
    it('renders empty state when no tasks', () => {
      render(<TaskList tasks={[]} />);
      expect(screen.getByText(/no tasks yet/i)).toBeInTheDocument();
      expect(screen.getByText(/create your first task/i)).toBeInTheDocument();
    });

    it('renders task list with tasks', () => {
      render(<TaskList tasks={mockTasks} />);
      expect(screen.getByText('Daily Report')).toBeInTheDocument();
      expect(screen.getByText('Weekly Summary')).toBeInTheDocument();
    });

    it('displays task prompt', () => {
      render(<TaskList tasks={mockTasks} />);
      expect(screen.getByText('Generate daily report')).toBeInTheDocument();
      expect(screen.getByText('Create weekly summary')).toBeInTheDocument();
    });

    it('displays schedule information', () => {
      render(<TaskList tasks={mockTasks} />);
      expect(screen.getByText(/0 9 \* \* \*/i)).toBeInTheDocument();
    });

    it('displays enabled state correctly', () => {
      render(<TaskList tasks={mockTasks} />);
      const switches = screen.getAllByRole('switch');
      expect(switches[0]).toHaveAttribute('aria-checked', 'true');
      expect(switches[1]).toHaveAttribute('aria-checked', 'false');
    });
  });

  describe('Interactions', () => {
    it('toggles task enabled state', async () => {
      const onToggle = vi.fn();
      const user = userEvent.setup();

      render(<TaskList tasks={mockTasks} onToggle={onToggle} />);

      const firstSwitch = screen.getAllByRole('switch')[0];
      await user.click(firstSwitch);

      expect(onToggle).toHaveBeenCalledWith('1', false);
    });

    it('deletes task after confirmation', async () => {
      const onDelete = vi.fn();
      const user = userEvent.setup();

      render(<TaskList tasks={mockTasks} onDelete={onDelete} />);

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
      await user.click(deleteButtons[0]);

      expect(screen.getByText(/are you sure/i)).toBeInTheDocument();

      const confirmButton = screen.getByRole('button', { name: /confirm/i });
      await user.click(confirmButton);

      expect(onDelete).toHaveBeenCalledWith('1');
    });

    it('cancels deletion', async () => {
      const onDelete = vi.fn();
      const user = userEvent.setup();

      render(<TaskList tasks={mockTasks} onDelete={onDelete} />);

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
      await user.click(deleteButtons[0]);

      const cancelButton = screen.getByRole('button', { name: /cancel/i });
      await user.click(cancelButton);

      expect(onDelete).not.toHaveBeenCalled();
      expect(screen.queryByText(/are you sure/i)).not.toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper aria labels', () => {
      render(<TaskList tasks={mockTasks} />);

      const switches = screen.getAllByRole('switch');
      const switchLabel = switches[0].getAttribute('aria-label');
      expect(switchLabel).toMatch(/toggle.*daily report/i);

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
      const deleteLabel = deleteButtons[0].getAttribute('aria-label');
      expect(deleteLabel).toMatch(/delete.*daily report/i);
    });

    it('supports keyboard navigation', async () => {
      const user = userEvent.setup();

      render(<TaskList tasks={mockTasks} />);

      await user.tab();
      expect(screen.getAllByRole('switch')[0]).toHaveFocus();

      await user.tab();
      expect(screen.getAllByRole('button', { name: /delete/i })[0]).toHaveFocus();
    });
  });
});

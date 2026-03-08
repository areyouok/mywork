import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ExecutionHistory } from './ExecutionHistory';
import type { Execution } from '@/types/execution';

function createMockExecution(overrides?: Partial<Execution>): Execution {
  const now = new Date().toISOString();
  return {
    id: 'exec-123',
    task_id: 'task-456',
    status: 'success',
    started_at: now,
    finished_at: now,
    ...overrides,
  };
}

describe('ExecutionHistory', () => {
  describe('Rendering', () => {
    it('should render empty state when no executions', () => {
      render(<ExecutionHistory executions={[]} />);

      expect(screen.getByText('No execution history')).toBeInTheDocument();
      expect(screen.getByText('Run this task to see execution history')).toBeInTheDocument();
    });

    it('should render list of executions', () => {
      const executions = [
        createMockExecution({ id: 'exec-1', started_at: '2024-03-09T10:00:00Z' }),
        createMockExecution({ id: 'exec-2', started_at: '2024-03-09T11:00:00Z' }),
        createMockExecution({ id: 'exec-3', started_at: '2024-03-09T12:00:00Z' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getAllByRole('listitem')).toHaveLength(3);
    });

    it('should display execution time', () => {
      const executions = [createMockExecution({ started_at: '2024-03-09T10:00:00Z' })];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/2024-03-09|Mar 9, 2024|ago/i)).toBeInTheDocument();
    });

    it('should display status for each execution', () => {
      const executions = [createMockExecution({ status: 'success' })];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText('success')).toBeInTheDocument();
    });

    it('should display duration when execution is finished', () => {
      const startedAt = '2024-03-09T10:00:00Z';
      const finishedAt = '2024-03-09T10:00:30Z';

      const executions = [
        createMockExecution({ started_at: startedAt, finished_at: finishedAt, status: 'success' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/30s|30 seconds/i)).toBeInTheDocument();
    });

    it('should not display duration when execution is not finished', () => {
      const executions = [createMockExecution({ status: 'running', finished_at: undefined })];

      render(<ExecutionHistory executions={executions} />);

      const item = screen.getByRole('listitem');
      expect(item).toHaveTextContent(/running/i);
    });
  });

  describe('Status Display', () => {
    it('should display pending status with gray color', () => {
      const executions = [createMockExecution({ status: 'pending' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('pending');
      expect(statusElement).toHaveClass('status-pending');
    });

    it('should display running status with blue color', () => {
      const executions = [createMockExecution({ status: 'running' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('running');
      expect(statusElement).toHaveClass('status-running');
    });

    it('should display success status with green color', () => {
      const executions = [createMockExecution({ status: 'success' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('success');
      expect(statusElement).toHaveClass('status-success');
    });

    it('should display failed status with red color', () => {
      const executions = [createMockExecution({ status: 'failed' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('failed');
      expect(statusElement).toHaveClass('status-failed');
    });

    it('should display timeout status with orange color', () => {
      const executions = [createMockExecution({ status: 'timeout' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('timeout');
      expect(statusElement).toHaveClass('status-timeout');
    });

    it('should display skipped status with yellow color', () => {
      const executions = [createMockExecution({ status: 'skipped' })];

      render(<ExecutionHistory executions={executions} />);

      const statusElement = screen.getByText('skipped');
      expect(statusElement).toHaveClass('status-skipped');
    });

    it('should display error message when execution failed', () => {
      const executions = [
        createMockExecution({
          status: 'failed',
          error_message: 'Task execution failed: command not found',
        }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/Task execution failed: command not found/i)).toBeInTheDocument();
    });
  });

  describe('Interaction', () => {
    it('should call onViewOutput when execution with output is clicked', async () => {
      const user = userEvent.setup();
      const executions = [createMockExecution({ output_file: '/output/result.txt' })];
      const onViewOutput = vi.fn();

      render(<ExecutionHistory executions={executions} onViewOutput={onViewOutput} />);

      const item = screen.getByRole('listitem');
      await user.click(item);

      expect(onViewOutput).toHaveBeenCalledTimes(1);
      expect(onViewOutput).toHaveBeenCalledWith(executions[0]);
    });

    it('should not call onViewOutput when execution has no output file', async () => {
      const user = userEvent.setup();
      const executions = [createMockExecution({ output_file: undefined })];
      const onViewOutput = vi.fn();

      render(<ExecutionHistory executions={executions} onViewOutput={onViewOutput} />);

      const item = screen.getByRole('listitem');
      await user.click(item);

      expect(onViewOutput).not.toHaveBeenCalled();
    });

    it('should not fail when onViewOutput is not provided', async () => {
      const user = userEvent.setup();
      const executions = [createMockExecution()];

      render(<ExecutionHistory executions={executions} />);

      const item = screen.getByRole('listitem');
      // Should not throw error
      await user.click(item);
    });

    it('should have clickable cursor when execution has output file', () => {
      const executions = [createMockExecution({ output_file: '/output/result.txt' })];
      const onViewOutput = vi.fn();

      render(<ExecutionHistory executions={executions} onViewOutput={onViewOutput} />);

      const item = screen.getByRole('listitem');
      expect(item).toHaveClass('clickable');
    });

    it('should not have clickable cursor when execution has no output file', () => {
      const executions = [createMockExecution({ output_file: undefined })];

      render(<ExecutionHistory executions={executions} />);

      const item = screen.getByRole('listitem');
      expect(item).not.toHaveClass('clickable');
    });
  });

  describe('Filtering', () => {
    it('should filter executions by taskId when provided', () => {
      const executions = [
        createMockExecution({ id: 'exec-1', task_id: 'task-1' }),
        createMockExecution({ id: 'exec-2', task_id: 'task-1' }),
        createMockExecution({ id: 'exec-3', task_id: 'task-2' }),
      ];

      render(<ExecutionHistory executions={executions} taskId="task-1" />);

      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(2);
    });

    it('should show all executions when taskId is not provided', () => {
      const executions = [
        createMockExecution({ id: 'exec-1', task_id: 'task-1' }),
        createMockExecution({ id: 'exec-2', task_id: 'task-2' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(2);
    });
  });

  describe('Time Formatting', () => {
    it('should display relative time for recent executions', () => {
      const oneHourAgo = new Date(Date.now() - 3600000).toISOString();
      const executions = [createMockExecution({ started_at: oneHourAgo })];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/1 hour ago|an hour ago/i)).toBeInTheDocument();
    });

    it('should display absolute time for old executions', () => {
      const oneYearAgo = '2023-03-09T10:00:00Z';
      const executions = [createMockExecution({ started_at: oneYearAgo })];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/2023-03-09|Mar 9, 2023/i)).toBeInTheDocument();
    });

    it('should display duration in seconds', () => {
      const startedAt = '2024-03-09T10:00:00Z';
      const finishedAt = '2024-03-09T10:00:45Z';

      const executions = [
        createMockExecution({ started_at: startedAt, finished_at: finishedAt, status: 'success' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/45s|45 seconds/i)).toBeInTheDocument();
    });

    it('should display duration in minutes', () => {
      const startedAt = '2024-03-09T10:00:00Z';
      const finishedAt = '2024-03-09T10:05:00Z';

      const executions = [
        createMockExecution({ started_at: startedAt, finished_at: finishedAt, status: 'success' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/5m|5 minutes/i)).toBeInTheDocument();
    });

    it('should display duration in hours', () => {
      const startedAt = '2024-03-09T10:00:00Z';
      const finishedAt = '2024-03-09T12:30:00Z';

      const executions = [
        createMockExecution({ started_at: startedAt, finished_at: finishedAt, status: 'success' }),
      ];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByText(/2h 30m|2 hours 30 minutes/i)).toBeInTheDocument();
    });
  });

  describe('Loading State', () => {
    it('should display loading state when loading is true', () => {
      render(<ExecutionHistory executions={[]} loading={true} />);

      expect(screen.getByText(/loading/i)).toBeInTheDocument();
    });

    it('should not display empty state when loading', () => {
      render(<ExecutionHistory executions={[]} loading={true} />);

      expect(screen.queryByText('No execution history')).not.toBeInTheDocument();
    });

    it('should display loading spinner', () => {
      render(<ExecutionHistory executions={[]} loading={true} />);

      expect(screen.getByRole('status', { name: /loading/i })).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper list structure', () => {
      const executions = [createMockExecution()];

      render(<ExecutionHistory executions={executions} />);

      expect(screen.getByRole('list')).toBeInTheDocument();
      expect(screen.getByRole('listitem')).toBeInTheDocument();
    });

    it('should have accessible name for clickable items', () => {
      const executions = [
        createMockExecution({ status: 'success', output_file: '/output/result.txt' }),
      ];
      const onViewOutput = vi.fn();

      render(<ExecutionHistory executions={executions} onViewOutput={onViewOutput} />);

      const item = screen.getByRole('listitem');
      expect(item).toHaveAttribute('aria-label');
      expect(item.getAttribute('aria-label')).toMatch(/execution|success/i);
    });

    it('should indicate status with aria-label', () => {
      const executions = [createMockExecution({ status: 'failed' })];

      render(<ExecutionHistory executions={executions} />);

      const item = screen.getByRole('listitem');
      expect(item.getAttribute('aria-label')).toMatch(/failed/i);
    });
  });
});

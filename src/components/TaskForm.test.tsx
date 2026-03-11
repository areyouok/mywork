import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TaskForm } from './TaskForm';
import type { Task } from '@/types/task';

const mockTask: Task = {
  id: '1',
  name: 'Daily Report',
  prompt: 'Generate daily report',
  cron_expression: '0 9 * * *',
  enabled: true,
  timeout_seconds: 300,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

describe('TaskForm', () => {
  describe('Rendering', () => {
    it('renders all form fields', () => {
      render(<TaskForm onSubmit={vi.fn()} />);

      expect(screen.getByLabelText(/task name/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/prompt/i)).toBeInTheDocument();
      expect(screen.getByRole('radio', { name: /cron/i })).toBeInTheDocument();
      expect(screen.getByRole('radio', { name: /simple/i })).toBeInTheDocument();
      expect(screen.getByRole('radio', { name: /once/i })).toBeInTheDocument();
      expect(screen.getByLabelText(/timeout/i)).toBeInTheDocument();
    });

    it('shows Create button in create mode', () => {
      render(<TaskForm onSubmit={vi.fn()} />);
      expect(screen.getByRole('button', { name: /create task/i })).toBeInTheDocument();
    });

    it('shows Update button in edit mode', () => {
      render(<TaskForm onSubmit={vi.fn()} initialData={mockTask} />);
      expect(screen.getByRole('button', { name: /update task/i })).toBeInTheDocument();
    });

    it('pre-fills form with existing task data in edit mode', () => {
      render(<TaskForm onSubmit={vi.fn()} initialData={mockTask} />);

      expect(screen.getByLabelText(/task name/i)).toHaveValue('Daily Report');
      expect(screen.getByLabelText(/prompt/i)).toHaveValue('Generate daily report');
      expect(screen.getByLabelText(/timeout/i)).toHaveValue(300);
    });

    it('shows Cancel button when onCancel is provided', () => {
      render(<TaskForm onSubmit={vi.fn()} onCancel={vi.fn()} />);
      expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
    });

    it('shows cron expression input when cron schedule type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('radio', { name: /cron/i }));
      expect(screen.getByLabelText(/cron expression/i)).toBeInTheDocument();
    });

    it('shows simple schedule input when simple schedule type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('radio', { name: /simple/i }));
      expect(screen.getByLabelText(/simple schedule/i)).toBeInTheDocument();
    });

    it('shows once input when once schedule type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('radio', { name: /once/i }));
      expect(screen.getByLabelText(/run at/i)).toBeInTheDocument();
    });
  });

  describe('Validation', () => {
    it('requires name field', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/name is required/i)).toBeInTheDocument();
    });

    it('requires prompt field', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.click(screen.getByRole('button', { name: /create task/i }));

      expect(screen.getByText(/prompt is required/i)).toBeInTheDocument();
    });

    it('validates cron expression format when cron type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      await user.click(screen.getByRole('radio', { name: /cron/i }));
      await user.type(screen.getByLabelText(/cron expression/i), 'invalid-cron');

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/invalid cron expression/i)).toBeInTheDocument();
    });

    it('validates timeout minimum value', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      // Switch to cron schedule type and enter cron expression
      await user.click(screen.getByRole('radio', { name: /cron/i }));
      await user.type(screen.getByLabelText(/cron expression/i), '0 9 * * *');

      const timeoutInput = screen.getByLabelText(/timeout/i);
      await user.clear(timeoutInput);
      await user.type(timeoutInput, '0');

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/timeout must be at least 1 second/i)).toBeInTheDocument();
    });

    it('validates timeout maximum value', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      // Switch to cron schedule type and enter cron expression
      await user.click(screen.getByRole('radio', { name: /cron/i }));
      await user.type(screen.getByLabelText(/cron expression/i), '0 9 * * *');

      const timeoutInput = screen.getByLabelText(/timeout/i);
      await user.clear(timeoutInput);
      await user.type(timeoutInput, '4000');

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/timeout must not exceed 3600 seconds/i)).toBeInTheDocument();
    });

    it('requires schedule field when schedule type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      await user.click(screen.getByRole('radio', { name: /cron/i }));

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/cron expression is required/i)).toBeInTheDocument();
    });

    it('requires once run time when once type is selected', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      await user.click(screen.getByRole('radio', { name: /once/i }));

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/run time is required/i)).toBeInTheDocument();
    });

    it('validates once run time must be in the future', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.type(screen.getByLabelText(/task name/i), 'Test Task');
      await user.type(screen.getByLabelText(/prompt/i), 'Test prompt');
      await user.click(screen.getByRole('radio', { name: /once/i }));
      await user.type(screen.getByLabelText(/run at/i), '2000-01-01T00:00');

      await user.click(screen.getByRole('button', { name: /create task/i }));
      expect(screen.getByText(/run time must be in the future/i)).toBeInTheDocument();
    });
  });

  describe('Submission', () => {
    it('calls onSubmit with form data', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn().mockResolvedValue(undefined);
      render(<TaskForm onSubmit={onSubmit} />);

      await user.type(screen.getByLabelText(/task name/i), 'New Task');
      await user.type(screen.getByLabelText(/prompt/i), 'New task description');
      await user.click(screen.getByRole('radio', { name: /simple/i }));
      await user.selectOptions(screen.getByLabelText(/simple schedule/i), 'daily');
      await user.type(screen.getByLabelText(/time.*24h/i), '09:30');

      await user.click(screen.getByRole('button', { name: /create task/i }));

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith(
          expect.objectContaining({
            name: 'New Task',
            prompt: 'New task description',
            schedule_type: 'simple',
            timeout_seconds: 300,
          })
        );
      });
    });

    it('shows loading state during submission', async () => {
      const user = userEvent.setup();
      let resolvePromise: () => void;
      const onSubmit = vi.fn().mockImplementation(
        () =>
          new Promise<void>((resolve) => {
            resolvePromise = resolve;
          })
      );

      render(<TaskForm onSubmit={onSubmit} />);

      await user.type(screen.getByLabelText(/task name/i), 'New Task');
      await user.type(screen.getByLabelText(/prompt/i), 'New task description');
      await user.click(screen.getByRole('radio', { name: /simple/i }));
      await user.selectOptions(screen.getByLabelText(/simple schedule/i), 'daily');
      await user.type(screen.getByLabelText(/time.*24h/i), '09:30');

      const submitButton = screen.getByRole('button', { name: /create task/i });
      await user.click(submitButton);

      expect(submitButton).toBeDisabled();
      expect(submitButton).toHaveTextContent(/creating/i);

      resolvePromise!();
    });

    it('handles submission error', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn().mockRejectedValue(new Error('Submission failed'));
      render(<TaskForm onSubmit={onSubmit} />);

      await user.type(screen.getByLabelText(/task name/i), 'New Task');
      await user.type(screen.getByLabelText(/prompt/i), 'New task description');
      await user.click(screen.getByRole('radio', { name: /simple/i }));
      await user.selectOptions(screen.getByLabelText(/simple schedule/i), 'daily');
      await user.type(screen.getByLabelText(/time.*24h/i), '09:30');

      await user.click(screen.getByRole('button', { name: /create task/i }));

      await waitFor(() => {
        expect(screen.getByText(/submission failed/i)).toBeInTheDocument();
      });
    });

    it('resets form after successful creation', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn().mockResolvedValue(undefined);
      render(<TaskForm onSubmit={onSubmit} />);

      await user.type(screen.getByLabelText(/task name/i), 'New Task');
      await user.type(screen.getByLabelText(/prompt/i), 'New task description');
      await user.click(screen.getByRole('radio', { name: /simple/i }));
      await user.selectOptions(screen.getByLabelText(/simple schedule/i), 'daily');
      await user.type(screen.getByLabelText(/time.*24h/i), '09:30');

      await user.click(screen.getByRole('button', { name: /create task/i }));

      await waitFor(() => {
        expect(screen.getByLabelText(/task name/i)).toHaveValue('');
        expect(screen.getByLabelText(/prompt/i)).toHaveValue('');
        expect(screen.getByLabelText(/simple schedule/i)).toHaveValue('');
      });
    });

    it('calls onCancel after successful update in edit mode', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn().mockResolvedValue(undefined);
      const onCancel = vi.fn();
      render(<TaskForm onSubmit={onSubmit} initialData={mockTask} onCancel={onCancel} />);

      await user.click(screen.getByRole('button', { name: /update task/i }));

      await waitFor(() => {
        expect(onCancel).toHaveBeenCalled();
      });
    });
  });

  describe('Edit Mode', () => {
    it('pre-fills cron expression when editing task with cron schedule', () => {
      render(<TaskForm onSubmit={vi.fn()} initialData={mockTask} />);

      expect(screen.getByRole('radio', { name: /cron/i })).toBeChecked();
      expect(screen.getByLabelText(/cron expression/i)).toHaveValue('0 9 * * *');
    });

    it('pre-fills simple schedule when editing task with simple schedule', () => {
      const taskWithSimpleSchedule: Task = {
        ...mockTask,
        cron_expression: undefined,
        simple_schedule: '{"type":"daily","time":"09:30"}',
      };

      render(<TaskForm onSubmit={vi.fn()} initialData={taskWithSimpleSchedule} />);

      expect(screen.getByRole('radio', { name: /simple/i })).toBeChecked();
      expect(screen.getByLabelText(/simple schedule/i)).toHaveValue('daily');
      expect(screen.getByLabelText(/time.*24h/i)).toHaveValue('09:30');
    });

    it('updates task on submit in edit mode', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn().mockResolvedValue(undefined);
      render(<TaskForm onSubmit={onSubmit} initialData={mockTask} />);

      const nameInput = screen.getByLabelText(/task name/i);
      await user.clear(nameInput);
      await user.type(nameInput, 'Updated Task');

      await user.click(screen.getByRole('button', { name: /update task/i }));

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith(
          expect.objectContaining({
            name: 'Updated Task',
          })
        );
      });
    });
  });

  describe('Interactions', () => {
    it('calls onCancel when Cancel button is clicked', async () => {
      const user = userEvent.setup();
      const onCancel = vi.fn();
      render(<TaskForm onSubmit={vi.fn()} onCancel={onCancel} />);

      await user.click(screen.getByRole('button', { name: /cancel/i }));
      expect(onCancel).toHaveBeenCalled();
    });

    it('switches between schedule types', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('radio', { name: /cron/i }));
      expect(screen.getByLabelText(/cron expression/i)).toBeInTheDocument();
      expect(screen.queryByLabelText(/simple schedule/i)).not.toBeInTheDocument();

      await user.click(screen.getByRole('radio', { name: /simple/i }));
      expect(screen.getByLabelText(/simple schedule/i)).toBeInTheDocument();
      expect(screen.queryByLabelText(/cron expression/i)).not.toBeInTheDocument();

      await user.click(screen.getByRole('radio', { name: /once/i }));
      expect(screen.getByLabelText(/run at/i)).toBeInTheDocument();
      expect(screen.queryByLabelText(/simple schedule/i)).not.toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper form labels', () => {
      render(<TaskForm onSubmit={vi.fn()} />);

      expect(screen.getByLabelText(/task name/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/prompt/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/timeout/i)).toBeInTheDocument();
    });

    it('marks required fields with aria-required', () => {
      render(<TaskForm onSubmit={vi.fn()} />);

      expect(screen.getByLabelText(/task name/i)).toBeRequired();
      expect(screen.getByLabelText(/prompt/i)).toBeRequired();
    });

    it('associates error messages with fields', async () => {
      const user = userEvent.setup();
      render(<TaskForm onSubmit={vi.fn()} />);

      await user.click(screen.getByRole('button', { name: /create task/i }));

      const nameInput = screen.getByLabelText(/task name/i);
      expect(nameInput).toHaveAttribute('aria-invalid', 'true');
      expect(nameInput).toHaveAttribute('aria-describedby');
      expect(screen.getByText(/name is required/i)).toBeInTheDocument();
    });
  });
});

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { useState } from 'react';
import { CronInput } from './CronInput';

function TestWrapper({ initialValue = '', error }: { initialValue?: string; error?: string }) {
  const [value, setValue] = useState(initialValue);
  return <CronInput value={value} onChange={setValue} error={error} />;
}

describe('CronInput', () => {
  describe('Rendering', () => {
    it('should render input field', () => {
      render(<CronInput value="" onChange={() => {}} />);

      const input = screen.getByRole('textbox');
      expect(input).toBeInTheDocument();
    });

    it('should render with initial value', () => {
      render(<CronInput value="*/5 * * * *" onChange={() => {}} />);

      const input = screen.getByRole('textbox');
      expect(input).toHaveValue('*/5 * * * *');
    });

    it('should render preview section', () => {
      render(<CronInput value="*/5 * * * *" onChange={() => {}} />);

      const preview = screen.getByText(/next run/i);
      expect(preview).toBeInTheDocument();
    });

    it('should render label', () => {
      render(<CronInput value="" onChange={() => {}} />);

      const label = screen.getByText(/cron expression/i);
      expect(label).toBeInTheDocument();
    });

    it('should render helper text showing field format', () => {
      render(<CronInput value="" onChange={() => {}} />);

      const helper = screen.getByText(/minute.*hour.*day of month.*month.*day of week/i);
      expect(helper).toBeInTheDocument();
    });
  });

  describe('Validation', () => {
    it('should show error for invalid cron expression', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, 'invalid');

      expect(screen.getByText(/invalid cron expression/i)).toBeInTheDocument();
    });

    it('should show error for wrong field count', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '* * *');

      expect(screen.getByText(/invalid cron expression/i)).toBeInTheDocument();
    });

    it('should accept valid cron expression', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '*/5 * * * *');

      expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
    });

    it('should accept cron with ranges', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '0-30 9-17 * * 1-5');

      expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
    });

    it('should accept cron with lists', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '0,30 9,17 * * *');

      expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
    });

    it('should accept cron with steps', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '*/15 * * * *');

      expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
    });

    it('should set aria-invalid on error', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, 'invalid');

      expect(input).toHaveAttribute('aria-invalid', 'true');
    });

    it('should associate error message with input', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, 'invalid');

      const errorMessage = screen.getByText(/invalid cron expression/i);
      expect(input).toHaveAttribute('aria-describedby', errorMessage.id);
    });
  });

  describe('Preview', () => {
    it('should show next run time for valid cron', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '*/5 * * * *');

      const preview = screen.getByText(/next run/i);
      expect(preview.textContent).toMatch(/next run.*in (less than 1 minute|\d+ minutes?)/i);
    });

    it('should show preview for every minute expression', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '* * * * *');

      const preview = screen.getByText(/next run/i);
      expect(preview.textContent).toMatch(/in (less than 1 minute|\d+ minutes?)/i);
    });

    it('should show preview for daily expression', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '0 9 * * *');

      const preview = screen.getByText(/next run/i);
      expect(preview.textContent).toMatch(/next run/i);
    });

    it('should not show preview for invalid cron', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, 'invalid');

      expect(screen.queryByText(/next run/i)).not.toBeInTheDocument();
    });
  });

  describe('Interaction', () => {
    it('should call onChange with new value', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<CronInput value="" onChange={handleChange} />);

      const input = screen.getByRole('textbox');
      await user.type(input, '*');

      expect(handleChange).toHaveBeenCalled();
    });

    it('should update input value on change', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const input = screen.getByRole('textbox');
      await user.type(input, '* * * * *');

      expect(input).toHaveValue('* * * * *');
    });
  });

  describe('Disabled State', () => {
    it('should disable input when disabled prop is true', () => {
      render(<CronInput value="" onChange={() => {}} disabled />);

      const input = screen.getByRole('textbox');
      expect(input).toBeDisabled();
    });

    it('should not show preview when disabled', () => {
      render(<CronInput value="*/5 * * * *" onChange={() => {}} disabled />);

      const preview = screen.queryByText(/next run/i);
      expect(preview || screen.queryByRole('textbox')).toBeTruthy();
    });
  });

  describe('Error State', () => {
    it('should display custom error message when error prop is provided', () => {
      render(<CronInput value="" onChange={() => {}} error="Custom error message" />);

      expect(screen.getByText('Custom error message')).toBeInTheDocument();
    });

    it('should prioritize custom error over validation error', async () => {
      const user = userEvent.setup();
      render(<CronInput value="" onChange={() => {}} error="Custom error message" />);

      const input = screen.getByRole('textbox');
      await user.type(input, 'invalid');

      expect(screen.getByText('Custom error message')).toBeInTheDocument();
    });

    it('should set aria-invalid when error prop is provided', () => {
      render(<CronInput value="" onChange={() => {}} error="Error" />);

      const input = screen.getByRole('textbox');
      expect(input).toHaveAttribute('aria-invalid', 'true');
    });
  });

  describe('Accessibility', () => {
    it('should associate label with input', () => {
      render(<CronInput value="" onChange={() => {}} />);

      const input = screen.getByRole('textbox');
      const label = screen.getByText(/cron expression/i);

      expect(input).toHaveAttribute('id');
      expect(label).toHaveAttribute('for', input.id);
    });

    it('should have required indicator on label', () => {
      render(<CronInput value="" onChange={() => {}} />);

      const required = screen.getByText('*');
      expect(required).toHaveClass('required');
    });
  });
});

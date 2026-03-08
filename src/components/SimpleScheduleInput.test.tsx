import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { useState } from 'react';
import { SimpleScheduleInput } from './SimpleScheduleInput';

function TestWrapper({
  initialValue = '',
  error,
  disabled,
}: {
  initialValue?: string;
  error?: string;
  disabled?: boolean;
}) {
  const [value, setValue] = useState(initialValue);
  return (
    <SimpleScheduleInput value={value} onChange={setValue} error={error} disabled={disabled} />
  );
}

describe('SimpleScheduleInput', () => {
  describe('Rendering', () => {
    it('should render schedule type selector', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const selector = screen.getByRole('combobox', { name: /simple schedule/i });
      expect(selector).toBeInTheDocument();
    });

    it('should render three schedule type options', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const selector = screen.getByRole('combobox', { name: /simple schedule/i });
      const options = selector.querySelectorAll('option');
      expect(options).toHaveLength(4);
    });

    it('should render interval options when type is interval', () => {
      render(<TestWrapper initialValue='{"type":"interval","value":5,"unit":"minutes"}' />);

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      expect(intervalSelector).toBeInTheDocument();
    });

    it('should render time input when type is daily', () => {
      render(<TestWrapper initialValue='{"type":"daily","time":"09:30"}' />);

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toBeInTheDocument();
      expect(timeInput).toHaveValue('09:30');
    });

    it('should render day selector and time input when type is weekly', () => {
      render(<TestWrapper initialValue='{"type":"weekly","day":"monday","time":"09:30"}' />);

      const daySelector = screen.getByRole('combobox', { name: /day/i });
      expect(daySelector).toBeInTheDocument();

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toBeInTheDocument();
    });

    it('should render label', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const label = screen.getByText(/simple schedule/i);
      expect(label).toBeInTheDocument();
    });
  });

  describe('Interval Type', () => {
    it('should generate correct JSON for every 5 minutes', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'interval');

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      await user.selectOptions(intervalSelector, '5_minutes');

      expect(handleChange).toHaveBeenCalledWith('{"type":"interval","value":5,"unit":"minutes"}');
    });

    it('should generate correct JSON for every 1 hour', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'interval');

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      await user.selectOptions(intervalSelector, '1_hours');

      expect(handleChange).toHaveBeenCalledWith('{"type":"interval","value":1,"unit":"hours"}');
    });

    it('should generate correct JSON for every 1 day', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'interval');

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      await user.selectOptions(intervalSelector, '1_days');

      expect(handleChange).toHaveBeenCalledWith('{"type":"interval","value":1,"unit":"days"}');
    });
  });

  describe('Daily Type', () => {
    it('should generate correct JSON for daily time', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'daily');

      const timeInput = screen.getByLabelText(/time/i) as HTMLInputElement;
      fireEvent.change(timeInput, { target: { value: '09:30' } });

      const calls = handleChange.mock.calls;
      const lastCall = calls[calls.length - 1];
      expect(lastCall[0]).toBe('{"type":"daily","time":"09:30"}');
    });

    it('should parse initial daily time correctly', () => {
      render(<TestWrapper initialValue='{"type":"daily","time":"14:45"}' />);

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toHaveValue('14:45');
    });
  });

  describe('Weekly Type', () => {
    it('should generate correct JSON for weekly schedule', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'weekly');

      const daySelector = screen.getByRole('combobox', { name: /day/i });
      await user.selectOptions(daySelector, 'monday');

      const timeInput = screen.getByLabelText(/time/i) as HTMLInputElement;
      fireEvent.change(timeInput, { target: { value: '09:30' } });

      const calls = handleChange.mock.calls;
      const lastCall = calls[calls.length - 1];
      expect(lastCall[0]).toBe('{"type":"weekly","day":"monday","time":"09:30"}');
    });

    it('should render all 7 days of week', async () => {
      render(<TestWrapper initialValue='{"type":"weekly","day":"monday","time":"09:00"}' />);

      const daySelector = screen.getByRole('combobox', { name: /day/i });
      const options = daySelector.querySelectorAll('option');
      expect(options).toHaveLength(7);
    });

    it('should parse initial weekly schedule correctly', () => {
      render(<TestWrapper initialValue='{"type":"weekly","day":"friday","time":"17:00"}' />);

      const daySelector = screen.getByRole('combobox', { name: /day/i });
      expect(daySelector).toHaveValue('friday');

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toHaveValue('17:00');
    });
  });

  describe('Interaction', () => {
    it('should call onChange when schedule type changes', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'daily');

      expect(handleChange).toHaveBeenCalled();
    });

    it('should update interval selector when type changes to interval', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'interval');

      expect(screen.getByRole('combobox', { name: /interval/i })).toBeInTheDocument();
    });

    it('should update time input when type changes to daily', async () => {
      const user = userEvent.setup();
      render(<TestWrapper />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'daily');

      expect(screen.getByLabelText(/time/i)).toBeInTheDocument();
    });
  });

  describe('Error State', () => {
    it('should display custom error message when error prop is provided', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} error="Invalid schedule" />);

      expect(screen.getByText('Invalid schedule')).toBeInTheDocument();
    });

    it('should set aria-invalid when error prop is provided', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} error="Error" />);

      const selector = screen.getByRole('combobox', { name: /simple schedule/i });
      expect(selector).toHaveAttribute('aria-invalid', 'true');
    });
  });

  describe('Disabled State', () => {
    it('should disable all inputs when disabled prop is true', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} disabled />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      expect(typeSelector).toBeDisabled();
    });

    it('should disable interval selector when disabled', () => {
      render(
        <TestWrapper initialValue='{"type":"interval","value":5,"unit":"minutes"}' disabled />
      );

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      expect(intervalSelector).toBeDisabled();
    });

    it('should disable time input when disabled', () => {
      render(<TestWrapper initialValue='{"type":"daily","time":"09:30"}' disabled />);

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toBeDisabled();
    });

    it('should disable day selector and time input when disabled', () => {
      render(
        <TestWrapper initialValue='{"type":"weekly","day":"monday","time":"09:30"}' disabled />
      );

      const daySelector = screen.getByRole('combobox', { name: /day/i });
      expect(daySelector).toBeDisabled();

      const timeInput = screen.getByLabelText(/time/i);
      expect(timeInput).toBeDisabled();
    });
  });

  describe('Accessibility', () => {
    it('should associate label with schedule type selector', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const selector = screen.getByRole('combobox', { name: /simple schedule/i });
      const label = screen.getByText(/simple schedule/i);

      expect(selector).toHaveAttribute('id');
      expect(label).toHaveAttribute('for', selector.id);
    });

    it('should have required indicator on label', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const required = screen.getByText('*');
      expect(required).toHaveClass('required');
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty value gracefully', () => {
      render(<SimpleScheduleInput value="" onChange={() => {}} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      expect(typeSelector).toHaveValue('');
    });

    it('should handle invalid JSON gracefully', () => {
      render(<SimpleScheduleInput value="invalid json" onChange={() => {}} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      expect(typeSelector).toHaveValue('');
    });

    it('should reset fields when type changes', async () => {
      const user = userEvent.setup();
      const handleChange = vi.fn();
      render(<SimpleScheduleInput value="" onChange={handleChange} />);

      const typeSelector = screen.getByRole('combobox', { name: /simple schedule/i });
      await user.selectOptions(typeSelector, 'interval');

      const intervalSelector = screen.getByRole('combobox', { name: /interval/i });
      await user.selectOptions(intervalSelector, '5_minutes');

      await user.selectOptions(typeSelector, 'daily');

      expect(screen.queryByRole('combobox', { name: /interval/i })).not.toBeInTheDocument();
      expect(screen.getByLabelText(/time/i)).toBeInTheDocument();
    });
  });
});

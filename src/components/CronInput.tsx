import { useState, useEffect, useMemo } from 'react';
import { CronJob } from 'cron';
import './CronInput.css';

interface CronInputProps {
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled?: boolean;
}

interface ValidationResult {
  isValid: boolean;
  error?: string;
}

function validateCronExpression(expression: string): ValidationResult {
  if (!expression || expression.trim() === '') {
    return { isValid: true };
  }

  const trimmed = expression.trim();
  const fields = trimmed.split(/\s+/);

  if (fields.length !== 5) {
    return {
      isValid: false,
      error: 'Invalid cron expression',
    };
  }

  try {
    const cronExpression = `0 ${trimmed}`;
    new CronJob(
      cronExpression,
      () => {},
      null,
      false,
      undefined,
      undefined,
      undefined,
      undefined,
      false
    );
    return { isValid: true };
  } catch (error) {
    return {
      isValid: false,
      error: 'Invalid cron expression',
    };
  }
}

function getNextRunTime(expression: string): Date | null {
  if (!expression || expression.trim() === '') {
    return null;
  }

  try {
    const cronExpression = `0 ${expression.trim()}`;
    const job = new CronJob(cronExpression, () => {}, null, false);
    const nextDate = job.nextDate();
    return nextDate.toJSDate();
  } catch {
    return null;
  }
}

function formatTimeUntil(date: Date): string {
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffMinutes = Math.round(diffMs / 60000);

  if (diffMinutes < 1) {
    return 'in less than 1 minute';
  } else if (diffMinutes === 1) {
    return 'in 1 minute';
  } else if (diffMinutes < 60) {
    return `in ${diffMinutes} minutes`;
  } else {
    const diffHours = Math.round(diffMinutes / 60);
    if (diffHours === 1) {
      return 'in 1 hour';
    } else if (diffHours < 24) {
      return `in ${diffHours} hours`;
    } else {
      const diffDays = Math.round(diffHours / 24);
      if (diffDays === 1) {
        return 'in 1 day';
      }
      return `in ${diffDays} days`;
    }
  }
}

export function CronInput({ value, onChange, error: externalError, disabled }: CronInputProps) {
  const [internalError, setInternalError] = useState<string | undefined>();
  const inputId = useMemo(() => `cron-input-${Math.random().toString(36).substr(2, 9)}`, []);

  const validation = useMemo(() => {
    return validateCronExpression(value);
  }, [value]);

  const nextRunTime = useMemo(() => {
    if (!validation.isValid || !value || disabled) {
      return null;
    }
    return getNextRunTime(value);
  }, [value, validation.isValid, disabled]);

  useEffect(() => {
    if (!value || value.trim() === '') {
      setInternalError(undefined);
    } else if (!validation.isValid) {
      setInternalError(validation.error);
    } else {
      setInternalError(undefined);
    }
  }, [value, validation]);

  const displayError = externalError || internalError;
  const hasError = Boolean(displayError);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  return (
    <div className="cron-input-container">
      <div className="form-field">
        <label htmlFor={inputId}>
          Cron Expression <span className="required">*</span>
        </label>
        <input
          id={inputId}
          type="text"
          value={value}
          onChange={handleChange}
          placeholder="*/5 * * * *"
          aria-invalid={hasError}
          aria-describedby={hasError ? `${inputId}-error` : undefined}
          disabled={disabled}
        />
        {hasError && (
          <span id={`${inputId}-error`} className="field-error">
            {displayError}
          </span>
        )}
        <span className="helper-text">Format: minute hour day of month month day of week</span>
      </div>

      {nextRunTime && !hasError && (
        <div className="cron-preview">
          <span className="preview-label">Next run: {formatTimeUntil(nextRunTime)}</span>
        </div>
      )}
    </div>
  );
}

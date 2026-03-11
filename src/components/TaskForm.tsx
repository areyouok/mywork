import { useState, useEffect, type SyntheticEvent } from 'react';
import type { Task } from '@/types/task';
import { CronInput } from './CronInput';
import { SimpleScheduleInput } from './SimpleScheduleInput';
import './TaskForm.css';

export interface TaskFormData {
  name: string;
  prompt: string;
  schedule_type: 'cron' | 'simple';
  cron_expression?: string;
  simple_schedule?: string;
  timeout_seconds: number;
}

interface TaskFormProps {
  initialData?: Task;
  onSubmit: (data: TaskFormData) => Promise<void>;
  onCancel?: () => void;
}

interface FormErrors {
  name?: string;
  prompt?: string;
  cron_expression?: string;
  simple_schedule?: string;
  timeout_seconds?: string;
  submit?: string;
}

export function TaskForm({ initialData, onSubmit, onCancel }: TaskFormProps) {
  const [name, setName] = useState(initialData?.name || '');
  const [prompt, setPrompt] = useState(initialData?.prompt || '');
  const [scheduleType, setScheduleType] = useState<'cron' | 'simple'>(
    initialData?.cron_expression ? 'cron' : 'simple'
  );
  const [cronExpression, setCronExpression] = useState(initialData?.cron_expression || '');
  const [simpleSchedule, setSimpleSchedule] = useState(initialData?.simple_schedule || '');
  const [timeoutSeconds, setTimeoutSeconds] = useState(initialData?.timeout_seconds || 300);
  const [errors, setErrors] = useState<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (initialData) {
      setName(initialData.name);
      setPrompt(initialData.prompt);
      setScheduleType(initialData.cron_expression ? 'cron' : 'simple');
      setCronExpression(initialData.cron_expression || '');
      setSimpleSchedule(initialData.simple_schedule || '');
      setTimeoutSeconds(initialData.timeout_seconds);
    }
  }, [initialData]);

  const validateCronExpression = (expression: string): boolean => {
    const cronRegex =
      /^(\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-3])) (\*|([1-9]|[12][0-9]|3[01])) (\*|([1-9]|1[0-2])) (\*|([0-6]))$/;
    return cronRegex.test(expression);
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!prompt.trim()) {
      newErrors.prompt = 'Prompt is required';
    }

    if (scheduleType === 'cron') {
      if (!cronExpression.trim()) {
        newErrors.cron_expression = 'Cron expression is required';
      } else if (!validateCronExpression(cronExpression)) {
        newErrors.cron_expression = 'Invalid cron expression';
      }
    } else {
      if (!simpleSchedule.trim()) {
        newErrors.simple_schedule = 'Simple schedule is required';
      }
    }

    if (timeoutSeconds < 1) {
      newErrors.timeout_seconds = 'timeout must be at least 1 second';
    } else if (timeoutSeconds > 3600) {
      newErrors.timeout_seconds = 'timeout must not exceed 3600 seconds';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);
    setErrors({});

    try {
      await onSubmit({
        name: name.trim(),
        prompt: prompt.trim(),
        schedule_type: scheduleType,
        cron_expression: scheduleType === 'cron' ? cronExpression.trim() : undefined,
        simple_schedule: scheduleType === 'simple' ? simpleSchedule.trim() : undefined,
        timeout_seconds: timeoutSeconds,
      });

      if (!initialData) {
        setName('');
        setPrompt('');
        setCronExpression('');
        setSimpleSchedule('');
        setTimeoutSeconds(300);
        setScheduleType('simple');
      } else {
        onCancel?.();
      }
    } catch (error) {
      setErrors({
        submit: error instanceof Error ? error.message : 'Submission failed',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form className="task-form" onSubmit={handleSubmit}>
      <div className="form-field">
        <label htmlFor="task-name">
          Task Name <span className="required">*</span>
        </label>
        <input
          id="task-name"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          aria-required="true"
          aria-invalid={!!errors.name}
          aria-describedby={errors.name ? 'name-error' : undefined}
          disabled={isSubmitting}
        />
        {errors.name && (
          <span id="name-error" className="field-error">
            {errors.name}
          </span>
        )}
      </div>

      <div className="form-field">
        <label htmlFor="prompt">
          Prompt <span className="required">*</span>
        </label>
        <textarea
          id="prompt"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          rows={3}
          aria-required="true"
          aria-invalid={!!errors.prompt}
          aria-describedby={errors.prompt ? 'prompt-error' : undefined}
          disabled={isSubmitting}
        />
        {errors.prompt && (
          <span id="prompt-error" className="field-error">
            {errors.prompt}
          </span>
        )}
      </div>

      <div className="form-field">
        <label>Schedule Type</label>
        <div className="radio-group">
          <label className="radio-label">
            <input
              type="radio"
              name="schedule-type"
              value="cron"
              checked={scheduleType === 'cron'}
              onChange={() => setScheduleType('cron')}
              disabled={isSubmitting}
            />
            <span>Cron</span>
          </label>
          <label className="radio-label">
            <input
              type="radio"
              name="schedule-type"
              value="simple"
              checked={scheduleType === 'simple'}
              onChange={() => setScheduleType('simple')}
              disabled={isSubmitting}
            />
            <span>Simple</span>
          </label>
        </div>
      </div>

      {scheduleType === 'cron' && (
        <CronInput
          value={cronExpression}
          onChange={setCronExpression}
          error={errors.cron_expression}
          disabled={isSubmitting}
        />
      )}

      {scheduleType === 'simple' && (
        <SimpleScheduleInput
          value={simpleSchedule}
          onChange={setSimpleSchedule}
          error={errors.simple_schedule}
          disabled={isSubmitting}
        />
      )}

      <div className="form-field">
        <label htmlFor="timeout">Timeout (seconds)</label>
        <input
          id="timeout"
          type="number"
          value={timeoutSeconds}
          onChange={(e) => {
            const parsed = Number.parseInt(e.target.value, 10);
            setTimeoutSeconds(Number.isNaN(parsed) ? 0 : parsed);
          }}
          aria-invalid={!!errors.timeout_seconds}
          aria-describedby={errors.timeout_seconds ? 'timeout-error' : undefined}
          disabled={isSubmitting}
        />
        {errors.timeout_seconds && (
          <span id="timeout-error" className="field-error">
            {errors.timeout_seconds}
          </span>
        )}
      </div>

      {errors.submit && (
        <div className="form-error" role="alert">
          {errors.submit}
        </div>
      )}

      <div className="form-actions">
        {onCancel && (
          <button type="button" className="btn-cancel" onClick={onCancel} disabled={isSubmitting}>
            Cancel
          </button>
        )}
        <button type="submit" className="btn-submit" disabled={isSubmitting}>
          {isSubmitting
            ? initialData
              ? 'Updating...'
              : 'Creating...'
            : initialData
              ? 'Update Task'
              : 'Create Task'}
        </button>
      </div>
    </form>
  );
}

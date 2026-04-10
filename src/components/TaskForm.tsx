import { useState, useEffect, type SyntheticEvent } from 'react';
import type { Task } from '@/types/task';
import { CronInput } from './CronInput';
import { SimpleScheduleInput } from './SimpleScheduleInput';
import './TaskForm.css';

export interface TaskFormData {
  name: string;
  prompt: string;
  schedule_type: 'cron' | 'simple' | 'once';
  cron_expression?: string;
  simple_schedule?: string;
  once_at?: string;
  timeout_seconds: number;
  working_directory?: string;
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
  once_at?: string;
  timeout_seconds?: string;
  working_directory?: string;
  submit?: string;
}

function detectScheduleType(task?: Task): 'cron' | 'simple' | 'once' {
  if (task?.cron_expression) {
    return 'cron';
  }
  if (task?.once_at) {
    return 'once';
  }
  return 'simple';
}

function toLocalDateTimeInputValue(iso?: string): string {
  if (!iso) {
    return '';
  }

  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return '';
  }

  const pad = (n: number) => n.toString().padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function TaskForm({ initialData, onSubmit, onCancel }: TaskFormProps) {
  const [name, setName] = useState(initialData?.name || '');
  const [prompt, setPrompt] = useState(initialData?.prompt || '');
  const [scheduleType, setScheduleType] = useState<'cron' | 'simple' | 'once'>(
    detectScheduleType(initialData)
  );
  const [cronExpression, setCronExpression] = useState(initialData?.cron_expression || '');
  const [simpleSchedule, setSimpleSchedule] = useState(initialData?.simple_schedule || '');
  const [onceAt, setOnceAt] = useState(toLocalDateTimeInputValue(initialData?.once_at));
  const [timeoutSeconds, setTimeoutSeconds] = useState(initialData?.timeout_seconds || 300);
  const [workingDirectory, setWorkingDirectory] = useState(initialData?.working_directory || '');
  const [errors, setErrors] = useState<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (initialData) {
      setName(initialData.name);
      setPrompt(initialData.prompt);
      setScheduleType(detectScheduleType(initialData));
      setCronExpression(initialData.cron_expression || '');
      setSimpleSchedule(initialData.simple_schedule || '');
      setOnceAt(toLocalDateTimeInputValue(initialData.once_at));
      setTimeoutSeconds(initialData.timeout_seconds);
      setWorkingDirectory(initialData.working_directory || '');
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
    } else if (scheduleType === 'simple') {
      if (!simpleSchedule.trim()) {
        newErrors.simple_schedule = 'Simple schedule is required';
      }
    } else {
      if (!onceAt.trim()) {
        newErrors.once_at = 'Run time is required';
      } else {
        const runAt = new Date(onceAt);
        if (Number.isNaN(runAt.getTime())) {
          newErrors.once_at = 'Invalid run time';
        } else if (runAt.getTime() <= Date.now()) {
          newErrors.once_at = 'Run time must be in the future';
        }
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
        once_at: scheduleType === 'once' ? new Date(onceAt).toISOString() : undefined,
        timeout_seconds: timeoutSeconds,
        working_directory: workingDirectory.trim() || undefined,
      });

      if (!initialData) {
        setName('');
        setPrompt('');
        setCronExpression('');
        setSimpleSchedule('');
        setOnceAt('');
        setTimeoutSeconds(300);
        setWorkingDirectory('');
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
          <label className="radio-label">
            <input
              type="radio"
              name="schedule-type"
              value="once"
              checked={scheduleType === 'once'}
              onChange={() => setScheduleType('once')}
              disabled={isSubmitting}
            />
            <span>Once</span>
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

      {scheduleType === 'once' && (
        <div className="form-field">
          <label htmlFor="once-at">
            Run At <span className="required">*</span>
          </label>
          <input
            id="once-at"
            type="datetime-local"
            value={onceAt}
            onChange={(e) => setOnceAt(e.target.value)}
            aria-invalid={!!errors.once_at}
            aria-describedby={errors.once_at ? 'once-at-error' : undefined}
            disabled={isSubmitting}
          />
          {errors.once_at && (
            <span id="once-at-error" className="field-error">
              {errors.once_at}
            </span>
          )}
        </div>
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

      <div className="form-field">
        <label htmlFor="working-directory">
          Working Directory
          <span className="field-hint"> (optional, uses default if empty or invalid)</span>
        </label>
        <input
          id="working-directory"
          type="text"
          value={workingDirectory}
          onChange={(e) => setWorkingDirectory(e.target.value)}
          placeholder="e.g., /Users/username/projects/my-project"
          aria-invalid={!!errors.working_directory}
          aria-describedby={errors.working_directory ? 'working-directory-error' : undefined}
          disabled={isSubmitting}
        />
        {errors.working_directory && (
          <span id="working-directory-error" className="field-error">
            {errors.working_directory}
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

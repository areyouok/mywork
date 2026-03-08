import { useState, useEffect } from 'react';
import type { Task } from '@/types/task';
import './TaskForm.css';

export interface TaskFormData {
  name: string;
  prompt: string;
  schedule_type: 'cron' | 'simple';
  cron_expression?: string;
  simple_schedule?: string;
  timeout_seconds: number;
  skip_if_running: boolean;
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
  const [cronExpression, setCronExpression] = useState(
    initialData?.cron_expression || ''
  );
  const [simpleSchedule, setSimpleSchedule] = useState(
    initialData?.simple_schedule || ''
  );
  const [timeoutSeconds, setTimeoutSeconds] = useState(
    initialData?.timeout_seconds || 300
  );
  const [skipIfRunning, setSkipIfRunning] = useState(
    initialData?.skip_if_running || false
  );
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
      setSkipIfRunning(initialData.skip_if_running);
    }
  }, [initialData]);

  const validateCronExpression = (expression: string): boolean => {
    const cronRegex = /^(\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-3])) (\*|([1-9]|[12][0-9]|3[01])) (\*|([1-9]|1[0-2])) (\*|([0-6]))$/;
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

  const handleSubmit = async (e: React.FormEvent) => {
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
        skip_if_running: skipIfRunning,
      });

      if (!initialData) {
        setName('');
        setPrompt('');
        setCronExpression('');
        setSimpleSchedule('');
        setTimeoutSeconds(300);
        setSkipIfRunning(false);
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
        <div className="form-field">
          <label htmlFor="cron-expression">
            Cron Expression <span className="required">*</span>
          </label>
          <input
            id="cron-expression"
            type="text"
            value={cronExpression}
            onChange={(e) => setCronExpression(e.target.value)}
            placeholder="0 9 * * *"
            aria-invalid={!!errors.cron_expression}
            aria-describedby={errors.cron_expression ? 'cron-error' : undefined}
            disabled={isSubmitting}
          />
          {errors.cron_expression && (
            <span id="cron-error" className="field-error">
              {errors.cron_expression}
            </span>
          )}
        </div>
      )}

      {scheduleType === 'simple' && (
        <div className="form-field">
          <label htmlFor="simple-schedule">
            Simple Schedule <span className="required">*</span>
          </label>
          <input
            id="simple-schedule"
            type="text"
            value={simpleSchedule}
            onChange={(e) => setSimpleSchedule(e.target.value)}
            placeholder='{"type":"daily","time":"09:30"}'
            aria-invalid={!!errors.simple_schedule}
            aria-describedby={errors.simple_schedule ? 'simple-error' : undefined}
            disabled={isSubmitting}
          />
          {errors.simple_schedule && (
            <span id="simple-error" className="field-error">
              {errors.simple_schedule}
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
          onChange={(e) => setTimeoutSeconds(parseInt(e.target.value) || 0)}
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
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={skipIfRunning}
            onChange={(e) => setSkipIfRunning(e.target.checked)}
            disabled={isSubmitting}
          />
          <span>Skip if running</span>
        </label>
      </div>

      {errors.submit && (
        <div className="form-error" role="alert">
          {errors.submit}
        </div>
      )}

      <div className="form-actions">
        {onCancel && (
          <button
            type="button"
            className="btn-cancel"
            onClick={onCancel}
            disabled={isSubmitting}
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          className="btn-submit"
          disabled={isSubmitting}
        >
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

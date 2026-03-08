import { useState, useEffect, useMemo, useId } from 'react';
import './SimpleScheduleInput.css';

interface SimpleScheduleInputProps {
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled?: boolean;
}

type ScheduleType = '' | 'interval' | 'daily' | 'weekly';

interface IntervalSchedule {
  type: 'interval';
  value: number;
  unit: 'minutes' | 'hours' | 'days';
}

interface DailySchedule {
  type: 'daily';
  time: string;
}

interface WeeklySchedule {
  type: 'weekly';
  day: string;
  time: string;
}

type Schedule = IntervalSchedule | DailySchedule | WeeklySchedule;

const INTERVAL_OPTIONS = [
  { value: '5_minutes', label: 'Every 5 minutes', json: { value: 5, unit: 'minutes' as const } },
  { value: '10_minutes', label: 'Every 10 minutes', json: { value: 10, unit: 'minutes' as const } },
  { value: '15_minutes', label: 'Every 15 minutes', json: { value: 15, unit: 'minutes' as const } },
  { value: '30_minutes', label: 'Every 30 minutes', json: { value: 30, unit: 'minutes' as const } },
  { value: '1_hours', label: 'Every 1 hour', json: { value: 1, unit: 'hours' as const } },
  { value: '2_hours', label: 'Every 2 hours', json: { value: 2, unit: 'hours' as const } },
  { value: '6_hours', label: 'Every 6 hours', json: { value: 6, unit: 'hours' as const } },
  { value: '12_hours', label: 'Every 12 hours', json: { value: 12, unit: 'hours' as const } },
  { value: '1_days', label: 'Every 1 day', json: { value: 1, unit: 'days' as const } },
];

const DAYS_OF_WEEK = [
  { value: 'monday', label: 'Monday' },
  { value: 'tuesday', label: 'Tuesday' },
  { value: 'wednesday', label: 'Wednesday' },
  { value: 'thursday', label: 'Thursday' },
  { value: 'friday', label: 'Friday' },
  { value: 'saturday', label: 'Saturday' },
  { value: 'sunday', label: 'Sunday' },
];

function parseSchedule(value: string): { type: ScheduleType; schedule?: Schedule } {
  if (!value || value.trim() === '') {
    return { type: '' };
  }

  try {
    const schedule = JSON.parse(value) as Schedule;
    return { type: schedule.type, schedule };
  } catch {
    return { type: '' };
  }
}

export function SimpleScheduleInput({
  value,
  onChange,
  error: externalError,
  disabled,
}: SimpleScheduleInputProps) {
  const inputId = useId();

  const parsed = useMemo(() => parseSchedule(value), [value]);
  const [scheduleType, setScheduleType] = useState<ScheduleType>(parsed.type);

  const [intervalValue, setIntervalValue] = useState('');
  const [dailyTime, setDailyTime] = useState('09:00');
  const [weeklyDay, setWeeklyDay] = useState('monday');
  const [weeklyTime, setWeeklyTime] = useState('09:00');

  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    setScheduleType(parsed.type);

    if (parsed.schedule) {
      if (parsed.schedule.type === 'interval') {
        const key = `${parsed.schedule.value}_${parsed.schedule.unit}`;
        setIntervalValue(key);
      } else if (parsed.schedule.type === 'daily') {
        setDailyTime(parsed.schedule.time);
      } else if (parsed.schedule.type === 'weekly') {
        setWeeklyDay(parsed.schedule.day);
        setWeeklyTime(parsed.schedule.time);
      }
    }
  }, [parsed]);
  /* eslint-enable react-hooks/set-state-in-effect */

  const handleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newType = e.target.value as ScheduleType;
    setScheduleType(newType);

    if (newType === 'interval') {
      setIntervalValue('5_minutes');
      const option = INTERVAL_OPTIONS.find((opt) => opt.value === '5_minutes');
      if (option) {
        const schedule: IntervalSchedule = {
          type: 'interval',
          ...option.json,
        };
        onChange(JSON.stringify(schedule));
      }
    } else if (newType === 'daily') {
      setDailyTime('09:00');
      const schedule: DailySchedule = {
        type: 'daily',
        time: '09:00',
      };
      onChange(JSON.stringify(schedule));
    } else if (newType === 'weekly') {
      setWeeklyDay('monday');
      setWeeklyTime('09:00');
      const schedule: WeeklySchedule = {
        type: 'weekly',
        day: 'monday',
        time: '09:00',
      };
      onChange(JSON.stringify(schedule));
    } else {
      onChange('');
    }
  };

  const handleIntervalChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const key = e.target.value;
    setIntervalValue(key);

    const option = INTERVAL_OPTIONS.find((opt) => opt.value === key);
    if (option) {
      const schedule: IntervalSchedule = {
        type: 'interval',
        ...option.json,
      };
      onChange(JSON.stringify(schedule));
    }
  };

  const handleDailyTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const time = e.target.value;
    setDailyTime(time);

    const schedule: DailySchedule = {
      type: 'daily',
      time,
    };
    onChange(JSON.stringify(schedule));
  };

  const handleWeeklyDayChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const day = e.target.value;
    setWeeklyDay(day);

    const schedule: WeeklySchedule = {
      type: 'weekly',
      day,
      time: weeklyTime,
    };
    onChange(JSON.stringify(schedule));
  };

  const handleWeeklyTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const time = e.target.value;
    setWeeklyTime(time);

    const schedule: WeeklySchedule = {
      type: 'weekly',
      day: weeklyDay,
      time,
    };
    onChange(JSON.stringify(schedule));
  };

  const hasError = Boolean(externalError);

  return (
    <div className="simple-schedule-input-container">
      <div className="form-field">
        <label htmlFor={inputId}>
          Simple Schedule <span className="required">*</span>
        </label>
        <select
          id={inputId}
          value={scheduleType}
          onChange={handleTypeChange}
          aria-invalid={hasError}
          aria-describedby={hasError ? `${inputId}-error` : undefined}
          disabled={disabled}
        >
          <option value="">Select schedule type</option>
          <option value="interval">Interval</option>
          <option value="daily">Daily</option>
          <option value="weekly">Weekly</option>
        </select>
        {hasError && (
          <span id={`${inputId}-error`} className="field-error">
            {externalError}
          </span>
        )}

        {scheduleType === 'interval' && (
          <div className="schedule-fields">
            <div className="form-field">
              <label htmlFor={`${inputId}-interval`}>Interval</label>
              <select
                id={`${inputId}-interval`}
                value={intervalValue}
                onChange={handleIntervalChange}
                disabled={disabled}
              >
                {INTERVAL_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </div>
        )}

        {scheduleType === 'daily' && (
          <div className="schedule-fields">
            <div className="form-field">
              <label htmlFor={`${inputId}-time`}>Time (24h)</label>
              <input
                id={`${inputId}-time`}
                type="time"
                value={dailyTime}
                onChange={handleDailyTimeChange}
                disabled={disabled}
              />
            </div>
          </div>
        )}

        {scheduleType === 'weekly' && (
          <div className="schedule-fields">
            <div className="form-field">
              <label htmlFor={`${inputId}-day`}>Day</label>
              <select
                id={`${inputId}-day`}
                value={weeklyDay}
                onChange={handleWeeklyDayChange}
                disabled={disabled}
              >
                {DAYS_OF_WEEK.map((day) => (
                  <option key={day.value} value={day.value}>
                    {day.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-field">
              <label htmlFor={`${inputId}-weekly-time`}>Time (24h)</label>
              <input
                id={`${inputId}-weekly-time`}
                type="time"
                value={weeklyTime}
                onChange={handleWeeklyTimeChange}
                disabled={disabled}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

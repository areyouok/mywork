import { useState } from 'react';
import type { TaskListProps, Task } from '@/types/task';
import './TaskList.css';

export function TaskList({ tasks, onToggle, onDelete }: TaskListProps) {
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  if (tasks.length === 0) {
    return (
      <div className="task-list-empty">
        <div className="empty-icon">📋</div>
        <h3>No tasks yet</h3>
        <p>Create your first task!</p>
      </div>
    );
  }

  const handleToggle = (task: Task) => {
    onToggle?.(task.id, !task.enabled);
  };

  const handleDeleteClick = (taskId: string) => {
    setConfirmDelete(taskId);
  };

  const handleConfirmDelete = (taskId: string) => {
    onDelete?.(taskId);
    setConfirmDelete(null);
  };

  const handleCancelDelete = () => {
    setConfirmDelete(null);
  };

  const formatSchedule = (task: Task): string => {
    if (task.cron_expression) {
      return task.cron_expression;
    }
    if (task.simple_schedule) {
      try {
        const schedule = JSON.parse(task.simple_schedule);

        if (schedule.type === 'interval') {
          return `Every ${schedule.value} ${schedule.unit}`;
        } else if (schedule.type === 'daily') {
          return `Daily at ${schedule.time}`;
        } else if (schedule.type === 'weekly') {
          const dayMap: Record<string, string> = {
            monday: 'Mon',
            tuesday: 'Tue',
            wednesday: 'Wed',
            thursday: 'Thu',
            friday: 'Fri',
            saturday: 'Sat',
            sunday: 'Sun',
          };
          const day = dayMap[schedule.day.toLowerCase()] || schedule.day;
          return `${day} at ${schedule.time}`;
        }

        return JSON.stringify(schedule);
      } catch {
        return 'Custom schedule';
      }
    }
    return 'No schedule';
  };

  return (
    <div className="task-list" role="list">
      {tasks.map((task) => (
        <div key={task.id} className="task-item" role="listitem">
          <div className="task-header">
            <h3 className="task-name">{task.name}</h3>
            <button
              role="switch"
              aria-checked={task.enabled}
              aria-label={`Toggle ${task.name}`}
              className={`toggle-switch ${task.enabled ? 'enabled' : ''}`}
              onClick={() => handleToggle(task)}
            >
              <span className="toggle-slider" />
            </button>
          </div>

          <div className="task-body">
            <p className="task-prompt">{task.prompt}</p>
            <div className="task-meta">
              <span className="task-schedule">{formatSchedule(task)}</span>
              {task.timeout_seconds !== 300 && (
                <span className="task-timeout">Timeout: {task.timeout_seconds}s</span>
              )}
            </div>
          </div>

          <div className="task-actions">
            {confirmDelete === task.id ? (
              <div className="confirm-delete">
                <span>Are you sure?</span>
                <button className="btn-confirm" onClick={() => handleConfirmDelete(task.id)}>
                  Confirm
                </button>
                <button className="btn-cancel" onClick={handleCancelDelete}>
                  Cancel
                </button>
              </div>
            ) : (
              <button
                className="btn-delete"
                aria-label={`Delete ${task.name}`}
                onClick={() => handleDeleteClick(task.id)}
              >
                Delete
              </button>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}

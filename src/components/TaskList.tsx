import { useState } from 'react';
import type { TaskListProps, Task } from '@/types/task';
import { formatOnceAt, formatSimpleSchedule } from '@/utils/format';
import './TaskList.css';

export function TaskList({
  tasks,
  runningTaskIds,
  onToggle,
  onDelete,
  onRun,
  onEdit,
  onHistory,
}: TaskListProps) {
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
    if (task.once_at) {
      return formatOnceAt(task.once_at);
    }
    return formatSimpleSchedule(task.simple_schedule);
  };

  return (
    <div className="task-list" role="list">
      {tasks.map((task) => (
        <div key={task.id} className="task-item" role="listitem">
          <div className="task-header">
            <div className="task-schedule-header">{formatSchedule(task)}</div>
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
            <textarea
              className="task-prompt"
              readOnly
              value={task.prompt}
              rows={12}
              tabIndex={-1}
            />
            {task.timeout_seconds !== 300 && (
              <div className="task-meta">
                <span className="task-timeout">Timeout: {task.timeout_seconds}s</span>
              </div>
            )}
          </div>

          <div className="task-actions">
            <div className="action-group primary">
              <button
                className={`btn-run ${runningTaskIds?.has(task.id) ? 'running' : ''}`}
                aria-label={`Run ${task.name}`}
                onClick={() => onRun?.(task.id)}
                disabled={runningTaskIds?.has(task.id)}
              >
                {runningTaskIds?.has(task.id) ? (
                  <>
                    <span className="spinner"></span>
                    Running...
                  </>
                ) : (
                  'Run'
                )}
              </button>
              <button
                className="btn-edit"
                aria-label={`Edit ${task.name}`}
                onClick={() => onEdit?.(task)}
              >
                Edit
              </button>
              <button
                className="btn-history"
                aria-label={`View history for ${task.name}`}
                onClick={() => onHistory?.(task)}
              >
                History
              </button>
            </div>
            <div className="action-group secondary">
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
        </div>
      ))}
    </div>
  );
}

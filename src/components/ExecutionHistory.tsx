import { useEffect, useRef } from 'react';
import { formatRelativeTime, formatDuration } from '@/utils/format';
import type { ExecutionHistoryProps, Execution } from '@/types/execution';
import './ExecutionHistory.css';

export function ExecutionHistory({
  executions,
  onViewOutput,
  taskId,
  loading,
  onRefresh,
}: ExecutionHistoryProps) {
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    const hasRunning = executions.some((e) => e.status === 'running');

    if (hasRunning && onRefresh) {
      intervalRef.current = setInterval(() => {
        onRefresh();
      }, 500);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [executions, onRefresh]);

  if (loading) {
    return (
      <div className="execution-history-loading">
        <div className="loading-spinner" role="status" aria-label="Loading execution history" />
        <p>Loading...</p>
      </div>
    );
  }

  const filteredExecutions = taskId
    ? executions.filter((execution) => execution.task_id === taskId)
    : executions;

  if (filteredExecutions.length === 0) {
    return (
      <div className="execution-history-empty">
        <div className="empty-icon">📊</div>
        <h3>No execution history</h3>
        <p>Run this task to see execution history</p>
      </div>
    );
  }

  const handleClick = (execution: Execution) => {
    if (onViewOutput && execution.output_file) {
      onViewOutput(execution);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent, execution: Execution) => {
    if (event.key === 'Enter' && onViewOutput && execution.output_file) {
      onViewOutput(execution);
    }
  };

  const getStatusLabel = (status: Execution['status']): string => {
    return status;
  };

  return (
    <div className="execution-history" role="list">
      {filteredExecutions.map((execution) => {
        const isClickable = onViewOutput && execution.output_file;
        const isRunning = execution.status === 'running' && !execution.finished_at;

        return (
          <div
            key={execution.id}
            className={`execution-item ${isClickable ? 'clickable' : ''}`}
            role="listitem"
            tabIndex={isClickable ? 0 : undefined}
            onClick={() => handleClick(execution)}
            onKeyPress={(e) => handleKeyPress(e, execution)}
            aria-label={`Execution ${getStatusLabel(execution.status)} at ${formatRelativeTime(execution.started_at)}`}
          >
            <div className="execution-header">
              <span className={`execution-status status-${execution.status}`}>
                {execution.status}
              </span>
              <span className="execution-time">{formatRelativeTime(execution.started_at)}</span>
            </div>

            <div className="execution-duration">
              {isRunning
                ? 'Running...'
                : execution.finished_at
                  ? `Duration: ${formatDuration(execution.started_at, execution.finished_at)}`
                  : null}
            </div>

            {execution.status === 'failed' && execution.error_message && (
              <div className="execution-error">{execution.error_message}</div>
            )}
          </div>
        );
      })}
    </div>
  );
}

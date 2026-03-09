import type { ExecutionHistoryProps, Execution } from '@/types/execution';
import './ExecutionHistory.css';

export function ExecutionHistory({
  executions,
  onViewOutput,
  taskId,
  loading,
}: ExecutionHistoryProps) {
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

  const formatTime = (dateString: string): string => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = diffMs / (1000 * 60 * 60);
    const diffDays = diffMs / (1000 * 60 * 60 * 24);

    if (diffHours < 1) {
      return 'less than 1 minute ago';
    }
    if (diffHours < 2) {
      return '1 hour ago';
    }
    if (diffHours < 24) {
      return `${Math.floor(diffHours)} hours ago`;
    }
    if (diffDays < 7) {
      return date.toLocaleString('en-US', {
        weekday: 'short',
        hour: '2-digit',
        minute: '2-digit',
      });
    }

    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const formatDuration = (startedAt: string, finishedAt: string): string => {
    const start = new Date(startedAt);
    const finish = new Date(finishedAt);
    const diffMs = finish.getTime() - start.getTime();

    if (diffMs < 1000) {
      return '<1s';
    }

    const diffSeconds = Math.floor(diffMs / 1000);
    const diffMinutes = Math.floor(diffSeconds / 60);
    const diffHours = Math.floor(diffMinutes / 60);

    if (diffHours > 0) {
      const remainingMinutes = diffMinutes % 60;
      return remainingMinutes > 0 ? `${diffHours}h ${remainingMinutes}m` : `${diffHours}h`;
    }

    if (diffMinutes > 0) {
      const remainingSeconds = diffSeconds % 60;
      return remainingSeconds > 0 ? `${diffMinutes}m ${remainingSeconds}s` : `${diffMinutes}m`;
    }

    return `${diffSeconds}s`;
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
            aria-label={`Execution ${getStatusLabel(execution.status)} at ${formatTime(execution.started_at)}`}
          >
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

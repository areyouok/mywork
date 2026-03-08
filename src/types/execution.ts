export type ExecutionStatus = 'pending' | 'running' | 'success' | 'failed' | 'timeout' | 'skipped';

export interface Execution {
  id: string;
  task_id: string;
  session_id?: string;
  status: ExecutionStatus;
  started_at: string;
  finished_at?: string;
  output_file?: string;
  error_message?: string;
}

export interface ExecutionHistoryProps {
  executions: Execution[];
  onViewOutput?: (execution: Execution) => void;
  taskId?: string;
  loading?: boolean;
}

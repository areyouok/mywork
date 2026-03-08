export interface Task {
  id: string;
  name: string;
  prompt: string;
  cron_expression?: string;
  simple_schedule?: string;
  enabled: boolean;
  timeout_seconds: number;
  skip_if_running: boolean;
  created_at: string;
  updated_at: string;
}

export interface TaskListProps {
  tasks: Task[];
  onToggle?: (taskId: string, enabled: boolean) => void;
  onDelete?: (taskId: string) => void;
}

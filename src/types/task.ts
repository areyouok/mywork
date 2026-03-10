export interface Task {
  id: string;
  name: string;
  prompt: string;
  cron_expression?: string;
  simple_schedule?: string;
  enabled: boolean;
  timeout_seconds: number;
  created_at: string;
  updated_at: string;
}

export interface TaskListProps {
  tasks: Task[];
  runningTaskIds?: Set<string>;
  onToggle?: (taskId: string, enabled: boolean) => void;
  onDelete?: (taskId: string) => void;
  onRun?: (taskId: string) => void;
  onEdit?: (task: Task) => void;
  onHistory?: (task: Task) => void;
}

import { invoke } from '@tauri-apps/api/core';
import type { Task } from '../types/task';
import type { Execution } from '../types/execution';

interface NewTask {
  name: string;
  prompt: string;
  cron_expression?: string;
  simple_schedule?: string;
  enabled?: number;
  timeout_seconds?: number;
  skip_if_running?: number;
}

interface UpdateTask {
  name?: string;
  prompt?: string;
  cron_expression?: string;
  simple_schedule?: string;
  enabled?: number;
  timeout_seconds?: number;
  skip_if_running?: number;
}

interface NewExecution {
  task_id: string;
  session_id?: string;
  status?: 'pending' | 'running' | 'success' | 'failed' | 'timeout' | 'skipped';
  output_file?: string;
  error_message?: string;
}

interface UpdateExecution {
  session_id?: string;
  status?: 'pending' | 'running' | 'success' | 'failed' | 'timeout' | 'skipped';
  finished_at?: string;
  output_file?: string;
  error_message?: string;
}

interface RawTask {
  id: string;
  name: string;
  prompt: string;
  cron_expression?: string;
  simple_schedule?: string;
  enabled: number;
  timeout_seconds: number;
  skip_if_running: number;
  created_at: string;
  updated_at: string;
}

function convertTask(raw: RawTask): Task {
  return {
    ...raw,
    enabled: raw.enabled === 1,
    skip_if_running: raw.skip_if_running === 1,
  };
}

export async function getTasks(): Promise<Task[]> {
  const tasks = await invoke<RawTask[]>('get_tasks');
  return tasks.map(convertTask);
}

export async function getTask(id: string): Promise<Task> {
  const task = await invoke<RawTask>('get_task', { id });
  return {
    ...task,
    enabled: task.enabled === 1,
    skip_if_running: task.skip_if_running === 1,
  };
}

export async function createTask(newTask: NewTask): Promise<Task> {
  const task = await invoke<RawTask>('create_task', { newTask });
  return {
    ...task,
    enabled: task.enabled === 1,
    skip_if_running: task.skip_if_running === 1,
  };
}

export async function updateTask(id: string, update: UpdateTask): Promise<Task> {
  const task = await invoke<RawTask>('update_task', { id, update });
  return {
    ...task,
    enabled: task.enabled === 1,
    skip_if_running: task.skip_if_running === 1,
  };
}

export async function deleteTask(id: string): Promise<boolean> {
  return await invoke<boolean>('delete_task', { id });
}

export async function getExecutions(taskId: string): Promise<Execution[]> {
  return await invoke<Execution[]>('get_executions', { taskId });
}

export async function getExecution(id: string): Promise<Execution> {
  return await invoke<Execution>('get_execution', { id });
}

export async function createExecution(newExecution: NewExecution): Promise<Execution> {
  return await invoke<Execution>('create_execution', { newExecution });
}

export async function updateExecution(id: string, update: UpdateExecution): Promise<Execution> {
  return await invoke<Execution>('update_execution', { id, update });
}

export async function startScheduler(): Promise<string> {
  return await invoke<string>('start_scheduler');
}

export async function stopScheduler(): Promise<string> {
  return await invoke<string>('stop_scheduler');
}

export async function getSchedulerStatus(): Promise<string> {
  return await invoke<string>('get_scheduler_status');
}

export async function getOutput(executionId: string): Promise<string> {
  return await invoke<string>('get_output', { executionId });
}

export async function deleteOutput(executionId: string): Promise<boolean> {
  return await invoke<boolean>('delete_output', { executionId });
}

import { useState, useCallback, useRef } from 'react';
import * as api from '@/api/tasks';
import type { Task } from '@/types/task';

interface NewTask {
  name: string;
  prompt: string;
  cron_expression?: string;
  simple_schedule?: string;
  once_at?: string;
  enabled?: number;
  timeout_seconds?: number;
}

interface UpdateTask {
  schedule_type?: 'cron' | 'simple' | 'once';
  name?: string;
  prompt?: string;
  cron_expression?: string | null;
  simple_schedule?: string | null;
  once_at?: string | null;
  enabled?: number;
  timeout_seconds?: number;
}

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const requestIdRef = useRef(0);

  const loadTasks = useCallback(async () => {
    const requestId = ++requestIdRef.current;

    try {
      setLoading(true);
      const loadedTasks = await api.getTasks();
      if (requestIdRef.current === requestId) {
        setTasks(loadedTasks);
      }
    } catch (error) {
      console.error('Failed to load tasks:', error);
    } finally {
      if (requestIdRef.current === requestId) {
        setLoading(false);
      }
    }
  }, []);

  const createTask = useCallback(async (data: NewTask) => {
    const requestId = ++requestIdRef.current;

    try {
      const newTask = await api.createTask(data);
      if (requestIdRef.current === requestId) {
        setTasks((prev) => [...prev, newTask]);
      }

      try {
        await api.reloadScheduler();
      } catch (error) {
        console.error('Failed to reload scheduler after creating task:', error);
      }

      return newTask;
    } catch (error) {
      console.error('Failed to create task:', error);
      throw error;
    }
  }, []);

  const updateTask = useCallback(async (id: string, data: UpdateTask) => {
    const requestId = ++requestIdRef.current;

    try {
      const updated = await api.updateTask(id, data);
      if (requestIdRef.current === requestId) {
        setTasks((prev) => {
          const newTasks = prev.map((task) => (task.id === id ? updated : task));
          return newTasks.sort(
            (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
          );
        });
      }

      try {
        await api.reloadScheduler();
      } catch (error) {
        console.error('Failed to reload scheduler after updating task:', error);
      }

      return updated;
    } catch (error) {
      console.error('Failed to update task:', error);
      throw error;
    }
  }, []);

  const deleteTask = useCallback(async (id: string) => {
    const requestId = ++requestIdRef.current;

    try {
      await api.deleteTask(id);
      if (requestIdRef.current === requestId) {
        setTasks((prev) => prev.filter((task) => task.id !== id));
      }
    } catch (error) {
      console.error('Failed to delete task:', error);
      throw error;
    }
  }, []);

  return { tasks, loading, loadTasks, createTask, updateTask, deleteTask };
}

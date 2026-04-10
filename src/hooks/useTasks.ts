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
  working_directory?: string;
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
  working_directory?: string | null;
}

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const loadRequestIdRef = useRef(0);
  const mutationVersionRef = useRef(0);
  const createRequestIdRef = useRef(0);
  const updateRequestIdRef = useRef(0);
  const deleteRequestIdRef = useRef(0);

  const loadTasks = useCallback(async () => {
    const requestId = ++loadRequestIdRef.current;
    const mutationVersionAtStart = mutationVersionRef.current;

    try {
      setLoading(true);
      const loadedTasks = await api.getTasks();
      if (
        loadRequestIdRef.current === requestId &&
        mutationVersionRef.current === mutationVersionAtStart
      ) {
        setTasks(loadedTasks);
      }
    } catch (error) {
      console.error('Failed to load tasks:', error);
    } finally {
      if (loadRequestIdRef.current === requestId) {
        setLoading(false);
      }
    }
  }, []);

  const createTask = useCallback(async (data: NewTask) => {
    const requestId = ++createRequestIdRef.current;

    try {
      const newTask = await api.createTask(data);
      mutationVersionRef.current += 1;
      if (createRequestIdRef.current === requestId) {
        setTasks((prev) => [...prev, newTask]);
      }

      return newTask;
    } catch (error) {
      console.error('Failed to create task:', error);
      throw error;
    }
  }, []);

  const updateTask = useCallback(async (id: string, data: UpdateTask) => {
    const requestId = ++updateRequestIdRef.current;

    try {
      const updated = await api.updateTask(id, data);
      mutationVersionRef.current += 1;
      if (updateRequestIdRef.current === requestId) {
        setTasks((prev) => {
          const newTasks = prev.map((task) => (task.id === id ? updated : task));
          return newTasks.sort(
            (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
          );
        });
      }

      return updated;
    } catch (error) {
      console.error('Failed to update task:', error);
      throw error;
    }
  }, []);

  const deleteTask = useCallback(async (id: string) => {
    const requestId = ++deleteRequestIdRef.current;

    try {
      await api.deleteTask(id);
      mutationVersionRef.current += 1;
      if (deleteRequestIdRef.current === requestId) {
        setTasks((prev) => prev.filter((task) => task.id !== id));
      }
    } catch (error) {
      console.error('Failed to delete task:', error);
      throw error;
    }
  }, []);

  return { tasks, loading, loadTasks, createTask, updateTask, deleteTask };
}

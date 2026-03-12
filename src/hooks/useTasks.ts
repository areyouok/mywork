import { useState, useCallback } from 'react';
import * as api from '@/api/tasks';
import type { Task } from '@/types/task';

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);

  const loadTasks = useCallback(async () => {
    try {
      setLoading(true);
      const loadedTasks = await api.getTasks();
      setTasks(loadedTasks);
    } catch (error) {
      console.error('Failed to load tasks:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const createTask = useCallback(
    async (data: {
      name: string;
      prompt: string;
      cron_expression?: string;
      simple_schedule?: string;
      once_at?: string;
      enabled?: number;
      timeout_seconds?: number;
    }) => {
      try {
        const newTask = await api.createTask(data);
        setTasks((prev) => [...prev, newTask]);
        await api.reloadScheduler();
        return newTask;
      } catch (error) {
        console.error('Failed to create task:', error);
        throw error;
      }
    },
    []
  );

  const updateTask = useCallback(async (id: string, data: Parameters<typeof api.updateTask>[1]) => {
    try {
      const updated = await api.updateTask(id, data);
      setTasks((prev) => {
        const newTasks = prev.map((task) => (task.id === id ? updated : task));
        return newTasks.sort(
          (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
        );
      });
      return updated;
    } catch (error) {
      console.error('Failed to update task:', error);
      throw error;
    }
  }, []);

  const deleteTask = useCallback(async (id: string) => {
    try {
      await api.deleteTask(id);
      setTasks((prev) => prev.filter((task) => task.id !== id));
    } catch (error) {
      console.error('Failed to delete task:', error);
      throw error;
    }
  }, []);

  return { tasks, loading, loadTasks, createTask, updateTask, deleteTask };
}

import { useState, useEffect, useCallback } from 'react';
import * as api from '@/api/tasks';

export function useScheduler() {
  const [status, setStatus] = useState<string>('unknown');
  const [runningTaskIds, setRunningTaskIds] = useState<Set<string>>(new Set());

  const loadRunningTasks = useCallback(async () => {
    try {
      const taskIds = await api.getRunningExecutions();
      setRunningTaskIds(new Set(taskIds));
    } catch (error) {
      console.error('Failed to load running tasks:', error);
    }
  }, []);

  const initScheduler = useCallback(async () => {
    try {
      const currentStatus = await api.getSchedulerStatus();
      if (currentStatus.startsWith('stopped')) {
        await api.startScheduler();
      }
      setStatus('running');
    } catch (error) {
      console.error('Failed to init scheduler:', error);
      setStatus('error');
    }
  }, []);

  useEffect(() => {
    void loadRunningTasks();
    void initScheduler();
  }, [loadRunningTasks, initScheduler]);

  const addRunningTask = useCallback((taskId: string) => {
    setRunningTaskIds((prev) => {
      const next = new Set(prev);
      next.add(taskId);
      return next;
    });
  }, []);

  const removeRunningTask = useCallback((taskId: string) => {
    setRunningTaskIds((prev) => {
      const next = new Set(prev);
      next.delete(taskId);
      return next;
    });
  }, []);

  const isRunning = useCallback(
    (taskId: string) => {
      return runningTaskIds.has(taskId);
    },
    [runningTaskIds]
  );

  return {
    status,
    runningTaskIds,
    addRunningTask,
    removeRunningTask,
    isRunning,
  };
}

import { useCallback, useRef, type RefObject } from 'react';
import * as api from '@/api/tasks';

export function useTaskActions(
  updateTask: (id: string, data: { enabled?: number }) => Promise<unknown>,
  deleteTask: (id: string) => Promise<void>,
  addRunningTask: (taskId: string) => void,
  removeRunningTask: (taskId: string) => void,
  loadExecutions: (taskId: string | null) => Promise<void>,
  selectedTaskIdRef: RefObject<string | null>,
  loadTasks: () => Promise<void>
) {
  const toggleQueueRef = useRef<Map<string, Promise<void>>>(new Map());

  const handleToggle = useCallback(
    async (taskId: string, enabled: boolean) => {
      const previousToggle = toggleQueueRef.current.get(taskId) ?? Promise.resolve();
      const currentToggle = previousToggle
        .catch(() => undefined)
        .then(async () => {
          try {
            await updateTask(taskId, { enabled: enabled ? 1 : 0 });
            await loadTasks();
          } catch (error) {
            console.error('Failed to toggle task:', error);
          }
        });

      toggleQueueRef.current.set(taskId, currentToggle);

      currentToggle.finally(() => {
        if (toggleQueueRef.current.get(taskId) === currentToggle) {
          toggleQueueRef.current.delete(taskId);
        }
      });

      await currentToggle;
    },
    [updateTask, loadTasks]
  );

  const handleDelete = useCallback(
    async (taskId: string) => {
      try {
        await deleteTask(taskId);
      } catch (error) {
        console.error('Failed to delete task:', error);
      }
    },
    [deleteTask]
  );

  const handleRun = useCallback(
    async (taskId: string) => {
      addRunningTask(taskId);
      try {
        await api.runTask(taskId);
      } catch (error) {
        console.error('Failed to run task:', error);
      } finally {
        removeRunningTask(taskId);
        if (selectedTaskIdRef.current === taskId) {
          try {
            await loadExecutions(taskId);
          } catch (error) {
            console.error('Failed to refresh executions after run:', error);
          }
        }
      }
    },
    [addRunningTask, removeRunningTask, loadExecutions, selectedTaskIdRef]
  );

  return { handleToggle, handleDelete, handleRun };
}

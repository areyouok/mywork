import { useCallback } from 'react';
import * as api from '@/api/tasks';

export function useTaskActions(
  updateTask: (id: string, data: { enabled?: number }) => Promise<unknown>,
  deleteTask: (id: string) => Promise<void>,
  addRunningTask: (taskId: string) => void,
  removeRunningTask: (taskId: string) => void,
  loadExecutions: (taskId: string | null) => Promise<void>,
  selectedTaskIdRef: React.MutableRefObject<string | null>
) {
  const handleToggle = useCallback(
    async (taskId: string, enabled: boolean) => {
      try {
        await updateTask(taskId, { enabled: enabled ? 1 : 0 });
      } catch (error) {
        console.error('Failed to toggle task:', error);
      }
    },
    [updateTask]
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
          loadExecutions(taskId);
        }
      }
    },
    [addRunningTask, removeRunningTask, loadExecutions, selectedTaskIdRef]
  );

  return { handleToggle, handleDelete, handleRun };
}

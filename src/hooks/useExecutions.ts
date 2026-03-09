import { useState, useCallback, useRef } from 'react';
import * as api from '@/api/tasks';
import type { Execution } from '@/types/execution';

export function useExecutions() {
  const [executions, setExecutions] = useState<Execution[]>([]);
  const requestIdRef = useRef(0);

  const loadExecutions = useCallback(async (taskId: string | null) => {
    const requestId = ++requestIdRef.current;

    if (!taskId) {
      setExecutions([]);
      return;
    }

    try {
      const loadedExecutions = await api.getExecutions(taskId);
      if (requestIdRef.current === requestId) {
        setExecutions(loadedExecutions);
      }
    } catch (error) {
      console.error('Failed to load executions:', error);
    }
  }, []);

  return { executions, loadExecutions };
}

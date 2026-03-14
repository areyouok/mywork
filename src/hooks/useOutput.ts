import { useState, useCallback, useRef } from 'react';
import * as api from '@/api/tasks';
import type { Execution } from '@/types/execution';

export function useOutput() {
  const [outputContent, setOutputContent] = useState<string>('');
  const [selectedExecutionId, setSelectedExecutionId] = useState<string | null>(null);
  const requestIdRef = useRef(0);

  const loadOutput = useCallback(async (execution: Execution | string) => {
    const executionId = typeof execution === 'string' ? execution : execution.id;
    const currentRequestId = ++requestIdRef.current;

    try {
      const content = await api.getOutput(executionId);
      if (currentRequestId === requestIdRef.current) {
        setOutputContent(content);
        setSelectedExecutionId(executionId);
      }
    } catch (error) {
      if (currentRequestId === requestIdRef.current) {
        console.error('Failed to load output:', error);
        setOutputContent('');
      }
    }
  }, []);

  return { outputContent, selectedExecutionId, loadOutput, setOutputContent };
}

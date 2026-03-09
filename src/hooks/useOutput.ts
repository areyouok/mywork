import { useState, useCallback } from 'react';
import * as api from '@/api/tasks';
import type { Execution } from '@/types/execution';

export function useOutput() {
  const [outputContent, setOutputContent] = useState<string>('');
  const [selectedExecutionId, setSelectedExecutionId] = useState<string | null>(null);

  const loadOutput = useCallback(async (execution: Execution | string) => {
    try {
      const executionId = typeof execution === 'string' ? execution : execution.id;
      const content = await api.getOutput(executionId);
      setOutputContent(content);
      setSelectedExecutionId(executionId);
    } catch (error) {
      console.error('Failed to load output:', error);
      setOutputContent('');
    }
  }, []);

  return { outputContent, selectedExecutionId, loadOutput, setOutputContent };
}

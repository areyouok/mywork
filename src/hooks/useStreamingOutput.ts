import { useState, useCallback, useRef } from 'react';
import { getOutput } from '@/api/tasks';

export function useStreamingOutput() {
  const [output, setOutput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pollingRef = useRef(false);
  const currentExecutionIdRef = useRef<string | null>(null);
  const lastOutputRef = useRef('');

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const startStreaming = useCallback(
    async (executionId: string) => {
      clearTimer();
      pollingRef.current = false;

      if (currentExecutionIdRef.current !== executionId) {
        setOutput('');
        lastOutputRef.current = '';
      }

      currentExecutionIdRef.current = executionId;
      setError(null);
      setIsStreaming(true);

      const fetchLatest = async () => {
        if (pollingRef.current) {
          return;
        }
        pollingRef.current = true;

        try {
          const content = await getOutput(executionId);

          const nextOutput =
            content.length >= lastOutputRef.current.length ? content : lastOutputRef.current;
          lastOutputRef.current = nextOutput;
          setOutput(nextOutput);
        } catch (pollError) {
          const errorMessage = pollError instanceof Error ? pollError.message : String(pollError);
          setError(errorMessage);
          setIsStreaming(false);
          clearTimer();
        } finally {
          pollingRef.current = false;
        }
      };

      await fetchLatest();
      timerRef.current = setInterval(() => {
        void fetchLatest();
      }, 1000);
    },
    [clearTimer]
  );

  const stopStreaming = useCallback(
    (clearOutput = true) => {
      clearTimer();
      pollingRef.current = false;
      setIsStreaming(false);
      if (clearOutput) {
        setOutput('');
        lastOutputRef.current = '';
        currentExecutionIdRef.current = null;
      }
    },
    [clearTimer]
  );

  const resetOutput = useCallback(() => {
    setOutput('');
    lastOutputRef.current = '';
  }, []);

  return {
    output,
    isStreaming,
    error,
    startStreaming,
    stopStreaming,
    resetOutput,
  };
}

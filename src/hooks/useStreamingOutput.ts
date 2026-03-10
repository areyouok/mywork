import { useState, useCallback, useRef } from 'react';
import { getExecution, getOutput } from '@/api/tasks';

export function useStreamingOutput() {
  const [output, setOutput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const startStreaming = useCallback(
    async (executionId: string) => {
      clearTimer();
      setError(null);
      setIsStreaming(true);

      try {
        const initial = await getOutput(executionId);
        setOutput(initial);

        timerRef.current = setInterval(async () => {
          try {
            const [content, execution] = await Promise.all([
              getOutput(executionId),
              getExecution(executionId),
            ]);
            setOutput(content);

            if (execution.status !== 'running') {
              setIsStreaming(false);
              clearTimer();
            }
          } catch (pollError) {
            const errorMessage = pollError instanceof Error ? pollError.message : String(pollError);
            setError(errorMessage);
            setIsStreaming(false);
            clearTimer();
          }
        }, 1000);
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        setError(errorMessage);
        setIsStreaming(false);
        clearTimer();
      }
    },
    [clearTimer]
  );

  const stopStreaming = useCallback(() => {
    clearTimer();
    setIsStreaming(false);
    setOutput('');
  }, [clearTimer]);

  const resetOutput = useCallback(() => {
    setOutput('');
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

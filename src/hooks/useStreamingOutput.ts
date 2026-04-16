import { useState, useCallback, useRef } from 'react';
import { getOutput } from '@/api/tasks';
import { parseJsonlEvents } from '@/types/event';
import type { OpenCodeEvent } from '@/types/event';

export function useStreamingOutput() {
  const [output, setOutput] = useState('');
  const [events, setEvents] = useState<OpenCodeEvent[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pollingRef = useRef(false);
  const currentExecutionIdRef = useRef<string | null>(null);
  const lastOutputRef = useRef('');
  const parsedIndexRef = useRef(0);

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
        setEvents([]);
        lastOutputRef.current = '';
        parsedIndexRef.current = 0;
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

          if (nextOutput !== lastOutputRef.current) {
            lastOutputRef.current = nextOutput;
            setOutput(nextOutput);

            const newContent = nextOutput.slice(parsedIndexRef.current);
            const lastNewlineIdx = newContent.lastIndexOf('\n');
            if (lastNewlineIdx >= 0) {
              const toParse = newContent.slice(0, lastNewlineIdx);
              parsedIndexRef.current += lastNewlineIdx + 1;

              const newEvents = parseJsonlEvents(toParse);
              if (newEvents.length > 0) {
                setEvents((prev) => [...prev, ...newEvents]);
              }
            }
          }
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

      if (!clearOutput && lastOutputRef.current.length > parsedIndexRef.current) {
        const remaining = lastOutputRef.current.slice(parsedIndexRef.current);
        const trimmed = remaining.trim();
        if (trimmed) {
          const newEvents = parseJsonlEvents(remaining);
          if (newEvents.length > 0) {
            setEvents((prev) => [...prev, ...newEvents]);
          }
        }
        parsedIndexRef.current = lastOutputRef.current.length;
      }

      if (clearOutput) {
        setOutput('');
        setEvents([]);
        lastOutputRef.current = '';
        parsedIndexRef.current = 0;
        currentExecutionIdRef.current = null;
      }
    },
    [clearTimer]
  );

  const resetOutput = useCallback(() => {
    setOutput('');
    setEvents([]);
    lastOutputRef.current = '';
    parsedIndexRef.current = 0;
  }, []);

  return {
    output,
    events,
    isStreaming,
    error,
    startStreaming,
    stopStreaming,
    resetOutput,
  };
}

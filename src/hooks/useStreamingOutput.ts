import { useState, useCallback } from 'react';
import { invoke, Channel } from '@tauri-apps/api/core';

type OutputEvent =
  | { stdout: { text: string } }
  | { stderr: { text: string } }
  | { finished: { exitCode: number } };

export function useStreamingOutput() {
  const [output, setOutput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const startStreaming = useCallback(async (taskId: string, prompt: string, cwd?: string) => {
    setError(null);
    setIsStreaming(true);
    setOutput('');

    const channel = new Channel<OutputEvent>();

    channel.onmessage = (msg: OutputEvent) => {
      if ('stdout' in msg) {
        setOutput((prev) => prev + msg.stdout.text + '\n');
      } else if ('stderr' in msg) {
        setOutput((prev) => prev + msg.stderr.text + '\n');
      } else if ('finished' in msg) {
        setIsStreaming(false);
      }
    };

    try {
      await invoke('execute_task_streaming', {
        taskId,
        prompt,
        cwd: cwd ?? null,
        channel,
      });
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
      setIsStreaming(false);
    }
  }, []);

  const stopStreaming = useCallback(() => {
    setIsStreaming(false);
    setOutput('');
  }, []);

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

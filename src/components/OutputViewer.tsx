import { useEffect, useRef } from 'react';
import { AnsiRenderer } from './AnsiRenderer';
import { useOutput } from '../hooks/useOutput';
import { useStreamingOutput } from '../hooks/useStreamingOutput';
import type { Execution } from '@/types/execution';
import './OutputViewer.css';

interface OutputViewerProps {
  content?: string;
  isMarkdown?: boolean;
  execution?: Execution | null;
}

export function OutputViewer({ content, isMarkdown: _isMarkdown, execution }: OutputViewerProps) {
  const isRunning = execution?.status === 'running';
  const { outputContent, loadOutput } = useOutput();
  const {
    output: streamingOutput,
    isStreaming,
    startStreaming,
    stopStreaming,
  } = useStreamingOutput();
  const containerRef = useRef<HTMLDivElement>(null);
  const useContentPropDirectly = content !== undefined && !isRunning;

  useEffect(() => {
    if (!useContentPropDirectly && !isRunning && execution) {
      loadOutput(execution);
    }
  }, [execution, isRunning, loadOutput, useContentPropDirectly]);

  useEffect(() => {
    if (!useContentPropDirectly && isRunning && execution) {
      startStreaming(execution.id);
    }
    return () => {
      stopStreaming();
    };
  }, [execution, isRunning, startStreaming, stopStreaming, useContentPropDirectly]);

  useEffect(() => {
    if (isStreaming && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [isStreaming, streamingOutput]);

  const displayContent = useContentPropDirectly
    ? content
    : isRunning
      ? streamingOutput
      : outputContent;
  const isEmpty = !displayContent || displayContent.trim() === '';

  if (isEmpty) {
    return (
      <div className="output-viewer output-viewer-empty">
        <p>No output</p>
      </div>
    );
  }

  return (
    <div className="output-viewer">
      <div className="output-viewer-content" ref={containerRef}>
        <AnsiRenderer text={displayContent} />
      </div>
    </div>
  );
}

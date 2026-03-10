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
  const { output: streamingOutput, startStreaming, stopStreaming } = useStreamingOutput();
  const containerRef = useRef<HTMLDivElement>(null);
  const shouldAutoScrollRef = useRef(true);
  const useContentPropDirectly = content !== undefined && !isRunning;

  const handleScroll = () => {
    if (!containerRef.current) {
      return;
    }
    const el = containerRef.current;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    shouldAutoScrollRef.current = distanceFromBottom < 24;
  };

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

  const displayContent = useContentPropDirectly
    ? content
    : isRunning
      ? streamingOutput
      : outputContent;

  useEffect(() => {
    if (containerRef.current && shouldAutoScrollRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [displayContent]);
  const isEmpty = !displayContent || displayContent.trim() === '';
  const statusLabel = execution ? execution.status : null;

  if (isEmpty) {
    return (
      <div className="output-viewer output-viewer-empty">
        <p>No output</p>
      </div>
    );
  }

  return (
    <div className="output-viewer">
      {statusLabel && (
        <div className="output-status-row">
          <span className={`execution-status status-${statusLabel}`}>{statusLabel}</span>
        </div>
      )}
      <div className="output-viewer-content" ref={containerRef} onScroll={handleScroll}>
        <AnsiRenderer text={displayContent} />
      </div>
    </div>
  );
}

import { useEffect, useRef } from 'react';
import { AnsiRenderer } from './AnsiRenderer';
import { useStreamingOutput } from '../hooks/useStreamingOutput';
import type { Execution } from '@/types/execution';
import './OutputViewer.css';

interface OutputViewerProps {
  content?: string;
  isMarkdown?: boolean;
  execution?: Execution | null;
}

export function OutputViewer({ content, isMarkdown: _isMarkdown, execution }: OutputViewerProps) {
  const { output: streamingOutput, startStreaming, stopStreaming } = useStreamingOutput();
  const containerRef = useRef<HTMLDivElement>(null);
  const shouldAutoScrollRef = useRef(true);
  const executionId = execution?.id;

  const handleScroll = () => {
    if (!containerRef.current) {
      return;
    }
    const el = containerRef.current;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    shouldAutoScrollRef.current = distanceFromBottom < 24;
  };

  useEffect(() => {
    if (executionId) {
      startStreaming(executionId);
    }

    return () => {
      stopStreaming();
    };
  }, [executionId, startStreaming, stopStreaming]);

  const displayContent = execution ? streamingOutput || content || '' : content || '';

  useEffect(() => {
    if (containerRef.current && shouldAutoScrollRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [displayContent]);
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
      <div className="output-viewer-content" ref={containerRef} onScroll={handleScroll}>
        <AnsiRenderer text={displayContent} />
      </div>
    </div>
  );
}

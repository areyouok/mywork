import { useEffect, useRef, useMemo } from 'react';
import { EventRenderer } from './EventRenderer';
import { useStreamingOutput } from '../hooks/useStreamingOutput';
import { parseJsonlEvents, sortEventsByPartId } from '../types/event';
import type { Execution } from '@/types/execution';
import './OutputViewer.css';

interface OutputViewerProps {
  content?: string;
  execution?: Execution | null;
}

export function OutputViewer({ content, execution }: OutputViewerProps) {
  const {
    output: streamingOutput,
    events: streamingEvents,
    startStreaming,
    stopStreaming,
  } = useStreamingOutput();
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

  const parsedEvents = useMemo(() => {
    if (streamingEvents.length > 0) return [];
    return sortEventsByPartId(parseJsonlEvents(displayContent));
  }, [displayContent, streamingEvents.length]);

  const events = streamingEvents.length > 0 ? streamingEvents : parsedEvents;

  useEffect(() => {
    if (containerRef.current && shouldAutoScrollRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [displayContent, events]);

  const fallbackExecutionMessage =
    execution?.status !== 'running' && execution?.error_message ? execution.error_message : '';
  const isEmpty = events.length === 0 && !fallbackExecutionMessage;

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
        {events.length > 0 ? (
          <EventRenderer events={events} />
        ) : (
          <div className="output-viewer-fallback-error">{fallbackExecutionMessage}</div>
        )}
      </div>
    </div>
  );
}

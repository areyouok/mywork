import { useState } from 'react';
import type { ToolUsePart } from '@/types/event';
import './ToolUseCard.css';

interface ToolUseCardProps {
  part: ToolUsePart;
}

function formatDuration(start?: number, end?: number): string | null {
  if (start == null || end == null) return null;
  const diff = end - start;
  if (diff >= 1000) {
    return `${(diff / 1000).toFixed(1)}s`;
  }
  return `${diff}ms`;
}

function formatInput(input: Record<string, unknown>): string {
  return Object.entries(input)
    .map(([key, value]) => {
      const formatted =
        typeof value === 'object' && value !== null
          ? JSON.stringify(value, null, 2)
          : String(value);
      return `${key}: ${formatted}`;
    })
    .join('\n');
}

export function ToolUseCard({ part }: ToolUseCardProps) {
  const [inputExpanded, setInputExpanded] = useState(false);
  const [outputExpanded, setOutputExpanded] = useState(true);
  const { state } = part;
  const hasOutput = !!state.output;
  const duration = formatDuration(state.time?.start, state.time?.end);
  const exitCode = state.metadata?.exit;
  const showExitCode = exitCode != null && exitCode !== 0;

  return (
    <div className="tool-use-card" data-call-id={part.callID}>
      <div className="tool-use-card-header">
        <span className={`tool-status-indicator status-${state.status}`} />
        <span className="tool-name">{part.tool}</span>
        {state.title && (
          <>
            <span className="tool-separator">&middot;</span>
            <span className="tool-title">{state.title}</span>
          </>
        )}
        <div className="tool-header-meta">
          {duration && <span className="tool-duration">{duration}</span>}
          {showExitCode && <span className="tool-exit-code">exit: {exitCode}</span>}
        </div>
      </div>

      <div className="tool-section">
        <div
          className="tool-section-header tool-input-toggle"
          onClick={() => setInputExpanded(!inputExpanded)}
          role="button"
          tabIndex={0}
        >
          <span className="tool-section-toggle">{inputExpanded ? '\u25BC' : '\u25B6'}</span>
          <span className="tool-section-label">Input</span>
        </div>
        <div className={`tool-input-content ${inputExpanded ? '' : 'collapsed'}`}>
          {formatInput(state.input)}
        </div>
      </div>

      {hasOutput && (
        <div className="tool-section">
          <div
            className="tool-section-header tool-output-toggle"
            onClick={() => setOutputExpanded(!outputExpanded)}
            role="button"
            tabIndex={0}
          >
            <span className="tool-section-toggle">{outputExpanded ? '\u25BC' : '\u25B6'}</span>
            <span className="tool-section-label">Output</span>
          </div>
          <div className={`tool-output-content ${outputExpanded ? '' : 'collapsed'}`}>
            {state.output}
          </div>
        </div>
      )}
    </div>
  );
}

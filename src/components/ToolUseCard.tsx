import { useState } from 'react';
import type { ToolUsePart } from '@/types/event';
import { TodoList } from './TodoList';
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

const TRUNCATE_HEAD = 50;
const TRUNCATE_TAIL = 50;

interface TruncateResult {
  head: string;
  tail: string;
  omittedCount: number;
  truncated: boolean;
}

function truncateOutput(output: string): TruncateResult {
  let lines = output.split(/\r?\n/);
  // Trailing newline produces an empty element; remove it for accurate line counting
  if (lines.length > 0 && lines[lines.length - 1] === '') {
    lines = lines.slice(0, -1);
  }
  if (lines.length <= TRUNCATE_HEAD + TRUNCATE_TAIL) {
    return { head: output, tail: '', omittedCount: 0, truncated: false };
  }
  const head = lines.slice(0, TRUNCATE_HEAD).join('\n');
  const tail = lines.slice(-TRUNCATE_TAIL).join('\n');
  const omittedCount = lines.length - TRUNCATE_HEAD - TRUNCATE_TAIL;
  return { head, tail, omittedCount, truncated: true };
}

function decodeHtmlEntities(text: string): string {
  return text
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
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
  const truncated = state.output ? truncateOutput(state.output) : null;

  return (
    <div className="tool-use-card" data-call-id={part.callID}>
      <div className="tool-use-card-header">
        <span className={`tool-status-indicator status-${state.status}`} />
        <span className="tool-name">{part.tool}</span>
        {state.title && (
          <>
            <span className="tool-separator">&middot;</span>
            <span className="tool-title">{decodeHtmlEntities(state.title)}</span>
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
            {part.tool === 'todowrite' && state.output ? (
              <TodoList output={state.output} />
            ) : truncated?.truncated ? (
              <>
                <pre className="tool-output-head">{truncated.head}</pre>
                <div className="truncation-notice">
                  ... {truncated.omittedCount} line
                  {truncated.omittedCount !== 1 ? 's' : ''} omitted ...
                </div>
                <pre className="tool-output-tail">{truncated.tail}</pre>
              </>
            ) : (
              state.output
            )}
          </div>
        </div>
      )}
    </div>
  );
}

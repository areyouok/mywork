import type { OpenCodeEvent, OpenCodeStepFinishEvent } from '@/types/event';
import { TextBlock } from './TextBlock';
import { ToolUseCard } from './ToolUseCard';
import './EventRenderer.css';

interface EventRendererProps {
  events: OpenCodeEvent[];
}

function formatTokens(tokens: OpenCodeStepFinishEvent['part']['tokens']): string | null {
  if (!tokens) return null;
  return `${tokens.input.toLocaleString()} in / ${tokens.output.toLocaleString()} out`;
}

function formatCost(cost: number | undefined): string | null {
  if (cost === undefined) return null;
  return `$${cost.toFixed(2)}`;
}

function StepFinishBar({ part }: { part: OpenCodeStepFinishEvent['part'] }) {
  const tokensText = formatTokens(part.tokens);
  const costText = formatCost(part.cost);
  return (
    <div className="step-stats-bar">
      <span className="step-stats-reason">{part.reason}</span>
      {tokensText && <span className="step-stats-tokens">{tokensText}</span>}
      {costText && <span className="step-stats-cost">{costText}</span>}
    </div>
  );
}

export function EventRenderer({ events }: EventRendererProps) {
  return (
    <div className="event-renderer">
      {events.map((event) => {
        const key = `${event.type}-${event.part.id}`;
        switch (event.type) {
          case 'text':
            return (
              <div key={key} data-event-key={key}>
                <TextBlock part={event.part} />
              </div>
            );
          case 'tool_use':
            return (
              <div key={key} data-event-key={key}>
                <ToolUseCard part={event.part} />
              </div>
            );
          case 'step_start':
            return null;
          case 'step_finish':
            return (
              <div key={key} data-event-key={key}>
                <StepFinishBar part={event.part} />
              </div>
            );
        }
      })}
    </div>
  );
}

import type { OpenCodeEvent } from '@/types/event';
import { TextBlock } from './TextBlock';
import { ToolUseCard } from './ToolUseCard';
import './EventRenderer.css';

interface EventRendererProps {
  events: OpenCodeEvent[];
}

export function EventRenderer({ events }: EventRendererProps) {
  return (
    <div className="event-renderer">
      {events.map((event, index) => {
        const key = `${event.type}-${event.timestamp}-${index}`;
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
          case 'step_finish':
            return null;
        }
      })}
    </div>
  );
}

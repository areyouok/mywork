import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { EventRenderer } from './EventRenderer';
import type { OpenCodeEvent } from '@/types/event';

const textEvent: OpenCodeEvent = {
  type: 'text',
  timestamp: 1000,
  sessionID: 'ses_1',
  part: {
    type: 'text',
    id: 'p1',
    messageID: 'm1',
    sessionID: 'ses_1',
    text: 'Hello from event',
  },
};

const toolEvent: OpenCodeEvent = {
  type: 'tool_use',
  timestamp: 2000,
  sessionID: 'ses_1',
  part: {
    type: 'tool',
    tool: 'bash',
    callID: 'call_1',
    state: {
      status: 'completed',
      input: { command: 'echo hi' },
      output: 'hi\n',
      metadata: { exit: 0 },
      title: 'Echo hi',
      time: { start: 2000, end: 2003 },
    },
    id: 'p2',
    sessionID: 'ses_1',
    messageID: 'm1',
  },
};

const stepStartEvent: OpenCodeEvent = {
  type: 'step_start',
  timestamp: 500,
  sessionID: 'ses_1',
  part: {
    id: 'p0',
    messageID: 'm1',
    sessionID: 'ses_1',
    snapshot: 'snap1',
    type: 'step-start',
  },
};

const stepFinishEvent: OpenCodeEvent = {
  type: 'step_finish',
  timestamp: 3000,
  sessionID: 'ses_1',
  part: {
    id: 'p3',
    reason: 'stop',
    snapshot: 'snap2',
    messageID: 'm1',
    sessionID: 'ses_1',
    type: 'step-finish',
    tokens: { total: 100, input: 10, output: 90, reasoning: 0, cache: { write: 0, read: 0 } },
    cost: 0,
  },
};

describe('EventRenderer', () => {
  it('should render mixed event types', () => {
    const events: OpenCodeEvent[] = [stepStartEvent, textEvent, toolEvent, stepFinishEvent];
    render(<EventRenderer events={events} />);

    expect(screen.getByText('Hello from event')).toBeInTheDocument();
    expect(screen.getByText('Echo hi')).toBeInTheDocument();
  });

  it('should render empty events array without errors', () => {
    const { container } = render(<EventRenderer events={[]} />);
    expect(container.querySelector('.event-renderer')).toBeInTheDocument();
    expect(container.querySelector('.event-renderer')?.children.length).toBe(0);
  });

  it('should not render step_start events', () => {
    const { container } = render(<EventRenderer events={[stepStartEvent]} />);
    expect(container.querySelector('.event-renderer')?.children.length).toBe(0);
  });

  it('should render step_finish events with statistics bar', () => {
    const { container } = render(<EventRenderer events={[stepFinishEvent]} />);
    expect(container.querySelector('.step-stats-bar')).toBeInTheDocument();
  });

  it('should display reason in step_finish bar', () => {
    render(<EventRenderer events={[stepFinishEvent]} />);
    expect(screen.getByText('stop')).toBeInTheDocument();
  });

  it('should display formatted tokens in step_finish bar', () => {
    const { container } = render(<EventRenderer events={[stepFinishEvent]} />);
    expect(container.querySelector('.step-stats-tokens')).toHaveTextContent('10 in / 90 out');
  });

  it('should display formatted cost in step_finish bar', () => {
    const { container } = render(<EventRenderer events={[stepFinishEvent]} />);
    expect(container.querySelector('.step-stats-cost')).toHaveTextContent('$0.00');
  });

  it('should display only reason when tokens and cost are undefined', () => {
    const noDataEvent: OpenCodeEvent = {
      type: 'step_finish',
      timestamp: 3000,
      sessionID: 'ses_1',
      part: {
        id: 'p3',
        reason: 'tool-calls',
        snapshot: 'snap2',
        messageID: 'm1',
        sessionID: 'ses_1',
        type: 'step-finish',
      },
    };
    const { container } = render(<EventRenderer events={[noDataEvent]} />);
    expect(screen.getByText('tool-calls')).toBeInTheDocument();
    expect(container.querySelector('.step-stats-tokens')).not.toBeInTheDocument();
    expect(container.querySelector('.step-stats-cost')).not.toBeInTheDocument();
  });

  it('should render text events using TextBlock', () => {
    render(<EventRenderer events={[textEvent]} />);
    expect(screen.getByText('Hello from event')).toBeInTheDocument();
  });

  it('should render tool_use events using ToolUseCard', () => {
    render(<EventRenderer events={[toolEvent]} />);
    expect(screen.getByText('Echo hi')).toBeInTheDocument();
  });

  it('should use stable keys based on part.id', () => {
    const events: OpenCodeEvent[] = [textEvent, toolEvent];
    const { container } = render(<EventRenderer events={events} />);
    const children = container.querySelector('.event-renderer')?.children;

    expect(children?.[0].getAttribute('data-event-key')).toBe('text-p1');
    expect(children?.[1].getAttribute('data-event-key')).toBe('tool_use-p2');
  });
});

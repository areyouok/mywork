import { describe, it, expect } from 'vitest';
import { parseJsonlEvents, sortEventsByTimestamp } from './event';
import type { OpenCodeEvent } from './event';

describe('parseJsonlEvents', () => {
  it('should parse valid JSONL text into events', () => {
    const jsonl = [
      JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      }),
      JSON.stringify({
        type: 'tool_use',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: {
          type: 'tool',
          tool: 'bash',
          callID: 'call_1',
          state: { status: 'completed', input: { command: 'ls' } },
          id: 'p2',
          sessionID: 'ses_1',
          messageID: 'm1',
        },
      }),
    ].join('\n');

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(2);
    expect(events[0].type).toBe('text');
    expect(events[1].type).toBe('tool_use');
  });

  it('should skip non-JSON lines', () => {
    const jsonl = [
      'this is not json',
      JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      }),
      'also not json',
    ].join('\n');

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe('text');
  });

  it('should skip empty lines', () => {
    const jsonl = [
      '',
      JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      }),
      '',
      '',
    ].join('\n');

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(1);
  });

  it('should handle mixed valid and invalid lines', () => {
    const jsonl = [
      JSON.stringify({
        type: 'step_start',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { id: 'p1', messageID: 'm1', sessionID: 'ses_1', snapshot: 's1', type: 'step-start' },
      }),
      'some random log line',
      '',
      JSON.stringify({
        type: 'text',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p2', messageID: 'm1', sessionID: 'ses_1', text: 'result' },
      }),
      '{ incomplete json',
    ].join('\n');

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(2);
    expect(events[0].type).toBe('step_start');
    expect(events[1].type).toBe('text');
  });

  it('should return empty array for empty string', () => {
    expect(parseJsonlEvents('')).toEqual([]);
  });

  it('should return empty array for whitespace-only string', () => {
    expect(parseJsonlEvents('   \n  \n  ')).toEqual([]);
  });

  it('should parse single event', () => {
    const json = JSON.stringify({
      type: 'text',
      timestamp: 1000,
      sessionID: 'ses_1',
      part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'single' },
    });

    const events = parseJsonlEvents(json);
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe('text');
    const textEvent = events[0] as OpenCodeEvent & { type: 'text' };
    expect(textEvent.part.text).toBe('single');
  });

  it('should parse step_finish event with tokens and cost', () => {
    const jsonl = JSON.stringify({
      type: 'step_finish',
      timestamp: 3000,
      sessionID: 'ses_1',
      part: {
        id: 'p1',
        reason: 'stop',
        snapshot: 's1',
        messageID: 'm1',
        sessionID: 'ses_1',
        type: 'step-finish',
        tokens: {
          total: 18548,
          input: 49,
          output: 3,
          reasoning: 0,
          cache: { write: 0, read: 18496 },
        },
        cost: 0,
      },
    });

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(1);
    const finishEvent = events[0] as OpenCodeEvent & { type: 'step_finish' };
    expect(finishEvent.part.tokens?.total).toBe(18548);
    expect(finishEvent.part.cost).toBe(0);
  });

  it('should parse tool_use event with full state', () => {
    const jsonl = JSON.stringify({
      type: 'tool_use',
      timestamp: 2000,
      sessionID: 'ses_1',
      part: {
        type: 'tool',
        tool: 'bash',
        callID: 'call_xxx',
        state: {
          status: 'completed',
          input: { command: 'echo hello', description: 'Echo hello' },
          output: 'hello\n',
          metadata: { output: 'hello\n', exit: 0, description: 'Echo hello', truncated: false },
          title: 'Echo hello',
          time: { start: 1776342614905, end: 1776342614908 },
        },
        id: 'p1',
        sessionID: 'ses_1',
        messageID: 'm1',
      },
    });

    const events = parseJsonlEvents(jsonl);
    expect(events).toHaveLength(1);
    const toolEvent = events[0] as OpenCodeEvent & { type: 'tool_use' };
    expect(toolEvent.part.tool).toBe('bash');
    expect(toolEvent.part.state.status).toBe('completed');
    expect(toolEvent.part.state.output).toBe('hello\n');
    expect(toolEvent.part.state.metadata?.exit).toBe(0);
  });
});

describe('sortEventsByTimestamp', () => {
  it('should sort mixed event types by timestamp', () => {
    const events: OpenCodeEvent[] = [
      {
        type: 'step_start',
        timestamp: 100,
        sessionID: 'ses_1',
        part: {
          id: 'prt_zzz',
          messageID: 'm1',
          sessionID: 'ses_1',
          snapshot: 's1',
          type: 'step-start',
        },
      },
      {
        type: 'tool_use',
        timestamp: 200,
        sessionID: 'ses_1',
        part: {
          type: 'tool',
          tool: 'bash',
          callID: 'call_1',
          state: { status: 'completed', input: {} },
          id: 'prt_zzz',
          sessionID: 'ses_1',
          messageID: 'm1',
        },
      },
      {
        type: 'text',
        timestamp: 201,
        sessionID: 'ses_1',
        part: {
          type: 'text',
          id: 'prt_aaa',
          messageID: 'm1',
          sessionID: 'ses_1',
          text: 'result',
        },
      },
      {
        type: 'step_finish',
        timestamp: 300,
        sessionID: 'ses_1',
        part: {
          id: 'prt_zzz',
          reason: 'stop',
          snapshot: 's1',
          messageID: 'm1',
          sessionID: 'ses_1',
          type: 'step-finish',
        },
      },
    ];

    const sorted = sortEventsByTimestamp(events);
    expect(sorted.map((e) => e.type)).toEqual(['step_start', 'tool_use', 'text', 'step_finish']);
  });

  it('should sort by timestamp when part.id order is reversed', () => {
    const events: OpenCodeEvent[] = [
      {
        type: 'text',
        timestamp: 201,
        sessionID: 'ses_1',
        part: {
          type: 'text',
          id: 'prt_aaa',
          messageID: 'm1',
          sessionID: 'ses_1',
          text: 'result',
        },
      },
      {
        type: 'tool_use',
        timestamp: 200,
        sessionID: 'ses_1',
        part: {
          type: 'tool',
          tool: 'bash',
          callID: 'call_1',
          state: { status: 'completed', input: {} },
          id: 'prt_zzz',
          sessionID: 'ses_1',
          messageID: 'm1',
        },
      },
    ];

    const sorted = sortEventsByTimestamp(events);
    expect(sorted[0].type).toBe('tool_use');
    expect(sorted[1].type).toBe('text');
  });

  it('should fallback to part.id when timestamps are equal', () => {
    const events: OpenCodeEvent[] = [
      {
        type: 'tool_use',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: {
          type: 'tool',
          tool: 'todowrite',
          callID: 'call_1',
          state: { status: 'completed', input: {} },
          id: 'prt_b',
          sessionID: 'ses_1',
          messageID: 'm1',
        },
      },
      {
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_a', messageID: 'm1', sessionID: 'ses_1', text: 'first' },
      },
    ];

    const sorted = sortEventsByTimestamp(events);
    const types = sorted.map((e) => e.type);
    expect(types).toEqual(['text', 'tool_use']);
  });

  it('should return empty array for empty input', () => {
    expect(sortEventsByTimestamp([])).toEqual([]);
  });

  it('should not mutate the original array', () => {
    const events: OpenCodeEvent[] = [
      {
        type: 'text',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_b', messageID: 'm1', sessionID: 'ses_1', text: 'b' },
      },
      {
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_a', messageID: 'm1', sessionID: 'ses_1', text: 'a' },
      },
    ];

    const sorted = sortEventsByTimestamp(events);
    expect(sorted).not.toBe(events);
    const originalFirst = (events[0] as OpenCodeEvent & { type: 'text' }).part.text;
    expect(originalFirst).toBe('b');
  });

  it('should sort same-type events by timestamp', () => {
    const events: OpenCodeEvent[] = [
      {
        type: 'text',
        timestamp: 3000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_c', messageID: 'm1', sessionID: 'ses_1', text: 'c' },
      },
      {
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_a', messageID: 'm1', sessionID: 'ses_1', text: 'a' },
      },
      {
        type: 'text',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'prt_b', messageID: 'm1', sessionID: 'ses_1', text: 'b' },
      },
    ];

    const sorted = sortEventsByTimestamp(events);
    expect(sorted.map((e) => (e as OpenCodeEvent & { type: 'text' }).part.text)).toEqual([
      'a',
      'b',
      'c',
    ]);
  });
});

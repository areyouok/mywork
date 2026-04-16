export interface TimeInfo {
  start: number;
  end: number;
}

export interface TokenInfo {
  total: number;
  input: number;
  output: number;
  reasoning: number;
  cache: {
    write: number;
    read: number;
  };
}

export interface TextPart {
  type: 'text';
  id: string;
  messageID: string;
  sessionID: string;
  text: string;
  time?: TimeInfo;
}

export interface ToolMetadata {
  exit?: number;
  description?: string;
  truncated?: boolean;
  output?: string;
}

export interface ToolState {
  status: string;
  input: Record<string, unknown>;
  output?: string;
  metadata?: ToolMetadata;
  title?: string;
  time?: TimeInfo;
}

export interface ToolUsePart {
  type: 'tool';
  tool: string;
  callID: string;
  title?: string;
  state: ToolState;
  id: string;
  sessionID: string;
  messageID: string;
}

export interface StepStartPart {
  id: string;
  messageID: string;
  sessionID: string;
  snapshot: string;
  type: 'step-start';
}

export interface StepFinishPart {
  id: string;
  reason: string;
  snapshot: string;
  messageID: string;
  sessionID: string;
  type: 'step-finish';
  tokens?: TokenInfo;
  cost?: number;
}

export interface OpenCodeTextEvent {
  type: 'text';
  timestamp: number;
  sessionID: string;
  part: TextPart;
}

export interface OpenCodeToolUseEvent {
  type: 'tool_use';
  timestamp: number;
  sessionID: string;
  part: ToolUsePart;
}

export interface OpenCodeStepStartEvent {
  type: 'step_start';
  timestamp: number;
  sessionID: string;
  part: StepStartPart;
}

export interface OpenCodeStepFinishEvent {
  type: 'step_finish';
  timestamp: number;
  sessionID: string;
  part: StepFinishPart;
}

export type OpenCodeEvent =
  | OpenCodeTextEvent
  | OpenCodeToolUseEvent
  | OpenCodeStepStartEvent
  | OpenCodeStepFinishEvent;

export function parseJsonlEvents(text: string): OpenCodeEvent[] {
  const events: OpenCodeEvent[] = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const event = JSON.parse(trimmed) as OpenCodeEvent;
      events.push(event);
    } catch {
      // skip non-JSON lines
    }
  }
  return events;
}

export function sortEventsByPartId(events: OpenCodeEvent[]): OpenCodeEvent[] {
  return [...events].sort((a, b) => {
    const idA = a.part?.id;
    const idB = b.part?.id;
    if (!idA && !idB) return 0;
    if (!idA) return 1;
    if (!idB) return -1;
    const idCompare = idA.localeCompare(idB);
    if (idCompare !== 0) return idCompare;
    return a.timestamp - b.timestamp;
  });
}

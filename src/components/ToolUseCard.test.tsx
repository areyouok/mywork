import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ToolUseCard } from './ToolUseCard';
import type { ToolUsePart } from '@/types/event';

function makeToolPart(overrides: Partial<ToolUsePart> = {}): ToolUsePart {
  return {
    type: 'tool',
    tool: 'bash',
    callID: 'call_1',
    state: {
      status: 'completed',
      input: { command: 'echo hello' },
      output: 'hello\n',
      metadata: { exit: 0 },
      title: 'Echo hello',
      time: { start: 1000, end: 1005 },
    },
    id: 'p1',
    sessionID: 'ses_1',
    messageID: 'm1',
    ...overrides,
  };
}

describe('ToolUseCard', () => {
  it('should render tool name and title', () => {
    render(<ToolUseCard part={makeToolPart()} />);
    expect(screen.getByText('bash')).toBeInTheDocument();
    expect(screen.getByText('Echo hello', { selector: '.tool-title' })).toBeInTheDocument();
  });

  it('should render with tool-use-card class', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    expect(container.querySelector('.tool-use-card')).toBeInTheDocument();
  });

  it('should show green indicator for completed status', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    const indicator = container.querySelector('.tool-status-indicator');
    expect(indicator).toHaveClass('status-completed');
  });

  it('should show red indicator for failed status', () => {
    const part = makeToolPart({
      state: {
        status: 'failed',
        input: { command: 'bad command' },
        metadata: { exit: 1 },
        title: 'Bad command',
        time: { start: 1000, end: 1005 },
      },
    });
    const { container } = render(<ToolUseCard part={part} />);
    const indicator = container.querySelector('.tool-status-indicator');
    expect(indicator).toHaveClass('status-failed');
  });

  it('should show duration in ms when under 1 second', () => {
    render(<ToolUseCard part={makeToolPart()} />);
    expect(screen.getByText('5ms')).toBeInTheDocument();
  });

  it('should show duration in seconds when over 1 second', () => {
    const part = makeToolPart({
      state: {
        status: 'completed',
        input: { command: 'sleep 2' },
        output: '',
        title: 'Sleep',
        time: { start: 1000, end: 3500 },
      },
    });
    render(<ToolUseCard part={part} />);
    expect(screen.getByText('2.5s')).toBeInTheDocument();
  });

  it('should not show duration when time is missing', () => {
    const part = makeToolPart({
      state: {
        status: 'completed',
        input: { command: 'test' },
        output: '',
      },
    });
    const { container } = render(<ToolUseCard part={part} />);
    expect(container.querySelector('.tool-duration')).not.toBeInTheDocument();
  });

  it('should show output area expanded by default when output exists', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    const outputArea = container.querySelector('.tool-output-content');
    expect(outputArea).toBeVisible();
  });

  it('should toggle output visibility on click', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    const toggleBtn = container.querySelector('.tool-output-toggle');

    expect(container.querySelector('.tool-output-content')).toBeVisible();

    fireEvent.click(toggleBtn!);
    expect(container.querySelector('.tool-output-content.collapsed')).toBeInTheDocument();

    fireEvent.click(toggleBtn!);
    expect(container.querySelector('.tool-output-content')).toBeVisible();
  });

  it('should toggle input visibility on click', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    const toggleBtn = container.querySelector('.tool-input-toggle');

    expect(container.querySelector('.tool-input-content.collapsed')).toBeInTheDocument();

    fireEvent.click(toggleBtn!);
    expect(container.querySelector('.tool-input-content')).toBeVisible();

    fireEvent.click(toggleBtn!);
    expect(container.querySelector('.tool-input-content.collapsed')).toBeInTheDocument();
  });

  it('should show exit code when non-zero', () => {
    const part = makeToolPart({
      state: {
        status: 'failed',
        input: { command: 'fail' },
        output: 'error',
        metadata: { exit: 127 },
        title: 'Fail',
        time: { start: 1000, end: 1005 },
      },
    });
    render(<ToolUseCard part={part} />);
    expect(screen.getByText(/exit: 127/i)).toBeInTheDocument();
  });

  it('should not show exit code when zero', () => {
    render(<ToolUseCard part={makeToolPart()} />);
    expect(screen.queryByText(/exit:/i)).not.toBeInTheDocument();
  });

  it('should handle empty output', () => {
    const part = makeToolPart({
      state: {
        status: 'completed',
        input: { command: 'true' },
        title: 'True',
        time: { start: 1000, end: 1001 },
      },
    });
    const { container } = render(<ToolUseCard part={part} />);
    expect(container.querySelector('.tool-output-content')).not.toBeInTheDocument();
  });

  it('should render different tool types', () => {
    const readPart = makeToolPart({
      tool: 'read_file',
      state: {
        status: 'completed',
        input: { file_path: '/some/file.ts' },
        output: 'file contents',
        title: 'Read file.ts',
        time: { start: 1000, end: 1002 },
      },
    });
    render(<ToolUseCard part={readPart} />);
    expect(screen.getByText('read_file')).toBeInTheDocument();
    expect(screen.getByText('Read file.ts', { selector: '.tool-title' })).toBeInTheDocument();
  });

  it('should display tool name from part.title when available', () => {
    const part = makeToolPart({
      tool: 'bash',
      state: {
        status: 'completed',
        input: { command: 'ls -la' },
        output: 'file1\nfile2',
        title: 'List files',
        time: { start: 1000, end: 1003 },
      },
    });
    render(<ToolUseCard part={part} />);
    expect(screen.getByText('bash')).toBeInTheDocument();
    expect(screen.getByText('List files', { selector: '.tool-title' })).toBeInTheDocument();
  });

  it('should use callID as data attribute for stable identification', () => {
    const { container } = render(<ToolUseCard part={makeToolPart()} />);
    const card = container.querySelector('.tool-use-card');
    expect(card?.getAttribute('data-call-id')).toBe('call_1');
  });
});

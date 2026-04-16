import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { OutputViewer } from './OutputViewer';

describe('OutputViewer', () => {
  describe('Empty State', () => {
    it('should render empty state when content is empty string', () => {
      render(<OutputViewer content="" />);

      expect(screen.getByText(/no output/i)).toBeInTheDocument();
    });

    it('should render empty state when content is whitespace only', () => {
      render(<OutputViewer content="   " />);

      expect(screen.getByText(/no output/i)).toBeInTheDocument();
    });

    it('should render execution error message when output is empty for terminal status', () => {
      render(
        <OutputViewer
          content=""
          execution={{
            id: 'exec-1',
            task_id: 'task-1',
            status: 'failed',
            started_at: '2024-01-01T00:00:00Z',
            error_message: 'Application was terminated unexpectedly',
          }}
        />
      );

      expect(screen.getByText('Application was terminated unexpectedly')).toBeInTheDocument();
      expect(screen.queryByText(/no output/i)).not.toBeInTheDocument();
    });
  });

  describe('Props', () => {
    it('should accept execution prop as null', () => {
      const { container } = render(<OutputViewer content="Test" execution={null} />);

      expect(container.firstChild).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should be accessible', () => {
      const { container } = render(<OutputViewer content="Test content" />);

      expect(container.firstChild).toBeInTheDocument();
    });
  });

  describe('JSONL Rendering', () => {
    const textEvent = JSON.stringify({
      type: 'text',
      timestamp: 1000,
      sessionID: 'ses_1',
      part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'JSONL text' },
    });

    const toolEvent = JSON.stringify({
      type: 'tool_use',
      timestamp: 2000,
      sessionID: 'ses_1',
      part: {
        type: 'tool',
        tool: 'bash',
        callID: 'call_1',
        state: {
          status: 'completed',
          input: { command: 'echo test' },
          output: 'test\n',
          metadata: { exit: 0 },
          title: 'Echo test',
          time: { start: 2000, end: 2003 },
        },
        id: 'p2',
        sessionID: 'ses_1',
        messageID: 'm1',
      },
    });

    it('should use EventRenderer for .jsonl output files', () => {
      render(
        <OutputViewer
          content={`${textEvent}\n${toolEvent}\n`}
          execution={{
            id: 'exec-1',
            task_id: 'task-1',
            status: 'success',
            started_at: '2024-01-01T00:00:00Z',
            output_file: 'output.jsonl',
          }}
        />
      );

      expect(screen.getByText('JSONL text')).toBeInTheDocument();
      expect(screen.getByText('Echo test')).toBeInTheDocument();
    });

    it('should show empty state for empty .jsonl content', () => {
      render(
        <OutputViewer
          content=""
          execution={{
            id: 'exec-1',
            task_id: 'task-1',
            status: 'success',
            started_at: '2024-01-01T00:00:00Z',
            output_file: 'output.jsonl',
          }}
        />
      );

      expect(screen.getByText(/no output/i)).toBeInTheDocument();
    });

    it('should show error message for failed .jsonl execution', () => {
      render(
        <OutputViewer
          content=""
          execution={{
            id: 'exec-1',
            task_id: 'task-1',
            status: 'failed',
            started_at: '2024-01-01T00:00:00Z',
            output_file: 'output.jsonl',
            error_message: 'Process crashed',
          }}
        />
      );

      expect(screen.getByText('Process crashed')).toBeInTheDocument();
    });
  });
});

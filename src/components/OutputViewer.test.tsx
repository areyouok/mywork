import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { OutputViewer } from './OutputViewer';

describe('OutputViewer', () => {
  describe('Rendering', () => {
    it('should render component with content', () => {
      render(<OutputViewer content="Test content" />);

      expect(screen.getByText('Test content')).toBeInTheDocument();
    });

    it('should render plain text content', () => {
      render(<OutputViewer content="Plain text content" isMarkdown={false} />);

      expect(screen.getByText('Plain text content')).toBeInTheDocument();
    });

    it('should render content with isMarkdown prop (treated as plain text)', () => {
      render(<OutputViewer content="# Heading" isMarkdown={true} />);

      expect(screen.getByText('# Heading')).toBeInTheDocument();
    });
  });

  describe('ANSI Content', () => {
    it('should render content with ANSI codes', () => {
      const esc = String.fromCharCode(0x1b);
      render(<OutputViewer content={`${esc}[31mRed Text${esc}[0m`} />);

      const redElement = screen.getByText('Red Text');
      expect(redElement).toBeInTheDocument();
    });

    it('should render content with multiple ANSI codes', () => {
      const esc = String.fromCharCode(0x1b);
      render(<OutputViewer content={`${esc}[1mBold${esc}[0m ${esc}[32mGreen${esc}[0m`} />);

      expect(screen.getByText('Bold')).toBeInTheDocument();
      expect(screen.getByText('Green')).toBeInTheDocument();
    });
  });

  describe('Empty State', () => {
    it('should render empty state when content is empty string', () => {
      render(<OutputViewer content="" />);

      expect(screen.getByText(/no output/i)).toBeInTheDocument();
    });

    it('should render empty state when content is whitespace only', () => {
      render(<OutputViewer content="   " />);

      expect(screen.getByText(/no output/i)).toBeInTheDocument();
    });

    it('should not render empty state when content exists', () => {
      render(<OutputViewer content="Some content" />);

      expect(screen.queryByText(/no output/i)).not.toBeInTheDocument();
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
    it('should accept content prop', () => {
      render(<OutputViewer content="Test content" />);

      expect(screen.getByText('Test content')).toBeInTheDocument();
    });

    it('should accept isMarkdown prop as true', () => {
      const { container } = render(<OutputViewer content="Test" isMarkdown={true} />);

      expect(container.firstChild).toBeInTheDocument();
    });

    it('should accept isMarkdown prop as false', () => {
      const { container } = render(<OutputViewer content="Test" isMarkdown={false} />);

      expect(container.firstChild).toBeInTheDocument();
    });

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
});

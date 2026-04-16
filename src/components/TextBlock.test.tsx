import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { TextBlock } from './TextBlock';
import type { TextPart } from '@/types/event';

describe('TextBlock', () => {
  const basePart: TextPart = {
    type: 'text',
    id: 'p1',
    messageID: 'm1',
    sessionID: 'ses_1',
    text: 'Hello world',
  };

  it('should render text content', () => {
    render(<TextBlock part={basePart} />);
    expect(screen.getByText('Hello world')).toBeInTheDocument();
  });

  it('should render with text-block class', () => {
    const { container } = render(<TextBlock part={basePart} />);
    expect(container.querySelector('.text-block')).toBeInTheDocument();
  });

  it('should render empty text without errors', () => {
    const { container } = render(<TextBlock part={{ ...basePart, text: '' }} />);
    expect(container.querySelector('.text-block')).toBeInTheDocument();
  });

  it('should preserve newlines with white-space pre-wrap', () => {
    const multilinePart: TextPart = {
      ...basePart,
      text: 'Line 1\nLine 2\nLine 3',
    };
    const { container } = render(<TextBlock part={multilinePart} />);
    const textEl = container.querySelector('.text-block-content');
    expect(textEl).toBeInTheDocument();
    expect(textEl).not.toHaveAttribute('style');
  });

  it('should render multi-line text', () => {
    const multilinePart: TextPart = {
      ...basePart,
      text: 'First line\nSecond line',
    };
    const { container } = render(<TextBlock part={multilinePart} />);
    const content = container.querySelector('.text-block-content');
    expect(content?.textContent).toBe('First line\nSecond line');
  });
});

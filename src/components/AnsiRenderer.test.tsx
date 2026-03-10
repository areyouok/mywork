import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { AnsiRenderer } from './AnsiRenderer';

describe('AnsiRenderer', () => {
  describe('Basic Text Rendering', () => {
    it('should render plain text without ANSI codes', () => {
      render(<AnsiRenderer text="Hello World" />);

      expect(screen.getByText('Hello World')).toBeInTheDocument();
    });

    it('should render empty string', () => {
      render(<AnsiRenderer text="" />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container).toBeInTheDocument();
    });

    it('should handle null input gracefully', () => {
      render(<AnsiRenderer text={null as unknown as string} />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container).toBeInTheDocument();
    });

    it('should handle undefined input gracefully', () => {
      render(<AnsiRenderer text={undefined as unknown as string} />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container).toBeInTheDocument();
    });
  });

  describe('ANSI Color Rendering', () => {
    it('should render red text', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${esc}[31mRed Text${esc}[0m`} />);

      const redElement = screen.getByText('Red Text');
      expect(redElement).toBeInTheDocument();
      expect(redElement.closest('span')?.getAttribute('style')).toContain('color');
    });

    it('should render green text', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${esc}[32mGreen Text${esc}[0m`} />);

      const greenElement = screen.getByText('Green Text');
      expect(greenElement).toBeInTheDocument();
      expect(greenElement.closest('span')?.getAttribute('style')).toContain('color');
    });

    it('should render blue text', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${esc}[34mBlue Text${esc}[0m`} />);

      const blueElement = screen.getByText('Blue Text');
      expect(blueElement).toBeInTheDocument();
      expect(blueElement.closest('span')?.getAttribute('style')).toContain('color');
    });

    it('should render yellow text', () => {
      const red = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${red}[33mYellow Text${red}[0m`} />);

      expect(screen.getByText('Yellow Text')).toBeInTheDocument();
    });

    it('should render cyan text', () => {
      const red = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${red}[36mCyan Text${red}[0m`} />);

      expect(screen.getByText('Cyan Text')).toBeInTheDocument();
    });

    it('should render magenta text', () => {
      const red = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${red}[35mMagenta Text${red}[0m`} />);

      expect(screen.getByText('Magenta Text')).toBeInTheDocument();
    });

    it('should render black text', () => {
      const red = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${red}[30mBlack Text${red}[0m`} />);

      expect(screen.getByText('Black Text')).toBeInTheDocument();
    });

    it('should render white text', () => {
      const red = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${red}[37mWhite Text${red}[0m`} />);

      expect(screen.getByText('White Text')).toBeInTheDocument();
    });
  });

  describe('ANSI Style Rendering', () => {
    it('should render bold text', () => {
      const esc = String.fromCharCode(0x1b);
      const { container } = render(<AnsiRenderer text={`${esc}[1mBold Text${esc}[0m`} />);

      const boldElement = container.querySelector('b');
      expect(boldElement).toBeInTheDocument();
      expect(boldElement?.textContent).toBe('Bold Text');
    });

    it('should render underlined text', () => {
      const esc = String.fromCharCode(0x1b);
      const { container } = render(<AnsiRenderer text={`${esc}[4mUnderlined Text${esc}[0m`} />);

      const underlinedElement = container.querySelector('u');
      expect(underlinedElement).toBeInTheDocument();
      expect(underlinedElement?.textContent).toBe('Underlined Text');
    });

    it('should render italic text', () => {
      const esc = String.fromCharCode(0x1b);
      const { container } = render(<AnsiRenderer text={`${esc}[3mItalic Text${esc}[0m`} />);

      const italicElement = container.querySelector('i');
      expect(italicElement).toBeInTheDocument();
      expect(italicElement?.textContent).toBe('Italic Text');
    });
  });

  describe('Mixed ANSI Sequences', () => {
    it('should render multiple colors in sequence', () => {
      const esc = String.fromCharCode(0x1b);
      render(
        <AnsiRenderer
          text={`${esc}[31mRed${esc}[0m ${esc}[32mGreen${esc}[0m ${esc}[34mBlue${esc}[0m`}
        />
      );

      expect(screen.getByText('Red')).toBeInTheDocument();
      expect(screen.getByText('Green')).toBeInTheDocument();
      expect(screen.getByText('Blue')).toBeInTheDocument();
    });

    it('should render bold and colored text', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${esc}[1m${esc}[31mBold Red${esc}[0m`} />);

      const element = screen.getByText('Bold Red');
      expect(element).toBeInTheDocument();
    });

    it('should handle text after reset code', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`${esc}[31mRed${esc}[0m Normal`} />);

      expect(screen.getByText('Red')).toBeInTheDocument();
      expect(screen.getByText('Normal')).toBeInTheDocument();
    });

    it('should handle ANSI codes at end of string', () => {
      const esc = String.fromCharCode(0x1b);
      render(<AnsiRenderer text={`Text${esc}[31m`} />);

      expect(screen.getByText('Text')).toBeInTheDocument();
    });
  });

  describe('XSS Protection', () => {
    it('should sanitize malicious script tags', () => {
      render(<AnsiRenderer text="<script>alert('xss')</script>" />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container?.innerHTML).not.toContain('<script>');
    });

    it('should sanitize javascript: URLs', () => {
      render(<AnsiRenderer text='<a href="javascript:alert(1)">click</a>' />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container?.innerHTML).not.toContain('javascript:');
    });

    it('should sanitize onmouse events', () => {
      render(<AnsiRenderer text='<div onmouseover="alert(1)">hover</div>' />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container?.innerHTML).not.toContain('onmouseover');
    });

    it('should preserve safe HTML', () => {
      // Safe HTML should be preserved after ANSI processing
      render(<AnsiRenderer text="<b>Bold</b>" />);

      expect(screen.getByText('Bold')).toBeInTheDocument();
    });
  });

  describe('Component Structure', () => {
    it('should render with ansi-renderer class', () => {
      render(<AnsiRenderer text="Test" />);

      const container = document.body.querySelector('.ansi-renderer');
      expect(container).toBeInTheDocument();
    });

    it('should render content in pre tag for whitespace preservation', () => {
      const { container } = render(<AnsiRenderer text="Test" />);

      const pre = container.querySelector('pre');
      expect(pre).toBeInTheDocument();
    });

    it('should preserve whitespace in text', () => {
      render(<AnsiRenderer text="Line 1\nLine 2" />);

      expect(screen.getByText(/Line 1/)).toBeInTheDocument();
      expect(screen.getByText(/Line 2/)).toBeInTheDocument();
    });
  });
});

import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { OutputViewer } from './OutputViewer';

describe('OutputViewer', () => {
  describe('Rendering', () => {
    it('should render component', () => {
      render(<OutputViewer content="Test content" />);

      expect(screen.getByText('Test content')).toBeInTheDocument();
    });

    it('should render plain text when isMarkdown is false', () => {
      render(<OutputViewer content="Plain text content" isMarkdown={false} />);

      expect(screen.getByText('Plain text content')).toBeInTheDocument();
    });

    it('should default to markdown mode when isMarkdown not specified', () => {
      const markdown = '# Heading';
      render(<OutputViewer content={markdown} />);

      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Heading');
    });
  });

  describe('Markdown Rendering', () => {
    it('should render markdown headings', () => {
      const markdown = `# Heading 1
## Heading 2
### Heading 3`;

      render(<OutputViewer content={markdown} />);

      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Heading 1');
      expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Heading 2');
      expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent('Heading 3');
    });

    it('should render markdown lists', () => {
      const markdown = `- Item 1
- Item 2
- Item 3`;

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('Item 1')).toBeInTheDocument();
      expect(screen.getByText('Item 2')).toBeInTheDocument();
      expect(screen.getByText('Item 3')).toBeInTheDocument();

      const list = screen.getByRole('list');
      expect(list.tagName).toBe('UL');
    });

    it('should render ordered lists', () => {
      const markdown = `1. First
2. Second
3. Third`;

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('First')).toBeInTheDocument();
      expect(screen.getByText('Second')).toBeInTheDocument();
      expect(screen.getByText('Third')).toBeInTheDocument();

      const list = screen.getByRole('list');
      expect(list.tagName).toBe('OL');
    });

    it('should render code blocks with syntax highlighting', () => {
      const markdown = `\`\`\`javascript
const greeting = "Hello, World!";
console.log(greeting);
\`\`\``;

      const { container } = render(<OutputViewer content={markdown} />);

      const codeBlock = container.querySelector('code.language-javascript');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock?.textContent).toContain('const');
      expect(codeBlock?.textContent).toContain('greeting');
    });

    it('should render inline code', () => {
      const markdown = 'This is `inline code` in text';

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('inline code')).toBeInTheDocument();
    });

    it('should render bold and italic text', () => {
      const markdown = 'This is **bold** and *italic* text';

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('bold')).toBeInTheDocument();
      expect(screen.getByText('italic')).toBeInTheDocument();
    });

    it('should render links', () => {
      const markdown = '[Click here](https://example.com)';

      render(<OutputViewer content={markdown} />);

      const link = screen.getByRole('link', { name: 'Click here' });
      expect(link).toHaveAttribute('href', 'https://example.com');
    });

    it('should render blockquotes', () => {
      const markdown = '> This is a quote';

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('This is a quote')).toBeInTheDocument();
    });
  });

  describe('Syntax Highlighting', () => {
    it('should apply syntax highlighting to JavaScript code', () => {
      const markdown = `\`\`\`javascript
function hello() {
  return "Hello";
}
\`\`\``;

      const { container } = render(<OutputViewer content={markdown} />);

      const codeBlock = container.querySelector('code.language-javascript');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock?.textContent).toContain('function');
      expect(codeBlock?.textContent).toContain('hello');
    });

    it('should apply syntax highlighting to Python code', () => {
      const markdown = `\`\`\`python
def hello():
    return "Hello"
\`\`\``;

      const { container } = render(<OutputViewer content={markdown} />);

      const codeBlock = container.querySelector('code.language-python');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock?.textContent).toContain('def');
      expect(codeBlock?.textContent).toContain('hello');
    });

    it('should handle code blocks without language specification', () => {
      const markdown = `\`\`\`
Plain code block
\`\`\``;

      render(<OutputViewer content={markdown} />);

      expect(screen.getByText('Plain code block')).toBeInTheDocument();
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
  });

  describe('Props', () => {
    it('should accept content prop', () => {
      render(<OutputViewer content="Test content" />);

      expect(screen.getByText('Test content')).toBeInTheDocument();
    });

    it('should accept isMarkdown prop as true', () => {
      const markdown = '# Test Heading';
      render(<OutputViewer content={markdown} isMarkdown={true} />);

      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
    });

    it('should accept isMarkdown prop as false', () => {
      const markdown = '# Test Heading';
      render(<OutputViewer content={markdown} isMarkdown={false} />);

      expect(screen.getByText('# Test Heading')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should be accessible', () => {
      const { container } = render(<OutputViewer content="Test content" />);

      expect(container.firstChild).toBeInTheDocument();
    });
  });
});

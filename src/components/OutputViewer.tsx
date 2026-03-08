import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import ReactMarkdown from 'react-markdown';
import './OutputViewer.css';

interface OutputViewerProps {
  content: string;
  isMarkdown?: boolean;
}

export function OutputViewer({ content, isMarkdown = true }: OutputViewerProps) {
  const isEmpty = !content || content.trim() === '';

  if (isEmpty) {
    return (
      <div className="output-viewer output-viewer-empty">
        <p>No output to display</p>
      </div>
    );
  }

  if (!isMarkdown) {
    return (
      <div className="output-viewer">
        <div className="output-viewer-content">
          <pre>{content}</pre>
        </div>
      </div>
    );
  }

  return (
    <div className="output-viewer">
      <div className="output-viewer-content">
        <ReactMarkdown
          components={{
            code({ className, children, ...props }) {
              const match = /language-(\w+)/.exec(className || '');
              const isInline = !match;

              if (isInline) {
                return (
                  <code className={className} {...props}>
                    {children}
                  </code>
                );
              }

              return (
                <SyntaxHighlighter style={vscDarkPlus} language={match[1]} PreTag="div">
                  {String(children).replace(/\n$/, '')}
                </SyntaxHighlighter>
              );
            },
          }}
        >
          {content}
        </ReactMarkdown>
      </div>
    </div>
  );
}

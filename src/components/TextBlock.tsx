import type { TextPart } from '@/types/event';
import './TextBlock.css';

interface TextBlockProps {
  part: TextPart;
}

export function TextBlock({ part }: TextBlockProps) {
  if (!part.text) {
    return (
      <div className="text-block">
        <div className="text-block-content" />
      </div>
    );
  }

  return (
    <div className="text-block">
      <div className="text-block-content">{part.text}</div>
    </div>
  );
}

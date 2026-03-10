import AnsiToHtml from 'ansi-to-html';
import DOMPurify from 'dompurify';

interface AnsiRendererProps {
  text: string | null | undefined;
}

const ansiConverter = new AnsiToHtml({
  fg: '#000',
  bg: '#fff',
  newline: true,
  colors: {
    0: '#000000',
    1: '#cd3131',
    2: '#0dbc79',
    3: '#e5e510',
    4: '#2472c8',
    5: '#bc3fbc',
    6: '#11a8cd',
    7: '#e5e5e5',
    8: '#666666',
    9: '#ff6666',
    10: '#66ff66',
  },
});

export function AnsiRenderer({ text }: AnsiRendererProps) {
  const safeText = text ?? '';
  const html = ansiConverter.toHtml(safeText);
  const sanitizedHtml = DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ['span', 'strong', 'em', 'u', 'br', 'pre', 'b', 'i'],
    ALLOWED_ATTR: ['class', 'style'],
  });

  return (
    <div className="ansi-renderer">
      <pre dangerouslySetInnerHTML={{ __html: sanitizedHtml }} />
    </div>
  );
}

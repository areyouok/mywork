import { describe, it, expect } from 'vitest';
import { formatAbsoluteTime, formatDuration } from './format';

describe('formatAbsoluteTime', () => {
  it('should format valid date string in ISO format', () => {
    const result = formatAbsoluteTime('2024-03-09T10:00:00Z');
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/);
  });

  it('should return "Invalid date" for invalid date string', () => {
    expect(formatAbsoluteTime('not-a-date')).toBe('Invalid date');
  });

  it('should return "Invalid date" for empty string', () => {
    expect(formatAbsoluteTime('')).toBe('Invalid date');
  });

  it('should pad single digit values with zeros', () => {
    const result = formatAbsoluteTime('2024-01-05T10:07:09Z');
    expect(result).toMatch(/2024-01-05/);
    expect(result).toMatch(/\d{2}:\d{2}:\d{2}/);
  });
});

describe('formatDuration', () => {
  it('should format duration in seconds', () => {
    expect(formatDuration('2024-03-09T10:00:00Z', '2024-03-09T10:00:30Z')).toBe('30 seconds');
  });

  it('should format duration in minutes', () => {
    expect(formatDuration('2024-03-09T10:00:00Z', '2024-03-09T10:05:00Z')).toBe('5 minutes');
  });

  it('should format duration in hours', () => {
    expect(formatDuration('2024-03-09T10:00:00Z', '2024-03-09T12:30:00Z')).toBe(
      '2 hours 30 minutes'
    );
  });

  it('should return <1s for sub-second duration', () => {
    expect(formatDuration('2024-03-09T10:00:00Z', '2024-03-09T10:00:00.500Z')).toBe('<1s');
  });
});

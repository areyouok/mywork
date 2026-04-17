import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { TodoList, parseTodoOutput } from './TodoList';

describe('parseTodoOutput', () => {
  it('should parse standard JSON array', () => {
    const input = JSON.stringify([
      { content: 'Task 1', status: 'completed' },
      { content: 'Task 2', status: 'pending' },
    ]);
    const result = parseTodoOutput(input);
    expect(result).toEqual([
      { content: 'Task 1', status: 'completed' },
      { content: 'Task 2', status: 'pending' },
    ]);
  });

  it('should parse nested { todos: [...] } structure', () => {
    const input = JSON.stringify({
      todos: [
        { content: 'Task A', status: 'in_progress' },
        { content: 'Task B', status: 'completed' },
      ],
    });
    const result = parseTodoOutput(input);
    expect(result).toEqual([
      { content: 'Task A', status: 'in_progress' },
      { content: 'Task B', status: 'completed' },
    ]);
  });

  it('should return null for invalid JSON', () => {
    expect(parseTodoOutput('not json')).toBeNull();
  });

  it('should return null for non-array JSON string', () => {
    expect(parseTodoOutput('"hello"')).toBeNull();
  });

  it('should return null for non-array JSON object', () => {
    expect(parseTodoOutput('{}')).toBeNull();
  });

  it('should return null when array contains non-object elements', () => {
    expect(parseTodoOutput('[1, "str"]')).toBeNull();
  });

  it('should return null when element lacks content field', () => {
    expect(parseTodoOutput('[{"status":"pending"}]')).toBeNull();
  });

  it('should return null when element lacks status field', () => {
    expect(parseTodoOutput('[{"content":"do stuff"}]')).toBeNull();
  });

  it('should return null when content is not a string', () => {
    expect(parseTodoOutput('[{"content":123,"status":"pending"}]')).toBeNull();
  });

  it('should return null when status is not a string', () => {
    expect(parseTodoOutput('[{"content":"task","status":true}]')).toBeNull();
  });

  it('should parse empty todos array', () => {
    expect(parseTodoOutput('[]')).toEqual([]);
  });

  it('should preserve optional priority field', () => {
    const input = JSON.stringify([{ content: 'Task', status: 'pending', priority: 'high' }]);
    const result = parseTodoOutput(input);
    expect(result).toEqual([{ content: 'Task', status: 'pending', priority: 'high' }]);
  });

  it('should tolerate unknown status values', () => {
    const input = JSON.stringify([{ content: 'Task', status: 'unknown_status' }]);
    const result = parseTodoOutput(input);
    expect(result).toEqual([{ content: 'Task', status: 'unknown_status' }]);
  });
});

describe('TodoList', () => {
  it('should render # Todos header and checklist for valid JSON array', () => {
    const output = JSON.stringify([
      { content: 'Task 1', status: 'completed' },
      { content: 'Task 2', status: 'pending' },
    ]);
    const { container } = render(<TodoList output={output} />);
    expect(container.querySelector('.todo-list-header')).toHaveTextContent('# Todos');
    expect(container.querySelectorAll('.todo-item')).toHaveLength(2);
  });

  it('should show [✓] for completed items', () => {
    const output = JSON.stringify([{ content: 'Done task', status: 'completed' }]);
    const { container } = render(<TodoList output={output} />);
    const checkbox = container.querySelector('.todo-checkbox');
    expect(checkbox).toHaveTextContent('[✓]');
    expect(checkbox).toHaveClass('todo-checkbox-completed');
  });

  it('should show [ ] for pending items', () => {
    const output = JSON.stringify([{ content: 'Pending task', status: 'pending' }]);
    const { container } = render(<TodoList output={output} />);
    const checkbox = container.querySelector('.todo-checkbox');
    expect(checkbox).toHaveTextContent('[ ]');
  });

  it('should show [ ] for in_progress items', () => {
    const output = JSON.stringify([{ content: 'Active task', status: 'in_progress' }]);
    const { container } = render(<TodoList output={output} />);
    const checkbox = container.querySelector('.todo-checkbox');
    expect(checkbox).toHaveTextContent('[ ]');
  });

  it('should add todo-item-cancelled class for cancelled items', () => {
    const output = JSON.stringify([{ content: 'Cancelled task', status: 'cancelled' }]);
    const { container } = render(<TodoList output={output} />);
    const item = container.querySelector('.todo-item');
    expect(item).toHaveClass('todo-item-cancelled');
    expect(item).not.toHaveClass('todo-item-completed');
  });

  it('should add todo-item-completed class for completed items', () => {
    const output = JSON.stringify([{ content: 'Done task', status: 'completed' }]);
    const { container } = render(<TodoList output={output} />);
    const item = container.querySelector('.todo-item');
    expect(item).toHaveClass('todo-item-completed');
  });

  it('should render nested { todos: [...] } structure', () => {
    const output = JSON.stringify({
      todos: [{ content: 'Nested task', status: 'pending' }],
    });
    const { container } = render(<TodoList output={output} />);
    expect(container.querySelectorAll('.todo-item')).toHaveLength(1);
    expect(screen.getByText('Nested task')).toBeInTheDocument();
  });

  it('should fall back to <pre> for invalid JSON', () => {
    const { container } = render(<TodoList output="not json at all" />);
    const pre = container.querySelector('pre');
    expect(pre).toBeInTheDocument();
    expect(pre).toHaveTextContent('not json at all');
    expect(container.querySelector('.todo-list-container')).not.toBeInTheDocument();
  });

  it('should fall back to <pre> for non-array JSON', () => {
    const { container } = render(<TodoList output='"hello"' />);
    const pre = container.querySelector('pre');
    expect(pre).toBeInTheDocument();
    expect(pre).toHaveTextContent('"hello"');
  });

  it('should fall back to <pre> for plain object JSON', () => {
    const { container } = render(<TodoList output='{"key":"value"}' />);
    const pre = container.querySelector('pre');
    expect(pre).toBeInTheDocument();
  });

  it('should render empty list without crashing', () => {
    const { container } = render(<TodoList output="[]" />);
    expect(container.querySelector('.todo-list-container')).toBeInTheDocument();
    expect(container.querySelector('.todo-list-header')).toHaveTextContent('# Todos');
    expect(container.querySelectorAll('.todo-item')).toHaveLength(0);
  });

  it('should safely render content with script tags', () => {
    const output = JSON.stringify([{ content: '<script>alert(1)</script>', status: 'pending' }]);
    const { container } = render(<TodoList output={output} />);
    expect(container.querySelector('script')).not.toBeInTheDocument();
    expect(screen.getByText('<script>alert(1)</script>')).toBeInTheDocument();
  });

  it('should correctly render content with newlines', () => {
    const output = JSON.stringify([{ content: 'line1\nline2', status: 'pending' }]);
    render(<TodoList output={output} />);
    expect(screen.getByText('line1\nline2', { normalizer: (s) => s })).toBeInTheDocument();
  });

  it('should fall back for array with non-object elements', () => {
    const output = JSON.stringify([1, 'str']);
    const { container } = render(<TodoList output={output} />);
    const pre = container.querySelector('pre');
    expect(pre).toBeInTheDocument();
  });

  it('should fall back for element missing content or status', () => {
    const { container } = render(<TodoList output='[{"status":"pending"}]' />);
    expect(container.querySelector('pre')).toBeInTheDocument();

    const { container: c2 } = render(<TodoList output='[{"content":"task"}]' />);
    expect(c2.querySelector('pre')).toBeInTheDocument();
  });

  it('should show [ ] for unknown status values', () => {
    const output = JSON.stringify([{ content: 'Mystery', status: 'unknown' }]);
    const { container } = render(<TodoList output={output} />);
    const checkbox = container.querySelector('.todo-checkbox');
    expect(checkbox).toHaveTextContent('[ ]');
    expect(checkbox).not.toHaveClass('todo-checkbox-completed');
  });

  it('should render multiple todo items completely', () => {
    const output = JSON.stringify([
      { content: 'Design API', status: 'completed', priority: 'high' },
      { content: 'Write tests', status: 'in_progress', priority: 'medium' },
      { content: 'Implement feature', status: 'pending', priority: 'low' },
      { content: 'Old idea', status: 'cancelled' },
    ]);
    const { container } = render(<TodoList output={output} />);
    const items = container.querySelectorAll('.todo-item');
    expect(items).toHaveLength(4);
    expect(items[0]).toHaveClass('todo-item-completed');
    expect(items[0].querySelector('.todo-checkbox')).toHaveTextContent('[✓]');
    expect(items[1].querySelector('.todo-checkbox')).toHaveTextContent('[ ]');
    expect(items[2].querySelector('.todo-checkbox')).toHaveTextContent('[ ]');
    expect(items[3]).toHaveClass('todo-item-cancelled');
    expect(items[3].querySelector('.todo-checkbox')).toHaveTextContent('[ ]');
    expect(screen.getByText('Design API')).toBeInTheDocument();
    expect(screen.getByText('Write tests')).toBeInTheDocument();
    expect(screen.getByText('Implement feature')).toBeInTheDocument();
    expect(screen.getByText('Old idea')).toBeInTheDocument();
  });
});

import './TodoList.css';

export interface TodoItem {
  content: string;
  status: 'pending' | 'in_progress' | 'completed' | 'cancelled';
  priority?: 'high' | 'medium' | 'low';
}

function isTodoItem(value: unknown): value is TodoItem {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  if (typeof obj.content !== 'string') return false;
  if (typeof obj.status !== 'string') return false;
  return true;
}

export function parseTodoOutput(output: string): TodoItem[] | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(output);
  } catch {
    return null;
  }

  let arr: unknown[];
  if (Array.isArray(parsed)) {
    arr = parsed;
  } else if (
    typeof parsed === 'object' &&
    parsed !== null &&
    'todos' in parsed &&
    Array.isArray((parsed as Record<string, unknown>).todos)
  ) {
    arr = (parsed as { todos: unknown[] }).todos;
  } else {
    return null;
  }

  for (const item of arr) {
    if (!isTodoItem(item)) return null;
  }

  return arr as TodoItem[];
}

export function TodoList({ output }: { output: string }) {
  const todos = parseTodoOutput(output);

  if (todos === null) {
    return <pre>{output}</pre>;
  }

  return (
    <div className="todo-list-container">
      <div className="todo-list-header"># Todos</div>
      <ul className="todo-list-items">
        {todos.map((todo, index) => {
          const isCompleted = todo.status === 'completed';
          const isCancelled = todo.status === 'cancelled';

          const itemClasses = ['todo-item'];
          if (isCompleted) itemClasses.push('todo-item-completed');
          if (isCancelled) itemClasses.push('todo-item-cancelled');

          const checkboxClasses = ['todo-checkbox-svg'];
          if (isCompleted) checkboxClasses.push('todo-checkbox-completed');

          return (
            <li key={index} className={itemClasses.join(' ')}>
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                className={checkboxClasses.join(' ')}
                aria-hidden="true"
              >
                <rect
                  x="1"
                  y="1"
                  width="14"
                  height="14"
                  rx="3"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  fill="none"
                />
                {isCompleted && (
                  <path
                    d="M4.5 8L7 10.5L11.5 5.5"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                )}
              </svg>
              <span className="todo-content">{todo.content}</span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useTasks } from './useTasks';
import * as api from '@/api/tasks';

vi.mock('@/api/tasks', () => ({
  getTasks: vi.fn(),
  createTask: vi.fn(),
  updateTask: vi.fn(),
  deleteTask: vi.fn(),
  reloadScheduler: vi.fn(),
}));

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe('useTasks', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loadTasks should set loading true then false when request resolves', async () => {
    const deferred = createDeferred<
      Array<{
        id: string;
        name: string;
        prompt: string;
        enabled: boolean;
        timeout_seconds: number;
        created_at: string;
        updated_at: string;
      }>
    >();

    vi.mocked(api.getTasks).mockReturnValueOnce(deferred.promise);

    const { result } = renderHook(() => useTasks());

    let loadPromise!: Promise<void>;
    act(() => {
      loadPromise = result.current.loadTasks();
    });

    expect(result.current.loading).toBe(true);

    deferred.resolve([
      {
        id: 'task-1',
        name: 'Task 1',
        prompt: 'prompt',
        enabled: true,
        timeout_seconds: 60,
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      },
    ]);

    await act(async () => {
      await loadPromise;
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.tasks).toHaveLength(1);
  });

  it('should not let stale loadTasks overwrite newer createTask result', async () => {
    const deferredLoad = createDeferred<
      Array<{
        id: string;
        name: string;
        prompt: string;
        enabled: boolean;
        timeout_seconds: number;
        created_at: string;
        updated_at: string;
      }>
    >();

    vi.mocked(api.getTasks).mockReturnValueOnce(deferredLoad.promise);
    vi.mocked(api.createTask).mockResolvedValueOnce({
      id: 'task-new',
      name: 'New Task',
      prompt: 'new prompt',
      enabled: true,
      timeout_seconds: 120,
      created_at: '2026-01-02T00:00:00Z',
      updated_at: '2026-01-02T00:00:00Z',
    });
    const { result } = renderHook(() => useTasks());

    let loadPromise!: Promise<void>;
    act(() => {
      loadPromise = result.current.loadTasks();
    });

    expect(result.current.loading).toBe(true);

    await act(async () => {
      await result.current.createTask({
        name: 'New Task',
        prompt: 'new prompt',
        timeout_seconds: 120,
        enabled: 1,
      });
    });

    expect(result.current.tasks).toHaveLength(1);
    expect(result.current.tasks[0].id).toBe('task-new');

    deferredLoad.resolve([
      {
        id: 'task-old',
        name: 'Old Task',
        prompt: 'old prompt',
        enabled: true,
        timeout_seconds: 30,
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      },
    ]);

    await act(async () => {
      await loadPromise;
    });

    expect(result.current.loading).toBe(false);

    expect(result.current.tasks).toHaveLength(1);
    expect(result.current.tasks[0].id).toBe('task-new');
    expect(vi.mocked(api.reloadScheduler)).not.toHaveBeenCalled();
  });

  it('should update task locally without calling reloadScheduler', async () => {
    vi.mocked(api.updateTask).mockResolvedValueOnce({
      id: 'task-1',
      name: 'Updated Task',
      prompt: 'updated prompt',
      enabled: true,
      timeout_seconds: 60,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-03T00:00:00Z',
    });

    const { result } = renderHook(() => useTasks());

    await act(async () => {
      await result.current.updateTask('task-1', { name: 'Updated Task' });
    });

    expect(vi.mocked(api.updateTask)).toHaveBeenCalledWith('task-1', { name: 'Updated Task' });
    expect(vi.mocked(api.reloadScheduler)).not.toHaveBeenCalled();
    expect(result.current.tasks).toHaveLength(0);
  });
});

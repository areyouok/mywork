import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useStreamingOutput } from './useStreamingOutput';
import * as tasksApi from '@/api/tasks';

vi.mock('@/api/tasks', () => ({
  getOutput: vi.fn(),
  getExecution: vi.fn(),
}));

describe('useStreamingOutput', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('初始状态正确', () => {
    const { result } = renderHook(() => useStreamingOutput());
    expect(result.current.output).toBe('');
    expect(result.current.isStreaming).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('startStreaming 会加载初始输出并进入 streaming 状态', async () => {
    vi.mocked(tasksApi.getOutput).mockResolvedValue('line1\n');
    vi.mocked(tasksApi.getExecution).mockResolvedValue({
      id: 'exec-1',
      task_id: 'task-1',
      status: 'running',
      started_at: new Date().toISOString(),
    });

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    expect(tasksApi.getOutput).toHaveBeenCalledWith('exec-1');
    expect(result.current.output).toBe('line1\n');
    expect(result.current.isStreaming).toBe(true);
  });

  it('轮询时会刷新输出并在任务完成后停止', async () => {
    vi.mocked(tasksApi.getOutput)
      .mockResolvedValueOnce('line1\n')
      .mockResolvedValueOnce('line1\nline2\n')
      .mockResolvedValueOnce('line1\nline2\n');
    vi.mocked(tasksApi.getExecution)
      .mockResolvedValueOnce({
        id: 'exec-1',
        task_id: 'task-1',
        status: 'running',
        started_at: new Date().toISOString(),
      })
      .mockResolvedValueOnce({
        id: 'exec-1',
        task_id: 'task-1',
        status: 'success',
        started_at: new Date().toISOString(),
        finished_at: new Date().toISOString(),
      });

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
      await Promise.resolve();
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
      await Promise.resolve();
    });

    expect(result.current.output).toBe('line1\nline2\n');
    expect(result.current.isStreaming).toBe(false);
  });

  it('初次加载失败会设置 error 并停止', async () => {
    vi.mocked(tasksApi.getOutput).mockRejectedValue(new Error('load failed'));

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    expect(result.current.error).toBe('load failed');
    expect(result.current.isStreaming).toBe(false);
  });

  it('stopStreaming 会停止并清空输出', async () => {
    vi.mocked(tasksApi.getOutput).mockResolvedValue('line1\n');
    vi.mocked(tasksApi.getExecution).mockResolvedValue({
      id: 'exec-1',
      task_id: 'task-1',
      status: 'running',
      started_at: new Date().toISOString(),
    });

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    act(() => {
      result.current.stopStreaming();
    });

    expect(result.current.isStreaming).toBe(false);
    expect(result.current.output).toBe('');
  });
});

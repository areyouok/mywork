import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useStreamingOutput } from './useStreamingOutput';
import * as tasksApi from '@/api/tasks';

vi.mock('@/api/tasks', () => ({
  getOutput: vi.fn(),
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
    expect(result.current.events).toEqual([]);
    expect(result.current.isStreaming).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('startStreaming 会加载初始输出并进入 streaming 状态', async () => {
    vi.mocked(tasksApi.getOutput).mockResolvedValue('line1\n');

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    expect(tasksApi.getOutput).toHaveBeenCalledWith('exec-1');
    expect(result.current.output).toBe('line1\n');
    expect(result.current.isStreaming).toBe(true);
  });

  it('轮询时会刷新输出并保持不回退', async () => {
    vi.mocked(tasksApi.getOutput)
      .mockResolvedValueOnce('a\n')
      .mockResolvedValueOnce('a\nb\n')
      .mockResolvedValueOnce('a\n');

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
      await Promise.resolve();
    });

    expect(result.current.output).toBe('a\nb\n');

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
      await Promise.resolve();
    });

    expect(result.current.output).toBe('a\nb\n');
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

    const { result } = renderHook(() => useStreamingOutput());

    await act(async () => {
      await result.current.startStreaming('exec-1');
    });

    act(() => {
      result.current.stopStreaming();
    });

    expect(result.current.isStreaming).toBe(false);
    expect(result.current.output).toBe('');
    expect(result.current.events).toEqual([]);
  });

  describe('JSONL 事件增量解析', () => {
    it('应解析 JSONL 内容为事件数组', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      });
      const event2 = JSON.stringify({
        type: 'tool_use',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: {
          type: 'tool',
          tool: 'bash',
          callID: 'call_1',
          state: { status: 'completed', input: { command: 'ls' } },
          id: 'p2',
          sessionID: 'ses_1',
          messageID: 'm1',
        },
      });

      vi.mocked(tasksApi.getOutput).mockResolvedValueOnce(`${event1}\n${event2}\n`);

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(2);
      expect(result.current.events[0].type).toBe('text');
      expect(result.current.events[1].type).toBe('tool_use');
    });

    it('应增量解析新增行', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'first' },
      });
      const event2 = JSON.stringify({
        type: 'text',
        timestamp: 2000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p2', messageID: 'm1', sessionID: 'ses_1', text: 'second' },
      });

      vi.mocked(tasksApi.getOutput)
        .mockResolvedValueOnce(`${event1}\n`)
        .mockResolvedValueOnce(`${event1}\n${event2}\n`);

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(1);

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
        await Promise.resolve();
      });

      expect(result.current.events).toHaveLength(2);
      const textEvent = result.current.events[1] as { type: 'text'; part: { text: string } };
      expect(textEvent.part.text).toBe('second');
    });

    it('应跳过非 JSON 行', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      });

      vi.mocked(tasksApi.getOutput).mockResolvedValueOnce(`not json\n${event1}\n`);

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(1);
    });

    it('切换 executionId 时应重置 events', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      });

      vi.mocked(tasksApi.getOutput).mockResolvedValueOnce(`${event1}\n`).mockResolvedValueOnce('');

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(1);

      await act(async () => {
        await result.current.startStreaming('exec-2');
      });

      expect(result.current.events).toHaveLength(0);
    });

    it('streaming 结尾无换行符时 stopStreaming(clearOutput=false) 应 flush 最后事件', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      });

      vi.mocked(tasksApi.getOutput).mockResolvedValueOnce(`${event1}\n${event1}`);

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(1);

      act(() => {
        result.current.stopStreaming(false);
      });

      expect(result.current.events).toHaveLength(2);
    });

    it('lastNewlineIdx === 0 时不应产生多余事件', async () => {
      const event1 = JSON.stringify({
        type: 'text',
        timestamp: 1000,
        sessionID: 'ses_1',
        part: { type: 'text', id: 'p1', messageID: 'm1', sessionID: 'ses_1', text: 'hello' },
      });

      vi.mocked(tasksApi.getOutput).mockResolvedValueOnce(`\n${event1}\n`);

      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('exec-1');
      });

      expect(result.current.events).toHaveLength(1);
    });
  });
});

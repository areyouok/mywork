import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';

type OutputEvent =
  | { stdout: { text: string } }
  | { stderr: { text: string } }
  | { finished: { exitCode: number } };

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => {
  class MockChannel {
    private _onmessage: ((msg: OutputEvent) => void) | null = null;
    static lastInstance: MockChannel | null = null;

    constructor() {
      MockChannel.lastInstance = this;
    }

    set onmessage(handler: ((msg: OutputEvent) => void) | null) {
      this._onmessage = handler;
    }

    get onmessage() {
      return this._onmessage;
    }
  }

  return {
    invoke: mockInvoke,
    Channel: MockChannel,
  };
});

import { Channel } from '@tauri-apps/api/core';
import { useStreamingOutput } from './useStreamingOutput';

describe('useStreamingOutput', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  function sendMessage(msg: OutputEvent) {
    const channel = Channel as unknown as {
      lastInstance?: { onmessage: ((msg: OutputEvent) => void) | null };
    };
    if (channel.lastInstance?.onmessage) {
      channel.lastInstance.onmessage(msg);
    }
  }

  describe('初始状态', () => {
    it('output 初始为空字符串', () => {
      const { result } = renderHook(() => useStreamingOutput());
      expect(result.current.output).toBe('');
    });

    it('isStreaming 初始为 false', () => {
      const { result } = renderHook(() => useStreamingOutput());
      expect(result.current.isStreaming).toBe(false);
    });

    it('error 初始为 null', () => {
      const { result } = renderHook(() => useStreamingOutput());
      expect(result.current.error).toBeNull();
    });
  });

  describe('startStreaming', () => {
    it('调用成功时设置 isStreaming = true', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt', '/path/to/cwd');
      });

      expect(result.current.isStreaming).toBe(true);
    });

    it('调用 invoke 时传入正确的参数', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt', '/path/to/cwd');
      });

      expect(mockInvoke).toHaveBeenCalledWith('execute_task_streaming', {
        taskId: 'task-123',
        prompt: 'test prompt',
        cwd: '/path/to/cwd',
        channel: expect.any(Object),
      });
    });

    it('调用失败时设置 error', async () => {
      mockInvoke.mockRejectedValue(new Error('Connection failed'));
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        try {
          await result.current.startStreaming('task-123', 'test prompt');
        } catch {}
      });

      expect(result.current.error).toBe('Connection failed');
      expect(result.current.isStreaming).toBe(false);
    });
  });

  describe('事件处理', () => {
    it('收到 Stdout 事件时追加到 output', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ stdout: { text: 'Hello from stdout' } });
      });

      expect(result.current.output).toBe('Hello from stdout\n');
    });

    it('收到 Stderr 事件时追加到 output', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ stderr: { text: 'Warning from stderr' } });
      });

      expect(result.current.output).toBe('Warning from stderr\n');
    });

    it('收到多个事件时按顺序累积', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ stdout: { text: 'Line 1' } });
      });

      act(() => {
        sendMessage({ stderr: { text: 'Error line' } });
      });

      act(() => {
        sendMessage({ stdout: { text: 'Line 2' } });
      });

      expect(result.current.output).toBe('Line 1\nError line\nLine 2\n');
    });

    it('收到 Finished 事件时设置 isStreaming = false', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      expect(result.current.isStreaming).toBe(true);

      act(() => {
        sendMessage({ finished: { exitCode: 0 } });
      });

      expect(result.current.isStreaming).toBe(false);
    });

    it('收到非零退出码时也设置 isStreaming = false', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ finished: { exitCode: 1 } });
      });

      expect(result.current.isStreaming).toBe(false);
    });
  });

  describe('stopStreaming', () => {
    it('停止时重置 isStreaming 为 false', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      expect(result.current.isStreaming).toBe(true);

      act(() => {
        result.current.stopStreaming();
      });

      expect(result.current.isStreaming).toBe(false);
    });

    it('停止时清空 output', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ stdout: { text: 'Some output' } });
      });

      expect(result.current.output).toBe('Some output\n');

      act(() => {
        result.current.stopStreaming();
      });

      expect(result.current.output).toBe('');
    });
  });

  describe('resetOutput', () => {
    it('重置 output 为空字符串', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { result } = renderHook(() => useStreamingOutput());

      await act(async () => {
        await result.current.startStreaming('task-123', 'test prompt');
      });

      act(() => {
        sendMessage({ stdout: { text: 'Some output' } });
      });

      expect(result.current.output).toBe('Some output\n');

      act(() => {
        result.current.resetOutput();
      });

      expect(result.current.output).toBe('');
    });
  });
});

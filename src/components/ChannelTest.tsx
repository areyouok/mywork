import { useState } from 'react';
import { invoke, Channel } from '@tauri-apps/api/core';

type PoCEvent =
  | { event: 'message'; data: { text: string } }
  | { event: 'progress'; data: { percent: number } }
  | { event: 'done'; data: null };

export function ChannelTest() {
  const [logs, setLogs] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);

  const runTest = async () => {
    setIsRunning(true);
    setLogs(['Starting channel test...']);

    const channel = new Channel<PoCEvent>();
    channel.onmessage = (msg) => {
      switch (msg.event) {
        case 'message':
          setLogs((prev) => [...prev, `[MESSAGE] ${msg.data.text}`]);
          break;
        case 'progress':
          setLogs((prev) => [...prev, `[PROGRESS] ${msg.data.percent}%`]);
          break;
        case 'done':
          setLogs((prev) => [...prev, '[DONE] Stream complete!']);
          setIsRunning(false);
          break;
      }
    };

    try {
      await invoke('test_channel_stream', { onEvent: channel });
    } catch (e) {
      setLogs((prev) => [...prev, `[ERROR] ${e}`]);
      setIsRunning(false);
    }
  };

  return (
    <div className="channel-test">
      <h3>Channel PoC Test</h3>
      <button onClick={runTest} disabled={isRunning}>
        {isRunning ? 'Running...' : 'Run Stream Test'}
      </button>
      <div className="test-logs">
        {logs.map((log, i) => (
          <div key={i} className="log-line">
            {log}
          </div>
        ))}
      </div>
    </div>
  );
}

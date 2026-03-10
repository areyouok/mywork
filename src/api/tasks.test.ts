import { describe, it, expect } from 'vitest';
import { convertTask } from './tasks';
import type { RawTask } from './tasks';

describe('API Tasks', () => {
  describe('convertTask', () => {
    it('should convert enabled from 1 to true', () => {
      const raw: RawTask = {
        id: '1',
        name: 'Test Task',
        prompt: 'Test prompt',
        enabled: 1,
        timeout_seconds: 300,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      const task = convertTask(raw);

      expect(task.enabled).toBe(true);
    });

    it('should convert enabled from 0 to false', () => {
      const raw: RawTask = {
        id: '1',
        name: 'Test Task',
        prompt: 'Test prompt',
        enabled: 0,
        timeout_seconds: 300,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      const task = convertTask(raw);

      expect(task.enabled).toBe(false);
    });

    it('should preserve all other fields', () => {
      const raw: RawTask = {
        id: 'task-123',
        name: 'My Task',
        prompt: 'My prompt',
        cron_expression: '0 * * * *',
        simple_schedule: undefined,
        enabled: 1,
        timeout_seconds: 600,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      };

      const task = convertTask(raw);

      expect(task.id).toBe('task-123');
      expect(task.name).toBe('My Task');
      expect(task.prompt).toBe('My prompt');
      expect(task.cron_expression).toBe('0 * * * *');
      expect(task.simple_schedule).toBeUndefined();
      expect(task.timeout_seconds).toBe(600);
      expect(task.created_at).toBe('2024-01-01T00:00:00Z');
      expect(task.updated_at).toBe('2024-01-02T00:00:00Z');
    });
  });
});

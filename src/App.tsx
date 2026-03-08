import { useState } from 'react';
import { TaskList } from './components/TaskList';
import { TaskForm, type TaskFormData } from './components/TaskForm';
import { ExecutionHistory } from './components/ExecutionHistory';
import type { Task } from './types/task';
import type { Execution } from './types/execution';
import './App.css';

type ViewMode = 'list' | 'form' | 'history';

// Mock data for development
const mockTasks: Task[] = [
  {
    id: '1',
    name: 'Daily Code Review',
    prompt: "Review today's code changes",
    simple_schedule: '{"type":"daily","hour":9}',
    enabled: true,
    timeout_seconds: 300,
    skip_if_running: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    name: 'Weekly Report',
    prompt: 'Generate weekly progress report',
    cron_expression: '0 17 * * 5',
    enabled: false,
    timeout_seconds: 600,
    skip_if_running: false,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
];

const mockExecutions: Execution[] = [
  {
    id: '1',
    task_id: '1',
    status: 'success',
    started_at: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    finished_at: new Date(Date.now() - 2 * 60 * 60 * 1000 + 5 * 60 * 1000).toISOString(),
    output_file: '/path/to/output1.txt',
  },
  {
    id: '2',
    task_id: '1',
    status: 'failed',
    started_at: new Date(Date.now() - 26 * 60 * 60 * 1000).toISOString(),
    finished_at: new Date(Date.now() - 26 * 60 * 60 * 1000 + 30 * 1000).toISOString(),
    error_message: 'Connection timeout',
  },
  {
    id: '3',
    task_id: '2',
    status: 'running',
    started_at: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
  },
];

function App() {
  const [tasks, setTasks] = useState<Task[]>(mockTasks);
  const [executions] = useState<Execution[]>(mockExecutions);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [editingTask, setEditingTask] = useState<Task | null>(null);

  const selectedTask = selectedTaskId ? tasks.find((t) => t.id === selectedTaskId) || null : null;

  const handleTaskSelect = (task: Task) => {
    setSelectedTaskId(task.id);
    setViewMode('list');
  };

  const handleCreateTask = () => {
    setEditingTask(null);
    setViewMode('form');
  };

  const handleEditTask = (task: Task) => {
    setEditingTask(task);
    setViewMode('form');
  };

  const handleViewHistory = (task: Task) => {
    setSelectedTaskId(task.id);
    setViewMode('history');
  };

  const handleToggleTask = (taskId: string, enabled: boolean) => {
    setTasks((prev) =>
      prev.map((task) =>
        task.id === taskId ? { ...task, enabled, updated_at: new Date().toISOString() } : task
      )
    );
  };

  const handleDeleteTask = (taskId: string) => {
    setTasks((prev) => prev.filter((task) => task.id !== taskId));
    if (selectedTaskId === taskId) {
      setSelectedTaskId(null);
    }
  };

  const handleSubmitTask = async (data: TaskFormData) => {
    await new Promise((resolve) => setTimeout(resolve, 500)); // Simulate API call

    if (editingTask) {
      setTasks((prev) =>
        prev.map((task) =>
          task.id === editingTask.id
            ? {
                ...task,
                name: data.name,
                prompt: data.prompt,
                cron_expression: data.cron_expression,
                simple_schedule: data.simple_schedule,
                timeout_seconds: data.timeout_seconds,
                skip_if_running: data.skip_if_running,
                updated_at: new Date().toISOString(),
              }
            : task
        )
      );
      setEditingTask(null);
    } else {
      const newTask: Task = {
        id: Date.now().toString(),
        name: data.name,
        prompt: data.prompt,
        cron_expression: data.cron_expression,
        simple_schedule: data.simple_schedule,
        enabled: true,
        timeout_seconds: data.timeout_seconds,
        skip_if_running: data.skip_if_running,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      setTasks((prev) => [...prev, newTask]);
    }

    setViewMode('list');
  };

  const handleCancelForm = () => {
    setEditingTask(null);
    setViewMode('list');
  };

  const handleViewOutput = (execution: Execution) => {
    console.log('View output for execution:', execution.id);
    // TODO: Implement output viewing
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>MyWork Scheduler</h1>
        <div className="header-actions">
          <button className="btn-primary" onClick={handleCreateTask}>
            + New Task
          </button>
        </div>
      </header>

      <main className="app-main">
        <aside className="app-sidebar">
          <div className="sidebar-header">
            <h2>Tasks</h2>
            <span className="task-count">{tasks.length}</span>
          </div>
          <div className="task-list-container">
            {tasks.map((task) => (
              <div
                key={task.id}
                className={`sidebar-task-item ${selectedTaskId === task.id ? 'selected' : ''}`}
                onClick={() => handleTaskSelect(task)}
              >
                <span
                  className={`task-status-indicator ${task.enabled ? 'enabled' : 'disabled'}`}
                />
                <span className="task-item-name">{task.name}</span>
              </div>
            ))}
          </div>
        </aside>

        <section className="app-content">
          {viewMode === 'form' && (
            <div className="content-panel">
              <div className="panel-header">
                <h2>{editingTask ? 'Edit Task' : 'Create New Task'}</h2>
              </div>
              <div className="panel-body">
                <TaskForm
                  initialData={editingTask || undefined}
                  onSubmit={handleSubmitTask}
                  onCancel={handleCancelForm}
                />
              </div>
            </div>
          )}

          {viewMode === 'list' && selectedTask && (
            <div className="content-panel">
              <div className="panel-header">
                <h2>{selectedTask.name}</h2>
                <div className="panel-actions">
                  <button className="btn-secondary" onClick={() => handleEditTask(selectedTask)}>
                    Edit
                  </button>
                  <button className="btn-secondary" onClick={() => handleViewHistory(selectedTask)}>
                    History
                  </button>
                </div>
              </div>
              <div className="panel-body">
                <TaskList
                  tasks={[selectedTask]}
                  onToggle={handleToggleTask}
                  onDelete={handleDeleteTask}
                />
              </div>
            </div>
          )}

          {viewMode === 'history' && selectedTask && (
            <div className="content-panel">
              <div className="panel-header">
                <h2>Execution History - {selectedTask.name}</h2>
                <div className="panel-actions">
                  <button className="btn-secondary" onClick={() => setViewMode('list')}>
                    Back to Task
                  </button>
                </div>
              </div>
              <div className="panel-body">
                <ExecutionHistory
                  executions={executions}
                  onViewOutput={handleViewOutput}
                  taskId={selectedTask.id}
                />
              </div>
            </div>
          )}

          {viewMode === 'list' && !selectedTask && (
            <div className="content-empty">
              <div className="empty-icon">📋</div>
              <h2>Select a Task</h2>
              <p>Choose a task from the sidebar to view details</p>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;

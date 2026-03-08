import { useState, useEffect } from 'react';
import { TaskList } from './components/TaskList';
import { TaskForm, type TaskFormData } from './components/TaskForm';
import { ExecutionHistory } from './components/ExecutionHistory';
import type { Task } from './types/task';
import type { Execution } from './types/execution';
import * as api from './api/tasks';
import './App.css';

type ViewMode = 'list' | 'form' | 'history';

function App() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [editingTask, setEditingTask] = useState<Task | null>(null);

  const selectedTask = selectedTaskId ? tasks.find((t) => t.id === selectedTaskId) || null : null;

  useEffect(() => {
    async function loadTasks() {
      try {
        const loadedTasks = await api.getTasks();
        setTasks(loadedTasks);
      } catch (error) {
        console.error('Failed to load tasks:', error);
      }
    }

    loadTasks();
  }, []);

  useEffect(() => {
    async function loadExecutions() {
      if (selectedTaskId) {
        try {
          const loadedExecutions = await api.getExecutions(selectedTaskId);
          setExecutions(loadedExecutions);
        } catch (error) {
          console.error('Failed to load executions:', error);
        }
      } else {
        setExecutions([]);
      }
    }

    loadExecutions();
  }, [selectedTaskId]);

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

  const handleToggleTask = async (taskId: string, enabled: boolean) => {
    try {
      await api.updateTask(taskId, { enabled: enabled ? 1 : 0 });
      setTasks((prev) =>
        prev.map((task) =>
          task.id === taskId ? { ...task, enabled, updated_at: new Date().toISOString() } : task
        )
      );
    } catch (error) {
      console.error('Failed to toggle task:', error);
    }
  };

  const handleDeleteTask = async (taskId: string) => {
    try {
      await api.deleteTask(taskId);
      setTasks((prev) => prev.filter((task) => task.id !== taskId));
      if (selectedTaskId === taskId) {
        setSelectedTaskId(null);
      }
    } catch (error) {
      console.error('Failed to delete task:', error);
    }
  };

  const handleSubmitTask = async (data: TaskFormData) => {
    try {
      if (editingTask) {
        const updatedTask = await api.updateTask(editingTask.id, {
          name: data.name,
          prompt: data.prompt,
          cron_expression: data.cron_expression,
          simple_schedule: data.simple_schedule,
          timeout_seconds: data.timeout_seconds,
          skip_if_running: data.skip_if_running ? 1 : 0,
        });
        setTasks((prev) => prev.map((task) => (task.id === editingTask.id ? updatedTask : task)));
        setEditingTask(null);
      } else {
        const newTask = await api.createTask({
          name: data.name,
          prompt: data.prompt,
          cron_expression: data.cron_expression,
          simple_schedule: data.simple_schedule,
          timeout_seconds: data.timeout_seconds,
          skip_if_running: data.skip_if_running ? 1 : 0,
          enabled: 1,
        });
        setTasks((prev) => [...prev, newTask]);
      }

      setViewMode('list');
    } catch (error) {
      console.error('Failed to submit task:', error);
    }
  };

  const handleCancelForm = () => {
    setEditingTask(null);
    setViewMode('list');
  };

  const handleViewOutput = (execution: Execution) => {
    console.log('View output for execution:', execution.id);
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

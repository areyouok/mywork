import { useState, useEffect, useRef } from 'react';
import { TaskList } from './components/TaskList';
import { TaskForm, type TaskFormData } from './components/TaskForm';
import { ExecutionHistory } from './components/ExecutionHistory';
import { OutputViewer } from './components/OutputViewer';
import { useTasks } from './hooks/useTasks';
import { useScheduler } from './hooks/useScheduler';
import { useExecutions } from './hooks/useExecutions';
import { useTaskActions } from './hooks/useTaskActions';
import { useOutput } from './hooks/useOutput';
import type { Task } from './types/task';
import type { Execution } from './types/execution';
import './App.css';

type ViewMode = 'list' | 'form' | 'history' | 'output';

function App() {
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [selectedExecution, setSelectedExecution] = useState<Execution | null>(null);

  const selectedTaskIdRef = useRef<string | null>(selectedTaskId);
  useEffect(() => {
    selectedTaskIdRef.current = selectedTaskId;
  }, [selectedTaskId]);

  const { tasks, loadTasks, createTask, updateTask, deleteTask } = useTasks();
  const {
    status: schedulerStatus,
    addRunningTask,
    removeRunningTask,
    runningTaskIds,
  } = useScheduler();
  const { executions, loadExecutions } = useExecutions();

  useEffect(() => {
    const tauriInternals = Reflect.get(window, '__TAURI_INTERNALS__');
    if (!tauriInternals) {
      return;
    }

    let unlistenStarted: (() => void) | undefined;
    let unlistenFinished: (() => void) | undefined;
    let mounted = true;

    import('@tauri-apps/api/event').then(({ listen }) => {
      if (!mounted) {
        return;
      }

      void listen<string>('execution-started', () => loadExecutions(selectedTaskId)).then(
        (unlisten) => {
          unlistenStarted = unlisten;
        }
      );

      void listen<string>('execution-finished', () => loadExecutions(selectedTaskId)).then(
        (unlisten) => {
          unlistenFinished = unlisten;
        }
      );
    });

    return () => {
      mounted = false;
      unlistenStarted?.();
      unlistenFinished?.();
    };
  }, [loadExecutions, selectedTaskId]);

  const { handleToggle, handleDelete, handleRun } = useTaskActions(
    updateTask,
    deleteTask,
    addRunningTask,
    removeRunningTask,
    loadExecutions,
    selectedTaskIdRef,
    loadTasks
  );
  const { outputContent, selectedExecutionId, loadOutput } = useOutput();

  const selectedTask = selectedTaskId ? tasks.find((t) => t.id === selectedTaskId) || null : null;

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  useEffect(() => {
    loadExecutions(selectedTaskId);
  }, [selectedTaskId, viewMode, loadExecutions]);

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

  const handleSubmitTask = async (data: TaskFormData) => {
    try {
      if (editingTask) {
        await updateTask(editingTask.id, {
          name: data.name,
          prompt: data.prompt,
          cron_expression: data.cron_expression,
          simple_schedule: data.simple_schedule,
          timeout_seconds: data.timeout_seconds,
        });
        setEditingTask(null);
      } else {
        await createTask({
          name: data.name,
          prompt: data.prompt,
          cron_expression: data.cron_expression,
          simple_schedule: data.simple_schedule,
          timeout_seconds: data.timeout_seconds,
          enabled: 1,
        });
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

  const handleViewOutput = async (execution: Execution | string) => {
    const exec =
      typeof execution === 'string'
        ? ({ id: execution, task_id: '', status: 'pending', started_at: '' } as Execution)
        : execution;
    setSelectedExecution(exec);
    await loadOutput(execution);
    setViewMode('output');
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>MyWork Scheduler</h1>
        <div className="header-actions">
          <span className={`scheduler-status status-${schedulerStatus}`}>
            Scheduler: {schedulerStatus}
          </span>
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
              </div>
              <div className="panel-body">
                <TaskList
                  tasks={[selectedTask]}
                  runningTaskIds={runningTaskIds}
                  onToggle={handleToggle}
                  onDelete={handleDelete}
                  onRun={handleRun}
                  onEdit={handleEditTask}
                  onHistory={handleViewHistory}
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

          {viewMode === 'output' && selectedTask && (
            <div className="content-panel">
              <div className="panel-header">
                <h2>Output - Execution {selectedExecutionId}</h2>
                <div className="panel-actions">
                  <button className="btn-secondary" onClick={() => setViewMode('history')}>
                    Back to History
                  </button>
                </div>
              </div>
              <div className="panel-body">
                <OutputViewer
                  content={outputContent}
                  isMarkdown={true}
                  execution={selectedExecution}
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

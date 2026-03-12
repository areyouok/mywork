use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Task queue error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskQueueError {
    /// Task is already running (skip execution)
    TaskAlreadyRunning { task_id: String },
    /// Task not found in running tasks
    TaskNotFound { task_id: String },
}

impl std::fmt::Display for TaskQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskQueueError::TaskAlreadyRunning { task_id } => {
                write!(f, "Task '{}' is already running", task_id)
            }
            TaskQueueError::TaskNotFound { task_id } => {
                write!(f, "Task '{}' not found in running tasks", task_id)
            }
        }
    }
}

impl std::error::Error for TaskQueueError {}

/// Guard that automatically releases slot when dropped
pub struct SlotGuard {
    task_id: String,
    running_tasks: Arc<Mutex<HashMap<String, ()>>>,
}

impl std::fmt::Debug for SlotGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlotGuard")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl SlotGuard {
    /// Get the task ID for this guard
    pub fn task_id(&self) -> &str {
        &self.task_id
    }
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        let mut running = self
            .running_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        running.remove(&self.task_id);
    }
}

/// Task queue with per-task concurrency control
/// Each task can only have one running instance at a time
pub struct TaskQueue {
    /// Map of task_id to their running status
    running_tasks: Arc<Mutex<HashMap<String, ()>>>,
}

impl TaskQueue {
    /// Create a new task queue
    ///
    /// # Example
    /// ```
    /// use mywork_lib::scheduler::task_queue::TaskQueue;
    ///
    /// let queue = TaskQueue::new();
    /// ```
    pub fn new() -> Self {
        Self {
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current number of running tasks
    pub async fn running_count(&self) -> usize {
        self.running_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    /// Check if a task is currently running
    ///
    /// # Arguments
    /// * `task_id` - The task ID to check
    ///
    /// # Returns
    /// `true` if the task is currently running, `false` otherwise
    pub async fn is_running(&self, task_id: &str) -> bool {
        self.running_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(task_id)
    }

    /// Try to acquire a slot for task execution
    /// Returns Ok if task is not already running, Err otherwise
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier
    ///
    /// # Returns
    /// * `Ok(SlotGuard)` - Successfully acquired slot, guard will release on drop
    /// * `Err(TaskQueueError::TaskAlreadyRunning)` - Task is already running
    pub async fn acquire_slot(&self, task_id: &str) -> Result<SlotGuard, TaskQueueError> {
        let mut running = self
            .running_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        
        if running.contains_key(task_id) {
            return Err(TaskQueueError::TaskAlreadyRunning {
                task_id: task_id.to_string(),
            });
        }
        
        running.insert(task_id.to_string(), ());

        Ok(SlotGuard {
            task_id: task_id.to_string(),
            running_tasks: self.running_tasks.clone(),
        })
    }

    /// Manually release a slot for a task
    /// Note: Slots are automatically released when SlotGuard is dropped
    ///
    /// # Arguments
    /// * `task_id` - The task ID to release
    ///
    /// # Returns
    /// * `Ok(())` - Successfully released
    /// * `Err(TaskQueueError::TaskNotFound)` - Task was not running
    pub async fn release_slot(&self, task_id: &str) -> Result<(), TaskQueueError> {
        let mut running = self
            .running_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if running.remove(task_id).is_some() {
            Ok(())
        } else {
            Err(TaskQueueError::TaskNotFound {
                task_id: task_id.to_string(),
            })
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_task_queue_creation() {
        let queue = TaskQueue::new();
        assert_eq!(queue.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_task_queue_default() {
        let queue = TaskQueue::default();
        assert_eq!(queue.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_acquire_slot_success() {
        let queue = TaskQueue::new();

        let guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");

        assert_eq!(queue.running_count().await, 1);
        assert!(queue.is_running("task-1").await);
        assert_eq!(guard.task_id(), "task-1");
    }

    #[tokio::test]
    async fn test_acquire_slot_multiple_tasks() {
        let queue = TaskQueue::new();

        let _guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");
        let _guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");

        assert_eq!(queue.running_count().await, 3);
        assert!(queue.is_running("task-1").await);
        assert!(queue.is_running("task-2").await);
        assert!(queue.is_running("task-3").await);
    }

    #[tokio::test]
    async fn test_acquire_slot_already_running() {
        let queue = TaskQueue::new();

        let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
        let result = queue.acquire_slot("task-1").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskQueueError::TaskAlreadyRunning { .. }));
        assert_eq!(queue.running_count().await, 1);
    }

    #[tokio::test]
    async fn test_release_slot_success() {
        let queue = TaskQueue::new();

        let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
        assert_eq!(queue.running_count().await, 1);

        queue.release_slot("task-1").await.expect("Failed to release slot");

        assert_eq!(queue.running_count().await, 0);
        assert!(!queue.is_running("task-1").await);
    }

    #[tokio::test]
    async fn test_release_slot_not_found() {
        let queue = TaskQueue::new();

        let result = queue.release_slot("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskQueueError::TaskNotFound { .. }));
    }

    #[tokio::test]
    async fn test_slot_guard_auto_release() {
        let queue = TaskQueue::new();

        {
            let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
            assert_eq!(queue.running_count().await, 1);
        }

        assert_eq!(queue.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_concurrent_tasks_allowed() {
        let queue = std::sync::Arc::new(TaskQueue::new());

        // Multiple different tasks can run concurrently
        let _guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");
        let _guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");

        assert_eq!(queue.running_count().await, 3);
        
        drop(_guard1);
        drop(_guard2);
        drop(_guard3);
        sleep(Duration::from_millis(10)).await;
        
        assert_eq!(queue.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_release_and_reacquire() {
        let queue = TaskQueue::new();

        let guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");

        assert_eq!(queue.running_count().await, 2);

        drop(guard1);

        assert!(!queue.is_running("task-1").await);

        let _guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");
        assert!(queue.is_running("task-3").await);
    }

    #[tokio::test]
    async fn test_error_display() {
        let err = TaskQueueError::TaskAlreadyRunning {
            task_id: "test-task".to_string(),
        };
        assert!(err.to_string().contains("already running"));

        let err = TaskQueueError::TaskNotFound {
            task_id: "test-task".to_string(),
        };
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let queue = TaskQueue::new();

        let guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");
        let guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");

        assert_eq!(queue.running_count().await, 3);

        let result = queue.acquire_slot("task-1").await;
        assert!(matches!(result.unwrap_err(), TaskQueueError::TaskAlreadyRunning { .. }));

        drop(guard1);

        assert_eq!(queue.running_count().await, 2);

        let guard4 = queue.acquire_slot("task-4").await.expect("Failed to acquire slot 4");
        assert!(queue.is_running("task-4").await);

        drop(guard2);
        drop(guard3);
        drop(guard4);

        assert_eq!(queue.running_count().await, 0);
    }
}

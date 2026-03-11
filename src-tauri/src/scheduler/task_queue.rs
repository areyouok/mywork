use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

/// Task queue error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskQueueError {
    /// No available slots (queue is full)
    NoAvailableSlots { task_id: String, max_concurrent: usize },
    /// Task is already running (for skip_if_running)
    TaskAlreadyRunning { task_id: String },
    /// Task not found in running tasks
    TaskNotFound { task_id: String },
}

impl std::fmt::Display for TaskQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskQueueError::NoAvailableSlots { task_id, max_concurrent } => {
                write!(
                    f,
                    "No available slots for task '{}' (max concurrent: {})",
                    task_id, max_concurrent
                )
            }
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
    running_tasks: Arc<Mutex<HashMap<String, OwnedSemaphorePermit>>>,
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
        let running_tasks = self.running_tasks.clone();
        let task_id = self.task_id.clone();
        
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            std::mem::drop(handle.spawn(async move {
                running_tasks.lock().await.remove(&task_id);
            }));
        }
    }
}

/// Task queue with concurrency control using semaphore
pub struct TaskQueue {
    /// Semaphore for limiting concurrent executions
    semaphore: Arc<Semaphore>,
    /// Map of task_id to their permits (running tasks)
    running_tasks: Arc<Mutex<HashMap<String, OwnedSemaphorePermit>>>,
    /// Maximum number of concurrent tasks
    max_concurrent: usize,
}

impl TaskQueue {
    /// Create a new task queue with specified maximum concurrent tasks
    ///
    /// # Arguments
    /// * `max_concurrent` - Maximum number of tasks that can run concurrently
    ///
    /// # Example
    /// ```
    /// use mywork_lib::scheduler::task_queue::TaskQueue;
    ///
    /// let queue = TaskQueue::new(3);
    /// assert_eq!(queue.max_concurrent(), 3);
    /// ```
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Get the maximum number of concurrent tasks
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Get the current number of running tasks
    pub async fn running_count(&self) -> usize {
        self.running_tasks.lock().await.len()
    }

    /// Check if a task is currently running
    ///
    /// # Arguments
    /// * `task_id` - The task ID to check
    ///
    /// # Returns
    /// `true` if the task is currently running, `false` otherwise
    pub async fn is_running(&self, task_id: &str) -> bool {
        self.running_tasks.lock().await.contains_key(task_id)
    }

    /// Try to acquire a slot for task execution (non-blocking)
    /// Returns immediately if slot is available or task is already running
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier
    ///
    /// # Returns
    /// * `Ok(SlotGuard)` - Successfully acquired slot, guard will release on drop
    /// * `Err(TaskQueueError::TaskAlreadyRunning)` - Task is already running
    /// * `Err(TaskQueueError::NoAvailableSlots)` - Queue is full
    pub async fn acquire_slot(&self, task_id: &str) -> Result<SlotGuard, TaskQueueError> {
        {
            let running = self.running_tasks.lock().await;
            if running.contains_key(task_id) {
                return Err(TaskQueueError::TaskAlreadyRunning {
                    task_id: task_id.to_string(),
                });
            }
        }

        let permit = self
            .semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| TaskQueueError::NoAvailableSlots {
                task_id: task_id.to_string(),
                max_concurrent: self.max_concurrent,
            })?;

        {
            let mut running = self.running_tasks.lock().await;
            running.insert(task_id.to_string(), permit);
        }

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
        let mut running = self.running_tasks.lock().await;
        if running.remove(task_id).is_some() {
            Ok(())
        } else {
            Err(TaskQueueError::TaskNotFound {
                task_id: task_id.to_string(),
            })
        }
    }

    /// Get the number of available slots
    pub fn available_slots(&self) -> usize {
        self.semaphore.available_permits()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_task_queue_creation() {
        let queue = TaskQueue::new(3);
        assert_eq!(queue.max_concurrent(), 3);
        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.available_slots(), 3);
    }

    #[tokio::test]
    async fn test_task_queue_default() {
        let queue = TaskQueue::default();
        assert_eq!(queue.max_concurrent(), 5);
    }

    #[tokio::test]
    async fn test_acquire_slot_success() {
        let queue = TaskQueue::new(3);

        let guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");

        assert_eq!(queue.running_count().await, 1);
        assert!(queue.is_running("task-1").await);
        assert_eq!(queue.available_slots(), 2);
        assert_eq!(guard.task_id(), "task-1");
    }

    #[tokio::test]
    async fn test_acquire_slot_multiple_tasks() {
        let queue = TaskQueue::new(3);

        let _guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");
        let _guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");

        assert_eq!(queue.running_count().await, 3);
        assert_eq!(queue.available_slots(), 0);
        assert!(queue.is_running("task-1").await);
        assert!(queue.is_running("task-2").await);
        assert!(queue.is_running("task-3").await);
    }

    #[tokio::test]
    async fn test_acquire_slot_queue_full() {
        let queue = TaskQueue::new(2);

        let _guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");

        let result = queue.acquire_slot("task-3").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskQueueError::NoAvailableSlots { .. }));
        assert_eq!(queue.running_count().await, 2);
    }

    #[tokio::test]
    async fn test_acquire_slot_already_running() {
        let queue = TaskQueue::new(3);

        let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
        let result = queue.acquire_slot("task-1").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskQueueError::TaskAlreadyRunning { .. }));
        assert_eq!(queue.running_count().await, 1);
    }

    #[tokio::test]
    async fn test_release_slot_success() {
        let queue = TaskQueue::new(3);

        let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
        assert_eq!(queue.running_count().await, 1);

        queue.release_slot("task-1").await.expect("Failed to release slot");

        assert_eq!(queue.running_count().await, 0);
        assert!(!queue.is_running("task-1").await);
        assert_eq!(queue.available_slots(), 3);
    }

    #[tokio::test]
    async fn test_release_slot_not_found() {
        let queue = TaskQueue::new(3);

        let result = queue.release_slot("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskQueueError::TaskNotFound { .. }));
    }

    #[tokio::test]
    async fn test_slot_guard_auto_release() {
        let queue = TaskQueue::new(3);

        {
            let _guard = queue.acquire_slot("task-1").await.expect("Failed to acquire slot");
            assert_eq!(queue.running_count().await, 1);
        }

        sleep(Duration::from_millis(10)).await;
        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.available_slots(), 3);
    }

    #[tokio::test]
    async fn test_concurrent_execution_limit() {
        let queue = Arc::new(TaskQueue::new(2));
        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let current_concurrent = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for i in 0..5 {
            let queue = queue.clone();
            let counter = counter.clone();
            let max_concurrent = max_concurrent.clone();
            let current_concurrent = current_concurrent.clone();

            let handle = tokio::spawn(async move {
                if let Ok(_guard) = queue.acquire_slot(&format!("task-{}", i)).await {
                    let current = current_concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    loop {
                        let max = max_concurrent.load(Ordering::SeqCst);
                        if current <= max || max_concurrent.compare_exchange(max, current, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                            break;
                        }
                    }

                    counter.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(50)).await;
                    current_concurrent.fetch_sub(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        sleep(Duration::from_millis(10)).await;

        let running = queue.running_count().await;
        assert!(running <= 2, "Expected at most 2 running, got {}", running);

        for handle in handles {
            let _ = handle.await;
        }

        sleep(Duration::from_millis(20)).await;

        let max = max_concurrent.load(Ordering::SeqCst);
        assert!(max <= 2, "Max concurrent {} exceeded limit of 2", max);

        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.available_slots(), 2);
    }

    #[tokio::test]
    async fn test_release_and_reacquire() {
        let queue = TaskQueue::new(2);

        let guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let _guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");

        assert_eq!(queue.available_slots(), 0);

        drop(guard1);
        sleep(Duration::from_millis(10)).await;

        assert_eq!(queue.available_slots(), 1);
        assert!(!queue.is_running("task-1").await);

        let _guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");
        assert!(queue.is_running("task-3").await);
    }

    #[tokio::test]
    async fn test_error_display() {
        let err = TaskQueueError::NoAvailableSlots {
            task_id: "test-task".to_string(),
            max_concurrent: 3,
        };
        assert!(err.to_string().contains("test-task"));
        assert!(err.to_string().contains("3"));

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
        let queue = TaskQueue::new(3);

        let guard1 = queue.acquire_slot("task-1").await.expect("Failed to acquire slot 1");
        let guard2 = queue.acquire_slot("task-2").await.expect("Failed to acquire slot 2");
        let guard3 = queue.acquire_slot("task-3").await.expect("Failed to acquire slot 3");

        assert_eq!(queue.running_count().await, 3);
        assert_eq!(queue.available_slots(), 0);

        let result = queue.acquire_slot("task-4").await;
        assert!(result.is_err());

        let result = queue.acquire_slot("task-1").await;
        assert!(matches!(result.unwrap_err(), TaskQueueError::TaskAlreadyRunning { .. }));

        drop(guard1);
        sleep(Duration::from_millis(10)).await;

        assert_eq!(queue.running_count().await, 2);
        assert_eq!(queue.available_slots(), 1);

        let guard4 = queue.acquire_slot("task-4").await.expect("Failed to acquire slot 4");
        assert!(queue.is_running("task-4").await);

        drop(guard2);
        drop(guard3);
        drop(guard4);
        sleep(Duration::from_millis(10)).await;

        assert_eq!(queue.running_count().await, 0);
        assert_eq!(queue.available_slots(), 3);
    }
}

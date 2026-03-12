use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

/// Scheduler error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchedulerError {
    /// Failed to create scheduler
    SchedulerCreationFailed { message: String },
    /// Failed to add job
    JobAddFailed { task_id: String, message: String },
    /// Failed to remove job
    JobRemoveFailed { task_id: String, message: String },
    /// Failed to start scheduler
    StartFailed { message: String },
    /// Failed to stop scheduler
    StopFailed { message: String },
    /// Job not found
    JobNotFound { task_id: String },
    /// Invalid cron expression
    InvalidCronExpression { expression: String, message: String },
    /// Scheduler already running
    AlreadyRunning,
    /// Scheduler not running
    NotRunning,
}

impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerError::SchedulerCreationFailed { message } => {
                write!(f, "Failed to create scheduler: {}", message)
            }
            SchedulerError::JobAddFailed { task_id, message } => {
                write!(f, "Failed to add job for task {}: {}", task_id, message)
            }
            SchedulerError::JobRemoveFailed { task_id, message } => {
                write!(f, "Failed to remove job for task {}: {}", task_id, message)
            }
            SchedulerError::StartFailed { message } => {
                write!(f, "Failed to start scheduler: {}", message)
            }
            SchedulerError::StopFailed { message } => {
                write!(f, "Failed to stop scheduler: {}", message)
            }
            SchedulerError::JobNotFound { task_id } => {
                write!(f, "Job not found for task {}", task_id)
            }
            SchedulerError::InvalidCronExpression { expression, message } => {
                write!(
                    f,
                    "Invalid cron expression '{}': {}",
                    expression, message
                )
            }
            SchedulerError::AlreadyRunning => {
                write!(f, "Scheduler is already running")
            }
            SchedulerError::NotRunning => {
                write!(f, "Scheduler is not running")
            }
        }
    }
}

impl std::error::Error for SchedulerError {}

/// Job callback type - async function that executes when job triggers
pub type JobCallback = Arc<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

/// Job information stored in the scheduler
#[derive(Debug, Clone)]
pub struct JobInfo {
    /// Task ID from database
    pub task_id: String,
    /// Job UUID from scheduler
    pub job_id: Uuid,
    /// Cron expression
    pub cron_expression: String,
}

/// Scheduler state
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerState {
    /// Scheduler is stopped
    Stopped,
    /// Scheduler is running
    Running,
}

/// Scheduler structure managing all scheduled jobs
pub struct Scheduler {
    /// Tokio cron scheduler instance
    scheduler: Arc<Mutex<Option<JobScheduler>>>,
    /// Map of task_id to job info
    jobs: Arc<Mutex<HashMap<String, JobInfo>>>,
    /// Scheduler state
    state: Arc<Mutex<SchedulerState>>,
}

impl Scheduler {
    /// Create a new scheduler instance
    pub fn new() -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(None)),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(SchedulerState::Stopped)),
        }
    }

    /// Add a new job to the scheduler
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier from database
    /// * `cron_expression` - 5-field cron expression (will be converted to 6-field)
    /// * `callback` - Async callback function to execute when job triggers
    ///
    /// # Returns
    /// * `Ok(Uuid)` - The job UUID if successfully added
    /// * `Err(SchedulerError)` - If job creation or addition fails
    ///
    /// # Example
    /// ```no_run
    /// use mywork_lib::scheduler::job_scheduler::{Scheduler, SchedulerError};
    /// use mywork_lib::scheduler::JobCallback;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), SchedulerError> {
    ///     let scheduler = Scheduler::new();
    ///     let callback: JobCallback = Arc::new(|| {
    ///         Box::pin(async { println!("Job executed!"); }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    ///     });
    ///     let job_id = scheduler.add_job("task-1", "*/5 * * * *", callback).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_job(
        &self,
        task_id: &str,
        cron_expression: &str,
        callback: JobCallback,
    ) -> Result<Uuid, SchedulerError> {
        super::validate_cron(cron_expression).map_err(|e| SchedulerError::InvalidCronExpression {
            expression: cron_expression.to_string(),
            message: e.to_string(),
        })?;

        let six_field_cron = format!("0 {}", cron_expression);

        let job = Job::new_async(six_field_cron.as_str(), move |_uuid, _l| {
            let cb = callback.clone();
            Box::pin(async move {
                cb().await;
            })
        })
        .map_err(|e| SchedulerError::JobAddFailed {
            task_id: task_id.to_string(),
            message: e.to_string(),
        })?;

        let job_id = job.guid();

        let should_start_scheduler = {
            let state = self.state.lock().await;
            *state == SchedulerState::Running
        };

        let mut scheduler_guard = self.scheduler.lock().await;
        let mut created_scheduler = false;
        if scheduler_guard.is_none() {
            let new_scheduler = JobScheduler::new()
                .await
                .map_err(|e| SchedulerError::SchedulerCreationFailed {
                    message: e.to_string(),
                })?;
            *scheduler_guard = Some(new_scheduler);
            created_scheduler = true;
        }

        if created_scheduler && should_start_scheduler {
            if let Some(sched) = scheduler_guard.as_ref() {
                sched
                    .start()
                    .await
                    .map_err(|e| SchedulerError::StartFailed {
                        message: e.to_string(),
                    })?;
            }
        }

        if let Some(sched) = scheduler_guard.as_ref() {
            sched
                .add(job)
                .await
                .map_err(|e| SchedulerError::JobAddFailed {
                    task_id: task_id.to_string(),
                    message: e.to_string(),
                })?;
        }

        let job_info = JobInfo {
            task_id: task_id.to_string(),
            job_id,
            cron_expression: cron_expression.to_string(),
        };

        let mut jobs = self.jobs.lock().await;
        jobs.insert(task_id.to_string(), job_info);

        Ok(job_id)
    }

    pub async fn add_one_shot_job(
        &self,
        task_id: &str,
        duration: Duration,
        callback: JobCallback,
    ) -> Result<Uuid, SchedulerError> {
        let jobs_map = self.jobs.clone();
        let task_id_for_cleanup = task_id.to_string();
        let job = Job::new_one_shot_async(duration, move |executed_job_id, _l| {
            let cb = callback.clone();
            let jobs = jobs_map.clone();
            let cleanup_task_id = task_id_for_cleanup.clone();
            Box::pin(async move {
                cb().await;
                let mut jobs_guard = jobs.lock().await;
                let should_cleanup = jobs_guard
                    .get(&cleanup_task_id)
                    .map(|job_info| job_info.job_id == executed_job_id)
                    .unwrap_or(false);

                if should_cleanup {
                    jobs_guard.remove(&cleanup_task_id);
                }
            })
        })
        .map_err(|e| SchedulerError::JobAddFailed {
            task_id: task_id.to_string(),
            message: e.to_string(),
        })?;

        let job_id = job.guid();

        let should_start_scheduler = {
            let state = self.state.lock().await;
            *state == SchedulerState::Running
        };

        let mut scheduler_guard = self.scheduler.lock().await;
        let mut created_scheduler = false;
        if scheduler_guard.is_none() {
            let new_scheduler = JobScheduler::new()
                .await
                .map_err(|e| SchedulerError::SchedulerCreationFailed {
                    message: e.to_string(),
                })?;
            *scheduler_guard = Some(new_scheduler);
            created_scheduler = true;
        }

        if created_scheduler && should_start_scheduler {
            if let Some(sched) = scheduler_guard.as_ref() {
                sched
                    .start()
                    .await
                    .map_err(|e| SchedulerError::StartFailed {
                        message: e.to_string(),
                    })?;
            }
        }

        if let Some(sched) = scheduler_guard.as_ref() {
            sched
                .add(job)
                .await
                .map_err(|e| SchedulerError::JobAddFailed {
                    task_id: task_id.to_string(),
                    message: e.to_string(),
                })?;
        }

        let job_info = JobInfo {
            task_id: task_id.to_string(),
            job_id,
            cron_expression: format!("@once+{}s", duration.as_secs()),
        };

        let mut jobs = self.jobs.lock().await;
        jobs.insert(task_id.to_string(), job_info);

        Ok(job_id)
    }

    /// Remove a job from the scheduler
    ///
    /// # Arguments
    /// * `task_id` - Task identifier of the job to remove
    ///
    /// # Returns
    /// * `Ok(())` - If job was successfully removed
    /// * `Err(SchedulerError)` - If job removal fails or job not found
    pub async fn remove_job(&self, task_id: &str) -> Result<(), SchedulerError> {
        let job_info = {
            let jobs = self.jobs.lock().await;
            jobs.get(task_id)
                .ok_or_else(|| SchedulerError::JobNotFound {
                    task_id: task_id.to_string(),
                })?
                .clone()
        };

        let scheduler_guard = self.scheduler.lock().await;
        if let Some(sched) = scheduler_guard.as_ref() {
            sched
                .remove(&job_info.job_id)
                .await
                .map_err(|e| SchedulerError::JobRemoveFailed {
                    task_id: task_id.to_string(),
                    message: e.to_string(),
                })?;
        }

        let mut jobs = self.jobs.lock().await;
        jobs.remove(task_id);

        Ok(())
    }

    /// Start the scheduler
    ///
    /// # Returns
    /// * `Ok(())` - If scheduler started successfully
    /// * `Err(SchedulerError)` - If start fails or scheduler already running
    pub async fn start(&self) -> Result<(), SchedulerError> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Running {
            return Err(SchedulerError::AlreadyRunning);
        }

        let mut scheduler_guard = self.scheduler.lock().await;
        if scheduler_guard.is_none() {
            let new_scheduler = JobScheduler::new()
                .await
                .map_err(|e| SchedulerError::SchedulerCreationFailed {
                    message: e.to_string(),
                })?;
            *scheduler_guard = Some(new_scheduler);
        }

        if let Some(sched) = scheduler_guard.as_ref() {
            sched
                .start()
                .await
                .map_err(|e| SchedulerError::StartFailed {
                    message: e.to_string(),
                })?;
        }

        *state = SchedulerState::Running;
        Ok(())
    }

    /// Stop the scheduler
    ///
    /// # Returns
    /// * `Ok(())` - If scheduler stopped successfully
    /// * `Err(SchedulerError)` - If stop fails or scheduler not running
    pub async fn stop(&self) -> Result<(), SchedulerError> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Stopped {
            return Err(SchedulerError::NotRunning);
        }

        let mut scheduler_guard = self.scheduler.lock().await;
        if let Some(sched) = scheduler_guard.as_mut() {
            sched
                .shutdown()
                .await
                .map_err(|e| SchedulerError::StopFailed {
                    message: e.to_string(),
                })?;
        }

        *state = SchedulerState::Stopped;
        Ok(())
    }

    /// Get scheduler state
    pub async fn get_state(&self) -> SchedulerState {
        self.state.lock().await.clone()
    }

    /// Get job count
    pub async fn job_count(&self) -> usize {
        self.jobs.lock().await.len()
    }

    /// Check if job exists for a task
    pub async fn has_job(&self, task_id: &str) -> bool {
        self.jobs.lock().await.contains_key(task_id)
    }

    /// Get job info for a task
    pub async fn get_job_info(&self, task_id: &str) -> Option<JobInfo> {
        self.jobs.lock().await.get(task_id).cloned()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);
        assert_eq!(scheduler.job_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_job_success() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let callback: JobCallback = Arc::new(move || {
            let c = counter_clone.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        let job_id = scheduler
            .add_job("task-1", "*/5 * * * *", callback)
            .await
            .expect("Failed to add job");

        assert!(!job_id.is_nil());
        assert_eq!(scheduler.job_count().await, 1);
        assert!(scheduler.has_job("task-1").await);
    }

    #[tokio::test]
    async fn test_add_job_invalid_cron() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        let result = scheduler.add_job("task-1", "invalid", callback).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchedulerError::InvalidCronExpression { .. }
        ));
    }

    #[tokio::test]
    async fn test_remove_job_success() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback.clone())
            .await
            .expect("Failed to add job");

        assert_eq!(scheduler.job_count().await, 1);

        scheduler
            .remove_job("task-1")
            .await
            .expect("Failed to remove job");

        assert_eq!(scheduler.job_count().await, 0);
        assert!(!scheduler.has_job("task-1").await);
    }

    #[tokio::test]
    async fn test_remove_job_not_found() {
        let scheduler = Scheduler::new();

        let result = scheduler.remove_job("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchedulerError::JobNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_start_scheduler() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback)
            .await
            .expect("Failed to add job");

        scheduler.start().await.expect("Failed to start scheduler");

        assert_eq!(scheduler.get_state().await, SchedulerState::Running);
    }

    #[tokio::test]
    async fn test_start_already_running() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback)
            .await
            .expect("Failed to add job");

        scheduler.start().await.expect("Failed to start scheduler");

        let result = scheduler.start().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SchedulerError::AlreadyRunning));
    }

    #[tokio::test]
    async fn test_stop_scheduler() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback)
            .await
            .expect("Failed to add job");

        scheduler.start().await.expect("Failed to start scheduler");
        scheduler.stop().await.expect("Failed to stop scheduler");

        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);
    }

    #[tokio::test]
    async fn test_stop_not_running() {
        let scheduler = Scheduler::new();

        let result = scheduler.stop().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SchedulerError::NotRunning));
    }

    #[tokio::test]
    async fn test_job_execution() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let callback: JobCallback = Arc::new(move || {
            let c = counter_clone.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        scheduler
            .add_job("task-1", "* * * * *", callback)
            .await
            .expect("Failed to add job");

        scheduler.start().await.expect("Failed to start scheduler");

        sleep(Duration::from_millis(100)).await;

        scheduler.stop().await.expect("Failed to stop scheduler");

        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);
    }

    #[tokio::test]
    async fn test_multiple_jobs() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback.clone())
            .await
            .expect("Failed to add job 1");

        scheduler
            .add_job("task-2", "*/10 * * * *", callback.clone())
            .await
            .expect("Failed to add job 2");

        scheduler
            .add_job("task-3", "*/15 * * * *", callback)
            .await
            .expect("Failed to add job 3");

        assert_eq!(scheduler.job_count().await, 3);
        assert!(scheduler.has_job("task-1").await);
        assert!(scheduler.has_job("task-2").await);
        assert!(scheduler.has_job("task-3").await);
    }

    #[tokio::test]
    async fn test_get_job_info() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        let job_id = scheduler
            .add_job("task-1", "*/5 * * * *", callback)
            .await
            .expect("Failed to add job");

        let job_info = scheduler
            .get_job_info("task-1")
            .await
            .expect("Job info not found");

        assert_eq!(job_info.task_id, "task-1");
        assert_eq!(job_info.job_id, job_id);
        assert_eq!(job_info.cron_expression, "*/5 * * * *");
    }

    #[tokio::test]
    async fn test_scheduler_error_display() {
        let err = SchedulerError::JobNotFound {
            task_id: "test-task".to_string(),
        };
        assert!(err.to_string().contains("test-task"));

        let err = SchedulerError::InvalidCronExpression {
            expression: "* * *".to_string(),
            message: "Invalid field count".to_string(),
        };
        assert!(err.to_string().contains("* * *"));
    }

    #[tokio::test]
    async fn test_add_job_after_start() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback.clone())
            .await
            .expect("Failed to add job 1");

        scheduler.start().await.expect("Failed to start scheduler");

        scheduler
            .add_job("task-2", "*/10 * * * *", callback)
            .await
            .expect("Failed to add job 2");

        assert_eq!(scheduler.job_count().await, 2);
    }

    #[tokio::test]
    async fn test_start_initializes_scheduler_without_jobs() {
        let scheduler = Scheduler::new();

        scheduler.start().await.expect("Failed to start scheduler");

        assert_eq!(scheduler.get_state().await, SchedulerState::Running);
        let scheduler_guard = scheduler.scheduler.lock().await;
        assert!(scheduler_guard.is_some(), "scheduler instance should be initialized on start");
    }

    #[tokio::test]
    async fn test_add_one_shot_job_after_start_without_existing_scheduler_executes() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        scheduler
            .start()
            .await
            .expect("Failed to start scheduler");

        let callback: JobCallback = Arc::new(move || {
            let c = counter_clone.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        scheduler
            .add_one_shot_job("task-oneshot", Duration::from_secs(1), callback)
            .await
            .expect("Failed to add one-shot job");

        sleep(Duration::from_secs(2)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_replace_one_shot_with_cron_reuses_task_id_safely() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_one_shot_job("task-switch", Duration::from_secs(30), callback.clone())
            .await
            .expect("Failed to add one-shot job");

        let one_shot_info = scheduler
            .get_job_info("task-switch")
            .await
            .expect("Expected one-shot job info");
        assert!(one_shot_info.cron_expression.starts_with("@once+"));

        scheduler
            .remove_job("task-switch")
            .await
            .expect("Failed to remove one-shot job");
        assert!(!scheduler.has_job("task-switch").await);

        scheduler
            .add_job("task-switch", "0 * * * *", callback)
            .await
            .expect("Failed to add cron job");

        let cron_info = scheduler
            .get_job_info("task-switch")
            .await
            .expect("Expected cron job info");
        assert_eq!(cron_info.cron_expression, "0 * * * *");
    }

    #[tokio::test]
    async fn test_one_shot_cleanup_does_not_remove_replaced_job_entry() {
        let scheduler = Scheduler::new();
        let gate = Arc::new(tokio::sync::Notify::new());
        let callback_started = Arc::new(AtomicUsize::new(0));

        scheduler
            .start()
            .await
            .expect("Failed to start scheduler");

        let gate_clone = gate.clone();
        let started_clone = callback_started.clone();
        let waiting_callback: JobCallback = Arc::new(move || {
            let gate = gate_clone.clone();
            let started = started_clone.clone();
            Box::pin(async move {
                started.fetch_add(1, Ordering::SeqCst);
                gate.notified().await;
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        scheduler
            .add_one_shot_job("task-race", Duration::from_secs(1), waiting_callback)
            .await
            .expect("Failed to add one-shot job");

        sleep(Duration::from_secs(2)).await;
        assert_eq!(callback_started.load(Ordering::SeqCst), 1);

        let replacement_callback: JobCallback = Arc::new(|| {
            Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        scheduler
            .add_job("task-race", "0 * * * *", replacement_callback)
            .await
            .expect("Failed to add replacement cron job");

        gate.notify_waiters();
        sleep(Duration::from_millis(200)).await;

        assert!(scheduler.has_job("task-race").await);
        let info = scheduler
            .get_job_info("task-race")
            .await
            .expect("Expected replacement job info");
        assert_eq!(info.cron_expression, "0 * * * *");
    }

    #[tokio::test]
    async fn test_remove_job_after_start() {
        let scheduler = Scheduler::new();
        let callback: JobCallback = Arc::new(|| Box::pin(async {}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);

        scheduler
            .add_job("task-1", "*/5 * * * *", callback.clone())
            .await
            .expect("Failed to add job 1");

        scheduler
            .add_job("task-2", "*/10 * * * *", callback)
            .await
            .expect("Failed to add job 2");

        scheduler.start().await.expect("Failed to start scheduler");

        scheduler
            .remove_job("task-1")
            .await
            .expect("Failed to remove job");

        assert_eq!(scheduler.job_count().await, 1);
        assert!(!scheduler.has_job("task-1").await);
        assert!(scheduler.has_job("task-2").await);
    }

    #[tokio::test]
    async fn test_scheduler_full_lifecycle() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let callback: JobCallback = Arc::new(move || {
            let c = counter_clone.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });

        scheduler
            .add_job("task-1", "* * * * *", callback.clone())
            .await
            .expect("Failed to add job 1");

        scheduler
            .add_job("task-2", "* * * * *", callback.clone())
            .await
            .expect("Failed to add job 2");

        assert_eq!(scheduler.job_count().await, 2);

        scheduler.start().await.expect("Failed to start scheduler");
        assert_eq!(scheduler.get_state().await, SchedulerState::Running);

        sleep(Duration::from_millis(100)).await;

        scheduler
            .remove_job("task-1")
            .await
            .expect("Failed to remove job");
        assert_eq!(scheduler.job_count().await, 1);

        scheduler.stop().await.expect("Failed to stop scheduler");
        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);
    }
}

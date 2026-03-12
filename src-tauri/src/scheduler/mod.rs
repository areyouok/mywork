pub mod cron_parser;
pub mod job_scheduler;
pub mod process_tracker;
pub mod simple_schedule;
pub mod task_queue;
pub mod timeout;

use crate::models::task::Task;
use chrono::{DateTime, Utc};
pub use cron_parser::{parse_cron, validate_cron, CronError, CronSchedule};
pub use job_scheduler::{JobCallback, JobInfo, Scheduler, SchedulerError, SchedulerState};
pub use process_tracker::{
    cleanup_orphan_processes, kill_all_processes, register_pid, running_count, unregister_pid,
};
pub use simple_schedule::{parse_simple_schedule, ScheduleError};
pub use task_queue::{SlotGuard, TaskQueue, TaskQueueError};
pub use timeout::{kill_process, run_with_timeout, ProcessOutput, TimeoutError};

pub enum TaskSchedule {
    Cron(String),
    Once(DateTime<Utc>),
}

pub fn get_task_schedule(task: &Task) -> Option<TaskSchedule> {
    if let Some(cron) = &task.cron_expression {
        return Some(TaskSchedule::Cron(cron.clone()));
    }

    if let Some(once_at) = &task.once_at {
        let parsed = DateTime::parse_from_rfc3339(once_at)
            .ok()
            .map(|dt| TaskSchedule::Once(dt.with_timezone(&Utc)));

        if parsed.is_some() {
            return parsed;
        }
    }

    if let Some(json) = &task.simple_schedule {
        return parse_simple_schedule(json).ok().map(TaskSchedule::Cron);
    }

    None
}

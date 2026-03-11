pub mod cron_parser;
pub mod job_scheduler;
pub mod process_tracker;
pub mod simple_schedule;
pub mod task_queue;
pub mod timeout;

pub use cron_parser::{parse_cron, validate_cron, CronError, CronSchedule};
use crate::models::task::Task;
pub use job_scheduler::{JobCallback, JobInfo, Scheduler, SchedulerError, SchedulerState};
pub use process_tracker::{cleanup_orphan_processes, kill_all_processes, register_pid, running_count, unregister_pid};
pub use simple_schedule::{parse_simple_schedule, ScheduleError};
pub use task_queue::{SlotGuard, TaskQueue, TaskQueueError};
pub use timeout::{kill_process, run_with_timeout, ProcessOutput, TimeoutError};

pub fn get_task_cron_expression(task: &Task) -> Option<String> {
    if let Some(cron) = &task.cron_expression {
        Some(cron.clone())
    } else if let Some(json) = &task.simple_schedule {
        parse_simple_schedule(json).ok()
    } else {
        None
    }
}

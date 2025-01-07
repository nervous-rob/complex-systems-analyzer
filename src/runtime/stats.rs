use std::collections::HashMap;
use crate::runtime::ThreadPoolStats;
use crate::runtime::scheduler::{SchedulerStats, Priority};
use crate::runtime::SystemState;

#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub thread_pool_stats: ThreadPoolStats,
    pub scheduler_stats: SchedulerStats,
    pub system_state: SystemState,
}

impl RuntimeStats {
    pub fn is_healthy(&self) -> bool {
        !matches!(self.system_state, SystemState::Error(_))
    }

    pub fn get_active_threads(&self) -> usize {
        self.thread_pool_stats.active_threads
    }

    pub fn get_total_queued_tasks(&self) -> usize {
        self.scheduler_stats.total_tasks
    }

    pub fn get_task_completion_rate(&self) -> f64 {
        let total = self.scheduler_stats.completed_tasks + self.scheduler_stats.failed_tasks;
        if total == 0 {
            0.0
        } else {
            self.scheduler_stats.completed_tasks as f64 / total as f64
        }
    }

    pub fn get_queue_lengths(&self) -> &HashMap<Priority, usize> {
        &self.scheduler_stats.queued_tasks
    }
} 
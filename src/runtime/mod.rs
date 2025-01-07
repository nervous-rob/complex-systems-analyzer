use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};

mod thread_pool;
mod scheduler;
mod lifecycle;
mod stats;

pub use thread_pool::{ThreadPool, ThreadPoolStats};
pub use scheduler::{TaskScheduler, Task, TaskHandle, Priority, TaskStatus, SchedulerConfig};
pub use lifecycle::{LifecycleManager, SystemState, LifecycleHook};
pub use stats::RuntimeStats;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub thread_count: usize,
    pub task_queue_size: usize,
    pub scheduler_config: SchedulerConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            thread_count: num_cpus::get(),
            task_queue_size: 1000,
            scheduler_config: SchedulerConfig::default(),
        }
    }
}

pub struct RuntimeManager {
    thread_pool: Arc<ThreadPool>,
    task_scheduler: Arc<TaskScheduler>,
    lifecycle_manager: Arc<LifecycleManager>,
    config: RuntimeConfig,
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let thread_pool = Arc::new(ThreadPool::new(config.thread_count)?);
        let task_scheduler = Arc::new(TaskScheduler::new(config.scheduler_config.clone())?);
        let lifecycle_manager = Arc::new(LifecycleManager::new()?);

        Ok(Self {
            thread_pool,
            task_scheduler,
            lifecycle_manager,
            config,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        self.lifecycle_manager.start_system().await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.lifecycle_manager.stop_system().await?;
        Ok(())
    }

    pub async fn submit_task(&self, task: Task) -> Result<TaskHandle> {
        self.task_scheduler.schedule_task(task).await
    }

    pub async fn cancel_task(&self, handle: &TaskHandle) -> Result<()> {
        self.task_scheduler.cancel_task(handle).await
    }

    pub async fn get_task_status(&self, handle: &TaskHandle) -> TaskStatus {
        self.task_scheduler.get_task_status(handle).await
    }

    pub async fn update_task_priority(&self, handle: &TaskHandle, priority: Priority) -> Result<()> {
        self.task_scheduler.update_priority(handle, priority).await
    }

    pub async fn get_runtime_stats(&self) -> RuntimeStats {
        RuntimeStats {
            thread_pool_stats: self.thread_pool.get_stats(),
            scheduler_stats: self.task_scheduler.get_stats().await,
            system_state: self.lifecycle_manager.get_system_state().await,
        }
    }
} 
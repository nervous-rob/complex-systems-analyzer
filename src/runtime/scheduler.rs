use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    High,
    Normal,
    Low,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub priority: Priority,
    pub dependencies: Vec<TaskHandle>,
    pub execution: Arc<dyn Fn() -> Result<()> + Send + Sync>,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: Uuid,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
    pub queue_size_per_priority: usize,
    pub default_timeout: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            queue_size_per_priority: 1000,
            default_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

pub struct TaskQueue {
    tasks: Vec<Task>,
    max_size: usize,
}

impl TaskQueue {
    fn new(max_size: usize) -> Self {
        Self {
            tasks: Vec::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, task: Task) -> Result<()> {
        if self.tasks.len() >= self.max_size {
            return Err(Error::Runtime("Task queue is full".into()));
        }
        self.tasks.push(task);
        Ok(())
    }

    fn pop(&mut self) -> Option<Task> {
        self.tasks.pop()
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }
}

pub struct TaskScheduler {
    queues: HashMap<Priority, Arc<RwLock<TaskQueue>>>,
    task_statuses: Arc<RwLock<HashMap<Uuid, TaskStatus>>>,
    config: SchedulerConfig,
}

impl TaskScheduler {
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        let mut queues = HashMap::new();
        for priority in [Priority::High, Priority::Normal, Priority::Low, Priority::Background] {
            queues.insert(
                priority,
                Arc::new(RwLock::new(TaskQueue::new(config.queue_size_per_priority))),
            );
        }

        Ok(Self {
            queues,
            task_statuses: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    pub async fn schedule_task(&self, task: Task) -> Result<TaskHandle> {
        let queue = self.queues.get(&task.priority)
            .ok_or_else(|| Error::Runtime("Invalid task priority".into()))?;

        let handle = TaskHandle {
            id: task.id,
            status: TaskStatus::Queued,
            created_at: Utc::now(),
            priority: task.priority,
        };

        // Update task status
        self.task_statuses.write().await.insert(task.id, TaskStatus::Queued);

        // Add task to queue
        queue.write().await.push(task)?;

        Ok(handle)
    }

    pub async fn cancel_task(&self, handle: &TaskHandle) -> Result<()> {
        let mut statuses = self.task_statuses.write().await;
        if let Some(status) = statuses.get_mut(&handle.id) {
            if *status == TaskStatus::Queued || *status == TaskStatus::Running {
                *status = TaskStatus::Cancelled;
                Ok(())
            } else {
                Err(Error::Runtime("Task cannot be cancelled in its current state".into()))
            }
        } else {
            Err(Error::Runtime("Task not found".into()))
        }
    }

    pub async fn get_task_status(&self, handle: &TaskHandle) -> TaskStatus {
        self.task_statuses.read().await
            .get(&handle.id)
            .copied()
            .unwrap_or(TaskStatus::Failed)
    }

    pub async fn update_priority(&self, handle: &TaskHandle, new_priority: Priority) -> Result<()> {
        // This would require more complex implementation to actually move tasks between queues
        Err(Error::Runtime("Priority update not implemented".into()))
    }

    pub async fn get_stats(&self) -> SchedulerStats {
        let mut stats = SchedulerStats {
            queued_tasks: HashMap::new(),
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
        };

        for (priority, queue) in &self.queues {
            let queue_len = queue.read().await.len();
            stats.queued_tasks.insert(*priority, queue_len);
            stats.total_tasks += queue_len;
        }

        for status in self.task_statuses.read().await.values() {
            match status {
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed | TaskStatus::TimedOut => stats.failed_tasks += 1,
                _ => {}
            }
        }

        stats
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub queued_tasks: HashMap<Priority, usize>,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
} 
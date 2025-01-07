use std::time::Duration;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use super::{AnalysisType, AnalysisConfig};
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTask {
    pub id: Uuid,
    pub analysis_config: AnalysisConfig,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub priority: TaskPriority,
    #[serde(with = "serde_duration")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHandle {
    pub id: Uuid,
    pub task_type: AnalysisType,
    pub status: TaskStatus,
    pub progress: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    #[serde(with = "serde_duration")]
    pub computation_time: Duration,
    pub memory_used: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    High,
    Normal,
    Low,
    Background,
}

impl ComputeTask {
    pub fn new(analysis_config: AnalysisConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            analysis_config,
            created_at: Utc::now(),
            priority: TaskPriority::Normal,
            timeout: Duration::from_secs(3600), // 1 hour default timeout
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        if timeout.as_secs() == 0 {
            self.timeout = Duration::from_secs(3600); // Default to 1 hour if 0
        } else {
            self.timeout = timeout;
        }
        self
    }

    pub fn validate(&self) -> Result<()> {
        // Validate timeout is reasonable
        if self.timeout.as_secs() > 24 * 3600 {
            return Err(Error::computation("Task timeout cannot exceed 24 hours"));
        }

        // Validate analysis config
        if let Some(max_iterations) = self.analysis_config.constraints.max_iterations {
            if max_iterations == 0 {
                return Err(Error::computation("Max iterations must be greater than 0"));
            }
        }

        if let Some(threshold) = self.analysis_config.constraints.convergence_threshold {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(Error::computation("Convergence threshold must be between 0 and 1"));
            }
        }

        if let Some(memory) = self.analysis_config.constraints.max_memory {
            if memory == 0 {
                return Err(Error::computation("Max memory must be greater than 0"));
            }
        }

        Ok(())
    }
}

impl TaskHandle {
    pub fn new(task: &ComputeTask) -> Self {
        Self {
            id: task.id,
            task_type: task.analysis_config.analysis_type.clone(),
            status: TaskStatus::Pending,
            progress: 0.0,
            created_at: task.created_at,
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::TimedOut
        )
    }

    pub fn update_progress(&mut self, progress: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&progress) {
            return Err(Error::computation("Progress must be between 0 and 1"));
        }
        self.progress = progress;
        Ok(())
    }
}

impl ComputeResult {
    pub fn success(task_id: Uuid, result: serde_json::Value, computation_time: Duration, memory_used: usize) -> Self {
        Self {
            task_id,
            status: TaskStatus::Completed,
            result: Some(result),
            error: None,
            computation_time,
            memory_used,
        }
    }

    pub fn failure(task_id: Uuid, error: String, computation_time: Duration, memory_used: usize) -> Self {
        Self {
            task_id,
            status: TaskStatus::Failed,
            result: None,
            error: Some(error),
            computation_time,
            memory_used,
        }
    }
}

// Helper module for serializing Duration
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
} 
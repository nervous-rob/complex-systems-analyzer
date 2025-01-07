use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use crate::error::{Error, Result};

pub struct ThreadPool {
    runtime: Runtime,
    semaphore: Arc<Semaphore>,
    thread_count: usize,
}

#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    pub total_threads: usize,
    pub active_threads: usize,
    pub queued_tasks: usize,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(thread_count)
            .enable_all()
            .build()
            .map_err(|e| Error::Runtime(format!("Failed to create runtime: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(thread_count));

        Ok(Self {
            runtime,
            semaphore,
            thread_count,
        })
    }

    pub async fn spawn<F, T>(&self, task: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let semaphore = Arc::clone(&self.semaphore);
        
        let result = self.runtime.spawn(async move {
            let permit = semaphore.acquire().await
                .map_err(|e| Error::Runtime(format!("Failed to acquire semaphore: {}", e)))?;
            let result = task();
            drop(permit);
            result
        }).await
            .map_err(|e| Error::Runtime(format!("Task execution failed: {}", e)))?;

        result
    }

    pub fn get_stats(&self) -> ThreadPoolStats {
        ThreadPoolStats {
            total_threads: self.thread_count,
            active_threads: self.thread_count - self.semaphore.available_permits(),
            queued_tasks: 0, // This would need to be tracked separately
        }
    }
} 
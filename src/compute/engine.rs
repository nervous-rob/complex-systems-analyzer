use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use rayon::ThreadPool;
use uuid::Uuid;
use sysinfo::{System, SystemExt, ProcessExt};

use crate::error::{Error, Result};
use super::{
    ComputeConfig, ComputeStats,
    task::{ComputeTask, TaskHandle, ComputeResult, TaskStatus},
    algorithms::{
        AnalysisAlgorithm, CentralityAnalysis, CommunityDetection, PathAnalysis,
        CentralityParams, CommunityParams, PathParams, PathWeightFunction,
        CentralityType, CommunityType, PathType,
        Graph, NodeId,
    },
    AnalysisType,
};

pub struct ComputeEngine {
    config: ComputeConfig,
    thread_pool: Arc<ThreadPool>,
    tasks: Arc<RwLock<HashMap<Uuid, TaskHandle>>>,
    results: Arc<RwLock<HashMap<Uuid, ComputeResult>>>,
    stats: Arc<RwLock<ComputeStats>>,
    sys_info: Arc<RwLock<System>>,
}

impl ComputeEngine {
    pub fn new(config: ComputeConfig) -> Result<Self> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.thread_count)
            .build()
            .map_err(|e| Error::computation(format!("Failed to create thread pool: {}", e)))?;

        let mut sys = System::new_all();
        sys.refresh_all();

        Ok(Self {
            config,
            thread_pool: Arc::new(thread_pool),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ComputeStats {
                active_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_task_duration: Duration::from_secs(0),
                memory_usage: 0,
            })),
            sys_info: Arc::new(RwLock::new(sys)),
        })
    }

    pub async fn submit_task(&self, task: ComputeTask) -> Result<TaskHandle> {
        // Validate task before accepting
        task.validate()?;

        let handle = TaskHandle::new(&task);
        
        // Check if we have capacity
        let stats = self.stats.read().await;
        if stats.active_tasks >= self.config.task_queue_size {
            return Err(Error::computation("Task queue is full"));
        }

        // Check memory usage
        let sys = self.sys_info.read().await;
        let current_memory = get_current_memory_usage(&sys);
        if current_memory > self.config.max_memory {
            return Err(Error::computation("System memory usage exceeds limit"));
        }
        drop(sys);
        drop(stats);
        
        // Store task handle
        self.tasks.write().await.insert(task.id, handle.clone());
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.active_tasks += 1;

        // Clone necessary Arc's for the task
        let tasks = Arc::clone(&self.tasks);
        let results = Arc::clone(&self.results);
        let stats = Arc::clone(&self.stats);
        let thread_pool = Arc::clone(&self.thread_pool);
        let sys_info = Arc::clone(&self.sys_info);
        let task_id = task.id;

        // Spawn task execution
        tokio::spawn(async move {
            let start_time = Instant::now();
            
            // Update task status
            if let Some(task_handle) = tasks.write().await.get_mut(&task_id) {
                task_handle.status = TaskStatus::Running;
            }

            let result = Self::execute_task(task, thread_pool).await;
            let duration = start_time.elapsed();

            // Get memory usage
            let sys = sys_info.read().await;
            let memory_used = get_current_memory_usage(&sys);
            drop(sys);

            // Store result
            let mut results = results.write().await;
            let mut tasks = tasks.write().await;
            let mut stats = stats.write().await;

            match &result {
                Ok(compute_result) => {
                    results.insert(compute_result.task_id, compute_result.clone());
                    if let Some(task_handle) = tasks.get_mut(&compute_result.task_id) {
                        task_handle.status = TaskStatus::Completed;
                        task_handle.progress = 1.0;
                    }
                    stats.completed_tasks += 1;
                }
                Err(error) => {
                    let compute_result = ComputeResult::failure(
                        task_id,
                        error.to_string(),
                        duration,
                        memory_used,
                    );
                    results.insert(task_id, compute_result);
                    if let Some(task_handle) = tasks.get_mut(&task_id) {
                        task_handle.status = TaskStatus::Failed;
                    }
                    stats.failed_tasks += 1;
                }
            }

            stats.active_tasks -= 1;
            stats.memory_usage = memory_used;

            // Update average duration
            if stats.completed_tasks > 0 {
                stats.average_task_duration = Duration::from_secs_f64(
                    (stats.average_task_duration.as_secs_f64() * (stats.completed_tasks - 1) as f64
                        + duration.as_secs_f64())
                        / stats.completed_tasks as f64,
                );
            }
        });

        Ok(handle)
    }

    pub async fn get_result(&self, handle: &TaskHandle) -> Result<ComputeResult> {
        let results = self.results.read().await;
        results
            .get(&handle.id)
            .cloned()
            .ok_or_else(|| Error::computation(format!("No result found for task {}", handle.id)))
    }

    pub async fn cancel_task(&self, handle: &TaskHandle) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let mut results = self.results.write().await;
        let mut stats = self.stats.write().await;

        if let Some(task) = tasks.get_mut(&handle.id) {
            if matches!(task.status, TaskStatus::Running | TaskStatus::Pending) {
                task.status = TaskStatus::Cancelled;
                
                // Create a cancelled result
                let result = ComputeResult::failure(
                    handle.id,
                    "Task cancelled by user".to_string(),
                    Duration::from_secs(0),
                    0,
                );
                results.insert(handle.id, result);
                
                // Update stats
                if matches!(task.status, TaskStatus::Running) {
                    stats.active_tasks -= 1;
                }
                stats.failed_tasks += 1;
            }
            Ok(())
        } else {
            Err(Error::computation(format!("Task {} not found", handle.id)))
        }
    }

    pub async fn get_engine_stats(&self) -> ComputeStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update current memory usage
        let sys = self.sys_info.read().await;
        stats.memory_usage = get_current_memory_usage(&sys);
        
        stats
    }

    async fn execute_task(task: ComputeTask, thread_pool: Arc<ThreadPool>) -> Result<ComputeResult> {
        let start_time = Instant::now();

        // Extract and convert graph parameter
        let graph: Graph = task.analysis_config.parameters.get("graph")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| Error::computation("Missing graph data".to_string()))?;

        let analysis_result = match task.analysis_config.analysis_type {
            AnalysisType::Centrality(centrality_type) => {
                // Convert parameters
                let params = CentralityParams {
                    normalize: task.analysis_config.parameters.get("normalize")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(true),
                    weight_threshold: task.analysis_config.parameters.get("weight_threshold")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                };

                let algorithm = CentralityAnalysis::new(centrality_type.into(), params);
                algorithm.execute(graph).await?
            }

            AnalysisType::Community(community_type) => {
                // Convert parameters
                let params = CommunityParams {
                    min_community_size: task.analysis_config.parameters.get("min_community_size")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(3),
                    max_iterations: task.analysis_config.parameters.get("max_iterations")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(100),
                    resolution: task.analysis_config.parameters.get("resolution")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(1.0),
                };

                let algorithm = CommunityDetection::new(community_type.into(), params);
                algorithm.execute(graph).await?
            }

            AnalysisType::Path(path_type) => {
                // Convert parameters
                let params = PathParams {
                    max_path_length: task.analysis_config.parameters.get("max_path_length")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    weight_function: task.analysis_config.parameters.get("weight_function")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(PathWeightFunction::Shortest),
                };
                let source: NodeId = task.analysis_config.parameters.get("source")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| Error::computation("Missing source node".to_string()))?;
                let target: NodeId = task.analysis_config.parameters.get("target") 
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| Error::computation("Missing target node".to_string()))?;

                let algorithm = PathAnalysis::new(path_type.into(), params);
                algorithm.execute((graph, source, target)).await?
            }

            AnalysisType::Custom(ref name) => {
                return Err(Error::computation(format!("Custom analysis type '{}' not implemented", name)));
            }
        };

        let duration = start_time.elapsed();

        // Convert the result to a Value before creating the ComputeResult
        let result_value = serde_json::to_value(&analysis_result)?;

        Ok(ComputeResult::success(
            task.id,
            result_value,
            duration,
            0, // Memory usage is tracked at a higher level
        ))
    }
}

fn get_current_memory_usage(sys: &System) -> usize {
    if let Some(process) = sys.processes().get(&sysinfo::get_current_pid().unwrap()) {
        process.memory() as usize
    } else {
        0
    }
} 
//! Thread Pool Manager for CPU-intensive tasks
//! 
//! Provides specialized thread pools for different types of CPU-intensive operations
//! to prevent blocking the main async runtime.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{oneshot, Semaphore};
use tokio::task::JoinHandle;
use std::collections::HashMap;
use dashmap::DashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Different types of CPU-intensive tasks that require specialized handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Text embeddings generation (vectorization)
    Embedding,
    /// Code analysis and parsing 
    CodeAnalysis,
    /// Document parsing and processing
    DocumentProcessing,
    /// File system operations (large directory scans)
    FileSystemOps,
    /// Compression/decompression operations
    Compression,
    /// General CPU-intensive tasks
    General,
}

/// Configuration for each thread pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPoolConfig {
    pub max_threads: usize,
    pub queue_size: usize,
    pub idle_timeout_seconds: u64,
    pub task_timeout_seconds: u64,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get().max(2),
            queue_size: 100,
            idle_timeout_seconds: 300, // 5 minutes
            task_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Task execution request
#[derive(Debug)]
pub struct TaskRequest<T> {
    pub id: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub payload: T,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Task execution result
#[derive(Debug)]
pub struct TaskResult<R> {
    pub task_id: String,
    pub result: Result<R, String>,
    pub execution_time: Duration,
    pub memory_used: Option<usize>,
}

/// Thread pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPoolStats {
    pub active_threads: usize,
    pub idle_threads: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time_ms: f64,
    pub total_memory_used: usize,
}

/// Individual thread pool for a specific task type
struct WorkerPool {
    config: ThreadPoolConfig,
    semaphore: Arc<Semaphore>,
    stats: Arc<DashMap<String, TaskExecutionStats>>,
    task_count: Arc<std::sync::atomic::AtomicU64>,
    failed_count: Arc<std::sync::atomic::AtomicU64>,
}

#[derive(Debug, Clone)]
struct TaskExecutionStats {
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    #[allow(dead_code)]
    pub memory_used: Option<usize>,
}

impl WorkerPool {
    fn new(config: ThreadPoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_threads)),
            stats: Arc::new(DashMap::new()),
            task_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            failed_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            config,
        }
    }

    async fn execute_task<T, R, F>(&self, task: TaskRequest<T>, executor: F) -> TaskResult<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: FnOnce(T) -> Result<R, String> + Send + 'static,
    {
        let task_id = task.id.clone();
        let start_time = Instant::now();
        
        // Record task start
        self.stats.insert(task_id.clone(), TaskExecutionStats {
            started_at: start_time,
            completed_at: None,
            memory_used: None,
        });

        // Acquire semaphore permit
        let permit = match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                return TaskResult {
                    task_id,
                    result: Err("Thread pool semaphore closed".to_string()),
                    execution_time: start_time.elapsed(),
                    memory_used: None,
                };
            }
        };

        // Execute task on blocking thread pool
        let (tx, rx) = oneshot::channel();
        let task_timeout = task.timeout.unwrap_or(Duration::from_secs(self.config.task_timeout_seconds));

        let handle: JoinHandle<()> = tokio::task::spawn_blocking(move || {
            let result = executor(task.payload);
            let _ = tx.send(result);
            drop(permit); // Release permit when done
        });

        // Wait for completion or timeout
        let result = tokio::select! {
            Ok(result) = rx => result,
            _ = tokio::time::sleep(task_timeout) => {
                handle.abort();
                Err("Task execution timeout".to_string())
            }
        };

        let execution_time = start_time.elapsed();
        
        // Update statistics
        if let Some(mut stats) = self.stats.get_mut(&task_id) {
            stats.completed_at = Some(Instant::now());
        }

        match &result {
            Ok(_) => {
                self.task_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            Err(_) => {
                self.failed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        TaskResult {
            task_id,
            result,
            execution_time,
            memory_used: None, // TODO: Implement memory tracking
        }
    }

    fn get_stats(&self) -> ThreadPoolStats {
        let available_permits = self.semaphore.available_permits();
        let active_threads = self.config.max_threads - available_permits;
        
        let completed_tasks = self.task_count.load(std::sync::atomic::Ordering::SeqCst);
        let failed_tasks = self.failed_count.load(std::sync::atomic::Ordering::SeqCst);
        
        // Calculate average execution time
        let total_execution_time: f64 = self.stats.iter()
            .filter_map(|entry| {
                let stats = entry.value();
                stats.completed_at.map(|completed| {
                    (completed - stats.started_at).as_millis() as f64
                })
            })
            .sum();
        
        let completed_count = self.stats.len() as f64;
        let average_execution_time_ms = if completed_count > 0.0 {
            total_execution_time / completed_count
        } else {
            0.0
        };

        ThreadPoolStats {
            active_threads,
            idle_threads: available_permits,
            queued_tasks: 0, // TODO: Implement queue tracking
            completed_tasks,
            failed_tasks,
            average_execution_time_ms,
            total_memory_used: 0, // TODO: Implement memory tracking
        }
    }
}

/// Main thread pool manager
pub struct ThreadPoolManager {
    pools: HashMap<TaskType, WorkerPool>,
    #[allow(dead_code)]
    default_config: ThreadPoolConfig,
}

impl ThreadPoolManager {
    pub fn new() -> Self {
        Self::new_with_config(ThreadPoolConfig::default())
    }

    pub fn new_with_config(default_config: ThreadPoolConfig) -> Self {
        let mut pools = HashMap::new();
        
        // Create specialized pools for different task types
        let task_types = vec![
            TaskType::Embedding,
            TaskType::CodeAnalysis,
            TaskType::DocumentProcessing,
            TaskType::FileSystemOps,
            TaskType::Compression,
            TaskType::General,
        ];

        for task_type in task_types {
            let config = match task_type {
                TaskType::Embedding => ThreadPoolConfig {
                    max_threads: (num_cpus::get() / 2).max(1), // CPU intensive
                    queue_size: 50,
                    ..default_config.clone()
                },
                TaskType::CodeAnalysis => ThreadPoolConfig {
                    max_threads: num_cpus::get().max(2),
                    queue_size: 100,
                    ..default_config.clone()
                },
                TaskType::FileSystemOps => ThreadPoolConfig {
                    max_threads: (num_cpus::get() * 2).min(8), // IO intensive
                    queue_size: 200,
                    ..default_config.clone()
                },
                _ => default_config.clone(),
            };
            
            pools.insert(task_type, WorkerPool::new(config));
        }

        Self {
            pools,
            default_config,
        }
    }

    /// Execute a CPU-intensive task on the appropriate thread pool
    pub async fn execute_task<T, R, F>(
        &self,
        task: TaskRequest<T>,
        executor: F,
    ) -> TaskResult<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: FnOnce(T) -> Result<R, String> + Send + 'static,
    {
        let pool = self.pools.get(&task.task_type)
            .or_else(|| self.pools.get(&TaskType::General))
            .expect("General thread pool should always exist");

        pool.execute_task(task, executor).await
    }

    /// Execute a batch of tasks concurrently
    pub async fn execute_batch<T, R, F>(
        &self,
        tasks: Vec<TaskRequest<T>>,
        executor_factory: impl Fn() -> F,
    ) -> Vec<TaskResult<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: FnOnce(T) -> Result<R, String> + Send + 'static,
    {
        let mut handles = Vec::new();

        for task in tasks {
            let executor = executor_factory();
            let handle = self.execute_task(task, executor);
            handles.push(handle);
        }

        // Wait for all tasks to complete
        futures::future::join_all(handles).await
    }

    /// Get statistics for all thread pools
    pub fn get_all_stats(&self) -> HashMap<TaskType, ThreadPoolStats> {
        self.pools.iter()
            .map(|(task_type, pool)| (task_type.clone(), pool.get_stats()))
            .collect()
    }

    /// Get statistics for a specific task type
    pub fn get_stats(&self, task_type: &TaskType) -> Option<ThreadPoolStats> {
        self.pools.get(task_type).map(|pool| pool.get_stats())
    }

    /// Create a task request with automatic ID generation
    pub fn create_task<T>(
        task_type: TaskType,
        priority: TaskPriority,
        payload: T,
    ) -> TaskRequest<T> {
        TaskRequest {
            id: format!("task_{}_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                rand::random::<u32>()
            ),
            task_type,
            priority,
            payload,
            timeout: None,
        }
    }
}

impl Default for ThreadPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_thread_pool_execution() {
        let manager = ThreadPoolManager::new();
        
        let task = ThreadPoolManager::create_task(
            TaskType::General,
            TaskPriority::Normal,
            42i32,
        );

        let result = manager.execute_task(task, |x| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(x * 2)
        }).await;

        assert!(result.result.is_ok());
        assert_eq!(result.result.unwrap(), 84);
        assert!(result.execution_time.as_millis() >= 100);
    }

    #[tokio::test]
    async fn test_batch_execution() {
        let manager = ThreadPoolManager::new();
        
        let tasks = (0..5).map(|i| {
            ThreadPoolManager::create_task(
                TaskType::General,
                TaskPriority::Normal,
                i,
            )
        }).collect();

        let results = manager.execute_batch(tasks, || {
            |x: i32| {
                std::thread::sleep(std::time::Duration::from_millis(50));
                Ok(x * 2)
            }
        }).await;

        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert!(result.result.is_ok());
            assert_eq!(result.result.as_ref().unwrap(), &(i as i32 * 2));
        }
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let manager = ThreadPoolManager::new();
        
        let mut task = ThreadPoolManager::create_task(
            TaskType::General,
            TaskPriority::Normal,
            (),
        );
        task.timeout = Some(Duration::from_millis(50));

        let result = manager.execute_task(task, |_| {
            std::thread::sleep(std::time::Duration::from_millis(200));
            Ok("should timeout")
        }).await;

        assert!(result.result.is_err());
        assert!(result.result.unwrap_err().contains("timeout"));
    }
}
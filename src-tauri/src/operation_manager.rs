//! Operation Manager for queuing and prioritizing backend operations

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    AICompletion,
    FileAnalysis,
    DocumentIndexing,
    CodeGeneration,
    RagQuery,
    ModelLoading,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OperationPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Background = 3,
    Maintenance = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub op_type: OperationType,
    pub priority: OperationPriority,
    pub created_at: u64,
    pub estimated_resources: ResourceRequirements,
    pub timeout_ms: Option<u64>,
    pub cancellable: bool,
    pub payload: serde_json::Value,
    pub completed_at: Option<u64>,
    pub memory_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_units: u32,
    pub memory_mb: u32,
    pub io_intensity: u32,
    pub network_kb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq)]
pub enum OperationStatus {
    Queued,
    Running { progress: Option<f32> },
    Completed { result: Option<serde_json::Value> },
    Failed { error: String },
    Cancelled,
    TimedOut,
}

pub struct OperationManager {
    pub operations: DashMap<String, (Operation, OperationStatus)>,
    pub queue_tx: mpsc::Sender<Operation>,
    pub resource_limits: ResourceLimits,
    pub semaphore: Arc<Semaphore>,
    pub cleanup_config: CleanupConfig,
    pub memory_monitor: Arc<MemoryMonitor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    pub cleanup_interval_seconds: u64,
    pub max_completed_operations: usize,
    pub max_operation_age_seconds: u64,
    pub cleanup_failed_operations: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_seconds: 300, // 5 minutes
            max_completed_operations: 1000,
            max_operation_age_seconds: 3600, // 1 hour
            cleanup_failed_operations: true,
        }
    }
}

#[derive(Debug)]
pub struct MemoryMonitor {
    pub current_usage: Arc<std::sync::atomic::AtomicU64>,
    pub peak_usage: Arc<std::sync::atomic::AtomicU64>,
    pub operations_memory: DashMap<String, u64>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            current_usage: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            peak_usage: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            operations_memory: DashMap::new(),
        }
    }

    pub fn allocate(&self, operation_id: &str, bytes: u64) {
        self.operations_memory.insert(operation_id.to_string(), bytes);
        let new_usage = self.current_usage.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst) + bytes;
        
        // Update peak usage
        let mut peak = self.peak_usage.load(std::sync::atomic::Ordering::SeqCst);
        while new_usage > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, 
                new_usage, 
                std::sync::atomic::Ordering::SeqCst, 
                std::sync::atomic::Ordering::SeqCst
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
    }

    pub fn deallocate(&self, operation_id: &str) -> Option<u64> {
        if let Some((_, bytes)) = self.operations_memory.remove(operation_id) {
            self.current_usage.fetch_sub(bytes, std::sync::atomic::Ordering::SeqCst);
            Some(bytes)
        } else {
            None
        }
    }

    pub fn current_usage(&self) -> u64 {
        self.current_usage.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn peak_usage(&self) -> u64 {
        self.peak_usage.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_concurrent_operations: usize,
    pub max_memory_usage: u64,
    pub max_cpu_usage: f32,
    pub io_throttling: bool,
}

impl OperationManager {
    pub fn new(resource_limits: ResourceLimits) -> Self {
        Self::new_with_cleanup(resource_limits, CleanupConfig::default())
    }

    pub fn new_with_cleanup(resource_limits: ResourceLimits, cleanup_config: CleanupConfig) -> Self {
        let (queue_tx, mut queue_rx) = mpsc::channel::<Operation>(100);
        let operations = DashMap::new();
        let operations_clone = operations.clone();
        let semaphore = Arc::new(Semaphore::new(resource_limits.max_concurrent_operations));
        let semaphore_clone = semaphore.clone();
        let memory_monitor = Arc::new(MemoryMonitor::new());
        let memory_monitor_clone = memory_monitor.clone();
        // Spawn the operation processor
        tokio::spawn(async move {
            let mut priority_queues: Vec<Vec<Operation>> = vec![Vec::new(); 5];
            loop {
                while let Ok(operation) = queue_rx.try_recv() {
                    let priority = operation.priority.clone() as usize;
                    priority_queues[priority].push(operation);
                }
                for priority_level in 0..priority_queues.len() {
                    if priority_queues[priority_level].is_empty() {
                        continue;
                    }
                    let operation = priority_queues[priority_level].remove(0);
                    let op_id = operation.id.clone();
                    operations_clone.insert(op_id.clone(), (operation.clone(), OperationStatus::Running { progress: None }));
                    let permit = match semaphore_clone.try_acquire() {
                        Ok(permit) => permit,
                        Err(_) => {
                            priority_queues[priority_level].push(operation);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                    };
                    let ops_map = operations_clone.clone();
                    let memory_monitor_task = memory_monitor_clone.clone();
                    tokio::spawn(async move {
                        let start_time = Instant::now();
                        
                        // Estimate and allocate memory for operation
                        let estimated_memory = (operation.estimated_resources.memory_mb as u64) * 1024 * 1024;
                        memory_monitor_task.allocate(&op_id, estimated_memory);
                        
                        // Actual operation execution logic
                        let result = match operation.op_type {
                            OperationType::AICompletion => mock_ai_completion(&operation).await,
                            OperationType::FileAnalysis => mock_file_analysis(&operation).await,
                            OperationType::DocumentIndexing => mock_document_indexing(&operation).await,
                            OperationType::CodeGeneration => mock_code_generation(&operation).await,
                            OperationType::RagQuery => mock_rag_query(&operation).await,
                            OperationType::ModelLoading => mock_model_loading(&operation).await,
                        };
                        
                        // Record completion time and memory used
                        let execution_time = start_time.elapsed();
                        let completed_at = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        
                        let mut updated_operation = operation.clone();
                        updated_operation.completed_at = Some(completed_at);
                        updated_operation.memory_used = Some(estimated_memory);
                        
                        match result {
                            Ok(res) => {
                                ops_map.insert(op_id.clone(), (updated_operation, OperationStatus::Completed { result: Some(res) }));
                            },
                            Err(e) => {
                                ops_map.insert(op_id.clone(), (updated_operation, OperationStatus::Failed { error: e.to_string() }));
                            }
                        }
                        
                        // Deallocate memory
                        memory_monitor_task.deallocate(&op_id);
                        drop(permit);
                    });
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Mock async operation handlers for demonstration
        async fn mock_ai_completion(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(300)).await;
            Ok(serde_json::json!({"result": "AI completion done"}))
        }
        async fn mock_file_analysis(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(serde_json::json!({"result": "File analysis complete"}))
        }
        async fn mock_document_indexing(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(250)).await;
            Ok(serde_json::json!({"result": "Document indexed"}))
        }
        async fn mock_code_generation(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(350)).await;
            Ok(serde_json::json!({"result": "Code generated"}))
        }
        async fn mock_rag_query(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(400)).await;
            Ok(serde_json::json!({"result": "RAG query complete"}))
        }
        async fn mock_model_loading(_op: &Operation) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(Duration::from_millis(150)).await;
            Ok(serde_json::json!({"result": "Model loaded"}))
        }

        // Spawn cleanup task
        let operations_cleanup = operations.clone();
        let cleanup_config_clone = cleanup_config.clone();
        let memory_monitor_cleanup = memory_monitor.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(cleanup_config_clone.cleanup_interval_seconds));
            loop {
                interval.tick().await;
                Self::cleanup_operations(&operations_cleanup, &cleanup_config_clone, &memory_monitor_cleanup).await;
            }
        });

        Self {
            operations,
            queue_tx,
            resource_limits,
            semaphore,
            cleanup_config,
            memory_monitor,
        }
    }

    pub async fn enqueue_operation(&self, mut operation: Operation) -> Result<String, String> {
        let op_id = operation.id.clone();
        
        // Initialize fields for new operation
        operation.completed_at = None;
        operation.memory_used = None;
        
        self.operations.insert(op_id.clone(), (operation.clone(), OperationStatus::Queued));
        match self.queue_tx.send(operation).await {
            Ok(_) => Ok(op_id),
            Err(e) => Err(format!("Failed to enqueue operation: {}", e))
        }
    }

    /// Helper method to create a new operation with default values
    pub fn create_operation(
        id: String,
        op_type: OperationType,
        priority: OperationPriority,
        estimated_resources: ResourceRequirements,
        payload: serde_json::Value,
    ) -> Operation {
        Operation {
            id,
            op_type,
            priority,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            estimated_resources,
            timeout_ms: None,
            cancellable: true,
            payload,
            completed_at: None,
            memory_used: None,
        }
    }

    pub fn get_operation_status(&self, operation_id: &str) -> Option<OperationStatus> {
        self.operations.get(operation_id).map(|entry| entry.value().1.clone())
    }

    pub fn cancel_operation(&self, operation_id: &str) -> Result<(), String> {
        if let Some(mut entry) = self.operations.get_mut(operation_id) {
            if entry.1 == OperationStatus::Queued || 
               matches!(entry.1, OperationStatus::Running { .. }) && entry.0.cancellable {
                entry.1 = OperationStatus::Cancelled;
                return Ok(());
            }
            return Err("Operation cannot be cancelled in its current state".to_string());
        }
        Err("Operation not found".to_string())
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage: self.memory_monitor.current_usage(),
            peak_usage: self.memory_monitor.peak_usage(),
            operation_count: self.operations.len(),
            active_operations: self.operations.iter()
                .filter(|entry| matches!(entry.1, OperationStatus::Running { .. }))
                .count(),
        }
    }

    /// Get operations by status for monitoring
    pub fn get_operations_by_status(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in self.operations.iter() {
            let status_key = match &entry.1 {
                OperationStatus::Queued => "queued",
                OperationStatus::Running { .. } => "running",
                OperationStatus::Completed { .. } => "completed",
                OperationStatus::Failed { .. } => "failed",
                OperationStatus::Cancelled => "cancelled",
                OperationStatus::TimedOut => "timed_out",
            };
            *counts.entry(status_key.to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Manual cleanup trigger
    pub async fn cleanup_now(&self) -> CleanupStats {
        Self::cleanup_operations(&self.operations, &self.cleanup_config, &self.memory_monitor).await
    }

    /// Internal cleanup implementation
    async fn cleanup_operations(
        operations: &DashMap<String, (Operation, OperationStatus)>,
        config: &CleanupConfig,
        memory_monitor: &MemoryMonitor,
    ) -> CleanupStats {
        let mut stats = CleanupStats::default();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Collect operations to remove
        let mut to_remove = Vec::new();
        let mut completed_operations = Vec::new();

        for entry in operations.iter() {
            let (operation, status) = entry.value();
            let should_remove = match status {
                OperationStatus::Completed { .. } => {
                    completed_operations.push((operation.created_at, entry.key().clone()));
                    if let Some(completed_at) = operation.completed_at {
                        current_time - completed_at > config.max_operation_age_seconds
                    } else {
                        current_time - operation.created_at > config.max_operation_age_seconds
                    }
                }
                OperationStatus::Failed { .. } => {
                    config.cleanup_failed_operations && 
                    current_time - operation.created_at > config.max_operation_age_seconds
                }
                OperationStatus::Cancelled | OperationStatus::TimedOut => {
                    current_time - operation.created_at > config.max_operation_age_seconds
                }
                _ => false,
            };

            if should_remove {
                to_remove.push(entry.key().clone());
            }
        }

        // Handle excess completed operations
        if completed_operations.len() > config.max_completed_operations {
            completed_operations.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by creation time
            let excess_count = completed_operations.len() - config.max_completed_operations;
            for (_, op_id) in completed_operations.iter().take(excess_count) {
                if !to_remove.contains(op_id) {
                    to_remove.push(op_id.clone());
                }
            }
        }

        // Remove operations and deallocate memory
        for op_id in &to_remove {
            if let Some((_, (operation, status))) = operations.remove(op_id) {
                memory_monitor.deallocate(op_id);
                stats.operations_removed += 1;
                
                if let Some(memory_used) = operation.memory_used {
                    stats.memory_freed += memory_used;
                }

                match status {
                    OperationStatus::Completed { .. } => stats.completed_removed += 1,
                    OperationStatus::Failed { .. } => stats.failed_removed += 1,
                    OperationStatus::Cancelled => stats.cancelled_removed += 1,
                    OperationStatus::TimedOut => stats.timed_out_removed += 1,
                    _ => {}
                }
            }
        }

        stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub operation_count: usize,
    pub active_operations: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub operations_removed: usize,
    pub memory_freed: u64,
    pub completed_removed: usize,
    pub failed_removed: usize,
    pub cancelled_removed: usize,
    pub timed_out_removed: usize,
}

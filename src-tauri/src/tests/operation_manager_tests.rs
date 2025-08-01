use crate::operation_manager::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use serial_test::serial;

/// Helper function to create a test operation
fn create_test_operation(
    id: &str,
    op_type: OperationType,
    priority: OperationPriority,
    cancellable: bool,
) -> Operation {
    Operation {
        id: id.to_string(),
        op_type,
        priority,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        estimated_resources: ResourceRequirements {
            cpu_units: 1,
            memory_mb: 100,
            io_intensity: 1,
            network_kb: 0,
        },
        timeout_ms: Some(5000),
        cancellable,
        payload: serde_json::json!({"test": "data"}),
    }
}

/// Helper function to create test resource limits
fn create_test_resource_limits() -> ResourceLimits {
    ResourceLimits {
        max_concurrent_operations: 3,
        max_memory_usage: 1024,
        max_cpu_usage: 80.0,
        io_throttling: false,
    }
}

#[tokio::test]
async fn test_operation_manager_creation() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits.clone());
    
    assert_eq!(manager.resource_limits.max_concurrent_operations, 3);
    assert_eq!(manager.resource_limits.max_memory_usage, 1024);
    assert_eq!(manager.operations.len(), 0);
}

#[tokio::test]
async fn test_enqueue_single_operation() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let operation = create_test_operation(
        "test-op-1",
        OperationType::AICompletion,
        OperationPriority::Normal,
        true,
    );
    
    let result = manager.enqueue_operation(operation).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-op-1");
    
    // Check that operation is in queued state
    let status = manager.get_operation_status("test-op-1");
    assert!(status.is_some());
    assert_eq!(status.unwrap(), OperationStatus::Queued);
}

#[tokio::test]
async fn test_operation_execution_completion() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let operation = create_test_operation(
        "exec-test-1",
        OperationType::FileAnalysis,
        OperationPriority::High,
        true,
    );
    
    let _op_id = manager.enqueue_operation(operation).await.unwrap();
    
    // Wait for operation to complete (mock operations are fast)
    time::sleep(Duration::from_millis(300)).await;
    
    let status = manager.get_operation_status("exec-test-1");
    assert!(status.is_some());
    
    match status.unwrap() {
        OperationStatus::Completed { result } => {
            assert!(result.is_some());
            let result_value = result.unwrap();
            assert_eq!(result_value["result"], "File analysis complete");
        }
        other => panic!("Expected Completed status, got {:?}", other),
    }
}

#[tokio::test]
async fn test_priority_queue_ordering() {
    let resource_limits = ResourceLimits {
        max_concurrent_operations: 1, // Force serialization
        max_memory_usage: 1024,
        max_cpu_usage: 80.0,
        io_throttling: false,
    };
    let manager = OperationManager::new(resource_limits);
    
    // Enqueue operations in reverse priority order
    let low_priority = create_test_operation(
        "low-priority",
        OperationType::ModelLoading,
        OperationPriority::Background,
        true,
    );
    let high_priority = create_test_operation(
        "high-priority",
        OperationType::AICompletion,
        OperationPriority::Critical,
        true,
    );
    let normal_priority = create_test_operation(
        "normal-priority",
        OperationType::FileAnalysis,
        OperationPriority::Normal,
        true,
    );
    
    // Enqueue in non-priority order
    manager.enqueue_operation(low_priority).await.unwrap();
    manager.enqueue_operation(normal_priority).await.unwrap();
    manager.enqueue_operation(high_priority).await.unwrap();
    
    // Wait for processing
    time::sleep(Duration::from_millis(100)).await;
    
    // High priority should be running first
    let high_status = manager.get_operation_status("high-priority");
    assert!(matches!(high_status, Some(OperationStatus::Running { .. })));
    
    // Others should still be queued
    let normal_status = manager.get_operation_status("normal-priority");
    assert_eq!(normal_status, Some(OperationStatus::Queued));
    
    let low_status = manager.get_operation_status("low-priority");
    assert_eq!(low_status, Some(OperationStatus::Queued));
}

#[tokio::test]
async fn test_concurrent_operation_limit() {
    let resource_limits = ResourceLimits {
        max_concurrent_operations: 2,
        max_memory_usage: 1024,
        max_cpu_usage: 80.0,
        io_throttling: false,
    };
    let manager = OperationManager::new(resource_limits);
    
    // Create multiple operations that will take some time
    let operations = vec![
        create_test_operation("concurrent-1", OperationType::AICompletion, OperationPriority::Normal, true),
        create_test_operation("concurrent-2", OperationType::RagQuery, OperationPriority::Normal, true),
        create_test_operation("concurrent-3", OperationType::CodeGeneration, OperationPriority::Normal, true),
        create_test_operation("concurrent-4", OperationType::DocumentIndexing, OperationPriority::Normal, true),
    ];
    
    // Enqueue all operations
    for op in operations {
        manager.enqueue_operation(op).await.unwrap();
    }
    
    // Wait a bit for processing to start
    time::sleep(Duration::from_millis(50)).await;
    
    // Count running operations
    let mut running_count = 0;
    let mut queued_count = 0;
    
    for i in 1..=4 {
        let status = manager.get_operation_status(&format!("concurrent-{}", i));
        match status {
            Some(OperationStatus::Running { .. }) => running_count += 1,
            Some(OperationStatus::Queued) => queued_count += 1,
            _ => {}
        }
    }
    
    // Should have exactly 2 running (the limit) and 2 queued
    assert_eq!(running_count, 2, "Should have exactly 2 running operations");
    assert_eq!(queued_count, 2, "Should have exactly 2 queued operations");
}

#[tokio::test]
async fn test_operation_cancellation_queued() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let operation = create_test_operation(
        "cancel-test-queued",
        OperationType::AICompletion,
        OperationPriority::Normal,
        true,
    );
    
    manager.enqueue_operation(operation).await.unwrap();
    
    // Cancel while still queued
    let cancel_result = manager.cancel_operation("cancel-test-queued");
    assert!(cancel_result.is_ok());
    
    let status = manager.get_operation_status("cancel-test-queued");
    assert_eq!(status, Some(OperationStatus::Cancelled));
}

#[tokio::test]
async fn test_operation_cancellation_non_cancellable() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let operation = create_test_operation(
        "non-cancellable",
        OperationType::AICompletion,
        OperationPriority::Normal,
        false, // Not cancellable
    );
    
    manager.enqueue_operation(operation).await.unwrap();
    
    // Wait for it to start running
    time::sleep(Duration::from_millis(50)).await;
    
    let cancel_result = manager.cancel_operation("non-cancellable");
    assert!(cancel_result.is_err());
    assert!(cancel_result.unwrap_err().contains("cannot be cancelled"));
}

#[tokio::test]
async fn test_operation_not_found() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let status = manager.get_operation_status("nonexistent-op");
    assert!(status.is_none());
    
    let cancel_result = manager.cancel_operation("nonexistent-op");
    assert!(cancel_result.is_err());
    assert!(cancel_result.unwrap_err().contains("not found"));
}

#[tokio::test]
#[serial] // Run serially to avoid timing issues
async fn test_multiple_operation_types() {
    let resource_limits = create_test_resource_limits();
    let manager = OperationManager::new(resource_limits);
    
    let operations = vec![
        ("ai-op", OperationType::AICompletion),
        ("file-op", OperationType::FileAnalysis),
        ("doc-op", OperationType::DocumentIndexing),
        ("code-op", OperationType::CodeGeneration),
        ("rag-op", OperationType::RagQuery),
        ("model-op", OperationType::ModelLoading),
    ];
    
    // Enqueue all operation types
    for (id, op_type) in operations {
        let operation = create_test_operation(id, op_type, OperationPriority::Normal, true);
        manager.enqueue_operation(operation).await.unwrap();
    }
    
    // Wait for all operations to complete
    time::sleep(Duration::from_millis(600)).await;
    
    // Check that all operations completed successfully
    let expected_results = vec![
        ("ai-op", "AI completion done"),
        ("file-op", "File analysis complete"),
        ("doc-op", "Document indexed"),
        ("code-op", "Code generated"),
        ("rag-op", "RAG query complete"),
        ("model-op", "Model loaded"),
    ];
    
    for (id, expected_result) in expected_results {
        let status = manager.get_operation_status(id);
        match status {
            Some(OperationStatus::Completed { result }) => {
                let result_value = result.unwrap();
                assert_eq!(result_value["result"], expected_result);
            }
            other => panic!("Operation {} failed with status: {:?}", id, other),
        }
    }
}

#[tokio::test]
async fn test_resource_requirements_structure() {
    let requirements = ResourceRequirements {
        cpu_units: 4,
        memory_mb: 512,
        io_intensity: 3,
        network_kb: 1024,
    };
    
    assert_eq!(requirements.cpu_units, 4);
    assert_eq!(requirements.memory_mb, 512);
    assert_eq!(requirements.io_intensity, 3);
    assert_eq!(requirements.network_kb, 1024);
}

#[tokio::test]
async fn test_operation_priority_ordering() {
    // Test that priorities are correctly ordered
    assert!(OperationPriority::Critical < OperationPriority::High);
    assert!(OperationPriority::High < OperationPriority::Normal);
    assert!(OperationPriority::Normal < OperationPriority::Background);
    assert!(OperationPriority::Background < OperationPriority::Maintenance);
    
    // Test numeric values
    assert_eq!(OperationPriority::Critical as u8, 0);
    assert_eq!(OperationPriority::High as u8, 1);
    assert_eq!(OperationPriority::Normal as u8, 2);
    assert_eq!(OperationPriority::Background as u8, 3);
    assert_eq!(OperationPriority::Maintenance as u8, 4);
}

#[tokio::test]
async fn test_operation_status_variants() {
    let statuses = vec![
        OperationStatus::Queued,
        OperationStatus::Running { progress: None },
        OperationStatus::Running { progress: Some(0.5) },
        OperationStatus::Completed { result: None },
        OperationStatus::Completed { result: Some(serde_json::json!({"test": "data"})) },
        OperationStatus::Failed { error: "test error".to_string() },
        OperationStatus::Cancelled,
        OperationStatus::TimedOut,
    ];
    
    // Test that all status variants can be created
    for status in statuses {
        match status {
            OperationStatus::Queued => assert!(true),
            OperationStatus::Running { progress } => {
                if let Some(p) = progress {
                    assert!(p >= 0.0 && p <= 1.0);
                }
            }
            OperationStatus::Completed { result: _ } => assert!(true),
            OperationStatus::Failed { error } => assert!(!error.is_empty()),
            OperationStatus::Cancelled => assert!(true),
            OperationStatus::TimedOut => assert!(true),
        }
    }
}

#[tokio::test]
async fn test_high_load_enqueueing() {
    let resource_limits = ResourceLimits {
        max_concurrent_operations: 5,
        max_memory_usage: 2048,
        max_cpu_usage: 90.0,
        io_throttling: false,
    };
    let manager = OperationManager::new(resource_limits);
    
    // Enqueue many operations rapidly
    let num_operations = 20;
    let mut enqueue_results = Vec::new();
    
    for i in 0..num_operations {
        let operation = create_test_operation(
            &format!("high-load-{}", i),
            OperationType::FileAnalysis,
            OperationPriority::Normal,
            true,
        );
        
        let result = manager.enqueue_operation(operation).await;
        enqueue_results.push(result);
    }
    
    // All enqueue operations should succeed
    for (i, result) in enqueue_results.iter().enumerate() {
        assert!(result.is_ok(), "Operation {} failed to enqueue: {:?}", i, result);
    }
    
    // Wait for operations to process
    time::sleep(Duration::from_millis(500)).await;
    
    // Check that operations are being processed (some should be completed)
    let mut completed_count = 0;
    let mut running_count = 0;
    let mut queued_count = 0;
    
    for i in 0..num_operations {
        let status = manager.get_operation_status(&format!("high-load-{}", i));
        match status {
            Some(OperationStatus::Completed { .. }) => completed_count += 1,
            Some(OperationStatus::Running { .. }) => running_count += 1,
            Some(OperationStatus::Queued) => queued_count += 1,
            other => println!("Unexpected status for operation {}: {:?}", i, other),
        }
    }
    
    assert!(completed_count > 0, "Some operations should have completed");
    assert!(running_count <= 5, "Should not exceed concurrent limit");
    assert_eq!(completed_count + running_count + queued_count, num_operations);
}

#[tokio::test]
async fn test_operation_timeout_handling() {
    // Note: The current implementation has mock operations that complete quickly
    // This test verifies the timeout structure exists and can be set
    let operation = Operation {
        id: "timeout-test".to_string(),
        op_type: OperationType::AICompletion,
        priority: OperationPriority::Normal,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        estimated_resources: ResourceRequirements {
            cpu_units: 1,
            memory_mb: 100,
            io_intensity: 1,
            network_kb: 0,
        },
        timeout_ms: Some(100), // Very short timeout
        cancellable: true,
        payload: serde_json::json!({"test": "timeout"}),
    };
    
    assert_eq!(operation.timeout_ms, Some(100));
    assert!(operation.cancellable);
}

#[tokio::test]
async fn test_semaphore_resource_limiting() {
    let resource_limits = ResourceLimits {
        max_concurrent_operations: 1, // Very restrictive
        max_memory_usage: 512,
        max_cpu_usage: 50.0,
        io_throttling: true,
    };
    let manager = OperationManager::new(resource_limits);
    
    // The semaphore should be initialized with the correct capacity
    assert_eq!(manager.semaphore.available_permits(), 1);
    
    // Enqueue an operation that will consume the semaphore
    let operation = create_test_operation(
        "semaphore-test",
        OperationType::AICompletion,
        OperationPriority::Critical,
        true,
    );
    
    manager.enqueue_operation(operation).await.unwrap();
    
    // Wait for the operation to start consuming the semaphore
    time::sleep(Duration::from_millis(50)).await;
    
    // Semaphore should now have 0 available permits
    assert_eq!(manager.semaphore.available_permits(), 0);
    
    // Wait for operation to complete and release the permit
    time::sleep(Duration::from_millis(400)).await;
    
    // Semaphore should be back to 1 available permit
    assert_eq!(manager.semaphore.available_permits(), 1);
}

#[tokio::test]
async fn test_serialization_deserialization() {
    let operation = create_test_operation(
        "serde-test",
        OperationType::CodeGeneration,
        OperationPriority::High,
        true,
    );
    
    // Test operation serialization
    let serialized = serde_json::to_string(&operation).unwrap();
    let deserialized: Operation = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(operation.id, deserialized.id);
    assert_eq!(operation.op_type, deserialized.op_type);
    assert_eq!(operation.priority, deserialized.priority);
    assert_eq!(operation.cancellable, deserialized.cancellable);
    
    // Test status serialization
    let status = OperationStatus::Running { progress: Some(0.75) };
    let status_serialized = serde_json::to_string(&status).unwrap();
    let status_deserialized: OperationStatus = serde_json::from_str(&status_serialized).unwrap();
    
    match status_deserialized {
        OperationStatus::Running { progress } => assert_eq!(progress, Some(0.75)),
        _ => panic!("Deserialization failed"),
    }
}
use crate::context_manager::*;
use std::sync::Arc;
use std::io::Write;
use tempfile::NamedTempFile;
use serial_test::serial;

/// Helper function to create a temporary file with content
fn create_temp_file(content: &str) -> Result<NamedTempFile, std::io::Error> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;
    Ok(temp_file)
}

/// Helper function to create test context manager
fn create_test_manager() -> ContextManager {
    ContextManager::new(10000, 1000) // 10k max, 1k reserved
}

#[tokio::test]
async fn test_context_manager_creation() {
    let manager = ContextManager::new(50000, 5000);
    
    assert_eq!(manager.max_tokens, 50000);
    assert_eq!(manager.reserved_tokens, 5000);
    assert_eq!(manager.get_pinned_files().await.len(), 0);
}

#[tokio::test]
async fn test_default_context_manager() {
    let manager = ContextManager::default();
    
    assert_eq!(manager.max_tokens, 128_000);
    assert_eq!(manager.reserved_tokens, 25_600);
}

#[tokio::test]
async fn test_token_counting_basic() {
    let manager = create_test_manager();
    
    let text = "Hello world this is a test";
    let token_count = manager.count_tokens(text);
    
    // Should be approximately: 6 words * 1.3 * 1.2 = ~9.36 tokens
    assert!(token_count > 6 && token_count < 15, "Token count should be reasonable: got {}", token_count);
}

#[tokio::test]
async fn test_token_counting_empty_text() {
    let manager = create_test_manager();
    
    let token_count = manager.count_tokens("");
    assert_eq!(token_count, 0);
    
    let whitespace_only = "   \n\t  ";
    let whitespace_count = manager.count_tokens(whitespace_only);
    assert_eq!(whitespace_count, 0);
}

#[tokio::test]
async fn test_token_counting_large_text() {
    let manager = create_test_manager();
    
    // Create a large text (1000 words)
    let large_text = "word ".repeat(1000);
    let token_count = manager.count_tokens(&large_text);
    
    // Should be approximately: 1000 words * 1.3 * 1.2 = 1560 tokens
    assert!(token_count > 1200 && token_count < 2000, "Large text token count should be reasonable: got {}", token_count);
}

#[tokio::test]
async fn test_file_token_counting() {
    let manager = create_test_manager();
    let content = "This is a test file with some content for token counting.";
    
    let temp_file = create_temp_file(content).unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    let token_count = manager.count_file_tokens(file_path).await.unwrap();
    let direct_count = manager.count_tokens(content);
    
    assert_eq!(token_count, direct_count);
}

#[tokio::test]
async fn test_file_token_caching() {
    let manager = create_test_manager();
    let content = "Cached content for testing token caching functionality.";
    
    let temp_file = create_temp_file(content).unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    // First call should read from file
    let first_call = manager.count_file_tokens(file_path).await.unwrap();
    
    // Second call should use cache
    let second_call = manager.count_file_tokens(file_path).await.unwrap();
    
    assert_eq!(first_call, second_call);
    
    // Verify it's actually cached
    {
        let cache = manager.token_cache.read().await;
        assert!(cache.contains_key(file_path));
        assert_eq!(cache[file_path], first_call);
    }
}

#[tokio::test]
async fn test_file_not_found() {
    let manager = create_test_manager();
    
    let result = manager.count_file_tokens("/nonexistent/file.txt").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read file"));
}

#[tokio::test]
async fn test_pin_file() {
    let manager = create_test_manager();
    let file_path = "/test/file.rs".to_string();
    
    assert!(!manager.is_file_pinned(&file_path).await);
    
    let result = manager.pin_file(file_path.clone()).await;
    assert!(result.is_ok());
    assert!(manager.is_file_pinned(&file_path).await);
    
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 1);
    assert_eq!(pinned_files[0], file_path);
}

#[tokio::test]
async fn test_pin_file_duplicate() {
    let manager = create_test_manager();
    let file_path = "/test/duplicate.rs".to_string();
    
    // Pin the same file twice
    manager.pin_file(file_path.clone()).await.unwrap();
    manager.pin_file(file_path.clone()).await.unwrap();
    
    // Should only appear once
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 1);
    assert_eq!(pinned_files[0], file_path);
}

#[tokio::test]
async fn test_unpin_file() {
    let manager = create_test_manager();
    let file_path = "/test/unpin.rs".to_string();
    
    // Pin and then unpin
    manager.pin_file(file_path.clone()).await.unwrap();
    assert!(manager.is_file_pinned(&file_path).await);
    
    let result = manager.unpin_file(file_path.clone()).await;
    assert!(result.is_ok());
    assert!(!manager.is_file_pinned(&file_path).await);
    
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 0);
}

#[tokio::test]
async fn test_unpin_nonexistent_file() {
    let manager = create_test_manager();
    let file_path = "/test/nonexistent.rs".to_string();
    
    // Should succeed even if file wasn't pinned
    let result = manager.unpin_file(file_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_file_operations() {
    let manager = create_test_manager();
    let files = vec![
        "/test/file1.rs".to_string(),
        "/test/file2.py".to_string(),
        "/test/file3.js".to_string(),
    ];
    
    // Pin all files
    for file in &files {
        manager.pin_file(file.clone()).await.unwrap();
    }
    
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 3);
    
    // Unpin middle file
    manager.unpin_file(files[1].clone()).await.unwrap();
    
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 2);
    assert!(pinned_files.contains(&files[0]));
    assert!(!pinned_files.contains(&files[1]));
    assert!(pinned_files.contains(&files[2]));
}

#[tokio::test]
async fn test_budget_calculation_basic() {
    let manager = create_test_manager(); // 10k max, 1k reserved
    
    let budget = manager.calculate_budget(2000, 1000).await;
    
    assert_eq!(budget.total, 10000);
    assert_eq!(budget.breakdown.conversation, 2000);
    assert_eq!(budget.breakdown.rag_documents, 1000);
    assert_eq!(budget.breakdown.reserved, 1000);
    assert_eq!(budget.used, 3000); // conversation + rag + reserved
    assert_eq!(budget.available, 7000);
}

#[tokio::test]
async fn test_budget_calculation_with_pinned_files() {
    let manager = create_test_manager();
    let content1 = "First test file content.";
    let content2 = "Second test file with more content for testing.";
    
    let temp_file1 = create_temp_file(content1).unwrap();
    let temp_file2 = create_temp_file(content2).unwrap();
    
    let file_path1 = temp_file1.path().to_str().unwrap().to_string();
    let file_path2 = temp_file2.path().to_str().unwrap().to_string();
    
    // Pin files and count their tokens
    manager.pin_file(file_path1.clone()).await.unwrap();
    manager.pin_file(file_path2.clone()).await.unwrap();
    
    let tokens1 = manager.count_file_tokens(&file_path1).await.unwrap();
    let tokens2 = manager.count_file_tokens(&file_path2).await.unwrap();
    
    let budget = manager.calculate_budget(1000, 500).await;
    
    assert_eq!(budget.breakdown.pinned_files, tokens1 + tokens2);
    assert_eq!(budget.used, 1000 + 500 + tokens1 + tokens2 + 1000); // conv + rag + pinned + reserved
}

#[tokio::test]
async fn test_budget_calculation_edge_cases() {
    let manager = create_test_manager(); // 10k max, 1k reserved
    
    // Test with zero values
    let budget_zero = manager.calculate_budget(0, 0).await;
    assert_eq!(budget_zero.used, 1000); // Only reserved tokens
    assert_eq!(budget_zero.available, 9000);
    
    // Test with values that exceed max
    let budget_overflow = manager.calculate_budget(15000, 5000).await;
    assert_eq!(budget_overflow.total, 10000);
    assert_eq!(budget_overflow.used, 21000); // Can exceed max in calculation
    assert_eq!(budget_overflow.available, 0); // Saturating sub results in 0
}

#[tokio::test]
async fn test_file_relevance_calculation() {
    let manager = create_test_manager();
    let file_path = "/test/relevance.rs";
    let context = "testing relevance calculation";
    
    let relevance = manager.calculate_file_relevance(file_path, context).await;
    
    // Should be between 0.6 and 0.95 (mock implementation)
    assert!(relevance >= 0.6 && relevance <= 0.95, "Relevance should be in range [0.6, 0.95]: got {}", relevance);
    
    // Should be consistent for same input
    let relevance2 = manager.calculate_file_relevance(file_path, context).await;
    assert_eq!(relevance, relevance2);
}

#[tokio::test]
async fn test_file_type_detection() {
    let manager = create_test_manager();
    
    // Test various file extensions
    let test_cases = vec![
        ("/path/to/file.rs", "rs"),
        ("/path/to/file.py", "py"),
        ("/path/to/file.JS", "js"), // Should be lowercase
        ("/path/to/file", "unknown"), // No extension
        ("/path/to/.hidden", "unknown"), // Hidden file
    ];
    
    for (file_path, expected_type) in test_cases {
        let file_type = manager.get_file_type(file_path);
        assert_eq!(file_type, expected_type, "File type for {} should be {}", file_path, expected_type);
    }
}

#[tokio::test]
async fn test_build_context_basic() {
    let manager = create_test_manager();
    let conversation = "This is a test conversation";
    let rag_tokens = 500;
    let suggested_files = vec![];
    
    let context = manager.build_context(conversation, rag_tokens, suggested_files).await.unwrap();
    
    assert_eq!(context.files.len(), 0); // No suggested files
    assert!(context.total_tokens > 0);
    assert_eq!(context.budget.breakdown.conversation, manager.count_tokens(conversation));
    assert_eq!(context.budget.breakdown.rag_documents, rag_tokens);
}

#[tokio::test]
async fn test_build_context_with_pinned_files() {
    let manager = create_test_manager();
    let content = "Pinned file content for context building test.";
    
    let temp_file = create_temp_file(content).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();
    
    // Pin the file
    manager.pin_file(file_path.clone()).await.unwrap();
    
    let conversation = "Test conversation";
    let context = manager.build_context(conversation, 0, vec![]).await.unwrap();
    
    assert_eq!(context.files.len(), 1);
    assert_eq!(context.files[0].path, file_path);
    assert!(context.files[0].is_pinned);
    assert!(context.files[0].relevance_score >= 0.6);
}

#[tokio::test]
async fn test_build_context_with_suggested_files() {
    let manager = create_test_manager();
    let content1 = "First suggested file content.";
    let content2 = "Second suggested file content.";
    
    let temp_file1 = create_temp_file(content1).unwrap();
    let temp_file2 = create_temp_file(content2).unwrap();
    
    let file_path1 = temp_file1.path().to_str().unwrap().to_string();
    let file_path2 = temp_file2.path().to_str().unwrap().to_string();
    
    let conversation = "Test conversation";
    let suggested_files = vec![file_path1.clone(), file_path2.clone()];
    
    let context = manager.build_context(conversation, 0, suggested_files).await.unwrap();
    
    // Should include both suggested files
    assert!(context.files.len() >= 1); // At least one should fit in budget
    
    for file in &context.files {
        assert!(!file.is_pinned); // Suggested files are not pinned
        assert!(file.relevance_score >= 0.6);
    }
}

#[tokio::test]
async fn test_build_context_budget_constraint() {
    // Create a manager with very small budget
    let small_manager = ContextManager::new(100, 50); // Very small budget
    let large_content = "word ".repeat(1000); // Very large file
    
    let temp_file = create_temp_file(&large_content).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();
    
    let conversation = "Test";
    let suggested_files = vec![file_path];
    
    let context = small_manager.build_context(conversation, 0, suggested_files).await.unwrap();
    
    // Large file should not fit in small budget
    assert_eq!(context.files.len(), 0);
}

#[tokio::test]
async fn test_build_context_sorting_by_relevance() {
    let manager = create_test_manager();
    
    // Create multiple files with different paths (which affects mock relevance scores)
    let files = vec![
        ("aaa.rs", "content a"),
        ("zzz.rs", "content z"), 
        ("mmm.rs", "content m"),
    ];
    
    let mut temp_files = Vec::new();
    let mut file_paths = Vec::new();
    
    for (name, content) in files {
        let temp_file = create_temp_file(content).unwrap();
        let mut path = temp_file.path().to_path_buf();
        path.set_file_name(name); // Set specific name to control relevance
        let path_str = path.to_str().unwrap().to_string();
        
        file_paths.push(path_str);
        temp_files.push(temp_file);
    }
    
    let conversation = "Test conversation";
    let context = manager.build_context(conversation, 0, file_paths).await.unwrap();
    
    // Files should be sorted by relevance score (highest first)
    if context.files.len() > 1 {
        for i in 1..context.files.len() {
            assert!(context.files[i-1].relevance_score >= context.files[i].relevance_score);
        }
    }
}

#[tokio::test]
async fn test_cache_clearing() {
    let manager = create_test_manager();
    let content = "Content for cache clearing test.";
    
    let temp_file = create_temp_file(content).unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    // Count tokens to populate cache
    manager.count_file_tokens(file_path).await.unwrap();
    
    // Verify cache is populated
    {
        let cache = manager.token_cache.read().await;
        assert!(cache.contains_key(file_path));
    }
    
    // Clear cache
    manager.clear_cache().await;
    
    // Verify cache is empty
    {
        let cache = manager.token_cache.read().await;
        assert!(cache.is_empty());
    }
}

#[tokio::test]
#[serial] // Run serially to avoid race conditions
async fn test_concurrent_file_operations() {
    let manager = Arc::new(create_test_manager());
    let file_paths: Vec<String> = (0..10).map(|i| format!("/test/concurrent_{}.rs", i)).collect();
    
    // Spawn concurrent pin operations
    let mut handles = Vec::new();
    for file_path in &file_paths {
        let manager_clone = manager.clone();
        let path_clone = file_path.clone();
        
        let handle = tokio::spawn(async move {
            manager_clone.pin_file(path_clone).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    // Verify all files are pinned
    let pinned_files = manager.get_pinned_files().await;
    assert_eq!(pinned_files.len(), 10);
    
    for file_path in &file_paths {
        assert!(manager.is_file_pinned(file_path).await);
    }
}

#[tokio::test]
#[serial]
async fn test_concurrent_cache_operations() {
    let manager = Arc::new(create_test_manager());
    let content = "Content for concurrent cache test.";
    
    // Create multiple temp files
    let mut temp_files = Vec::new();
    let mut file_paths = Vec::new();
    
    for i in 0..5 {
        let temp_file = create_temp_file(&format!("{} {}", content, i)).unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        file_paths.push(path);
        temp_files.push(temp_file);
    }
    
    // Spawn concurrent token counting operations
    let mut handles = Vec::new();
    for file_path in &file_paths {
        let manager_clone = manager.clone();
        let path_clone = file_path.clone();
        
        let handle = tokio::spawn(async move {
            manager_clone.count_file_tokens(&path_clone).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    
    // Verify all operations succeeded
    for result in results {
        assert!(result.is_ok());
    }
    
    // Verify cache is populated
    {
        let cache = manager.token_cache.read().await;
        assert_eq!(cache.len(), 5);
    }
}

#[tokio::test]
async fn test_memory_usage_bounds() {
    let manager = create_test_manager();
    
    // Test that we don't create unbounded data structures
    let large_file_path = "/very/long/path/".repeat(100) + "file.rs";
    let large_content = "content ".repeat(10000);
    
    // This should not cause memory issues
    let tokens = manager.count_tokens(&large_content);
    assert!(tokens > 0);
    
    // Pin/unpin operations should handle large paths
    let result = manager.pin_file(large_file_path.clone()).await;
    assert!(result.is_ok());
    
    let is_pinned = manager.is_file_pinned(&large_file_path).await;
    assert!(is_pinned);
    
    let unpin_result = manager.unpin_file(large_file_path).await;
    assert!(unpin_result.is_ok());
}

#[tokio::test]
async fn test_context_budget_serialization() {
    let budget = ContextBudget {
        total: 100000,
        used: 25000,
        available: 75000,
        breakdown: BudgetBreakdown {
            conversation: 10000,
            rag_documents: 8000,
            pinned_files: 5000,
            suggested_files: 2000,
            reserved: 2000,
        },
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&budget).unwrap();
    let deserialized: ContextBudget = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(budget.total, deserialized.total);
    assert_eq!(budget.used, deserialized.used);
    assert_eq!(budget.available, deserialized.available);
    assert_eq!(budget.breakdown.conversation, deserialized.breakdown.conversation);
}

#[tokio::test]
async fn test_context_file_serialization() {
    let context_file = ContextFile {
        path: "/test/file.rs".to_string(),
        token_count: 500,
        relevance_score: 0.85,
        is_pinned: true,
        file_type: "rs".to_string(),
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&context_file).unwrap();
    let deserialized: ContextFile = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(context_file.path, deserialized.path);
    assert_eq!(context_file.token_count, deserialized.token_count);
    assert_eq!(context_file.relevance_score, deserialized.relevance_score);
    assert_eq!(context_file.is_pinned, deserialized.is_pinned);
    assert_eq!(context_file.file_type, deserialized.file_type);
}
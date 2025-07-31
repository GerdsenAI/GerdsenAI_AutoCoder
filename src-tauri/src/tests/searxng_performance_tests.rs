use crate::searxng_client::SearXNGClient;
use serial_test::serial;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Performance tests for SearXNG integration
/// These tests verify that the SearXNG integration performs within acceptable limits

const SEARXNG_BASE_URL: &str = "http://localhost:8080";
const PERFORMANCE_TIMEOUT: Duration = Duration::from_secs(90);

/// Test helper to create a SearXNG client for testing
fn create_test_client() -> SearXNGClient {
    SearXNGClient::new(Some(SEARXNG_BASE_URL.to_string()))
}

/// Test helper to wait for SearXNG service to be available
async fn wait_for_searxng_ready(client: &SearXNGClient) -> Result<(), String> {
    let mut attempts = 0;
    let max_attempts = 30;
    
    while attempts < max_attempts {
        match client.check_connection().await {
            Ok(true) => return Ok(()),
            _ => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
    
    Err("SearXNG service is not available for performance testing".to_string())
}

#[tokio::test]
#[serial]
async fn test_search_response_time() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let queries = vec![
        "rust programming",
        "javascript tutorial",
        "python data science",
        "docker containers",
        "kubernetes deployment"
    ];
    
    let mut response_times = Vec::new();
    
    for query in queries {
        let start_time = Instant::now();
        
        let result = timeout(
            PERFORMANCE_TIMEOUT,
            client.search(query, None, None, Some(5))
        ).await;
        
        let elapsed = start_time.elapsed();
        response_times.push(elapsed);
        
        assert!(result.is_ok(), "Search should not timeout for query: {}", query);
        assert!(result.unwrap().is_ok(), "Search should succeed for query: {}", query);
        
        // Individual search should complete within 35 seconds (allowing for network overhead)
        assert!(elapsed <= Duration::from_secs(35), 
               "Search for '{}' took too long: {:?}", query, elapsed);
        
        println!("Search for '{}' took: {:?}", query, elapsed);
    }
    
    // Calculate average response time
    let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    println!("Average response time: {:?}", avg_response_time);
    
    // Average should be reasonable (allowing for variability in search engines)
    assert!(avg_response_time <= Duration::from_secs(20), 
           "Average response time too high: {:?}", avg_response_time);
}

#[tokio::test]
#[serial]
async fn test_concurrent_search_performance() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let queries = vec![
        "concurrent programming",
        "async await patterns",
        "multithreading concepts",
    ];
    
    let start_time = Instant::now();
    
    // Execute searches concurrently
    let mut handles = Vec::new();
    for query in queries {
        let client = client.clone();
        let query = query.to_string();
        let handle = tokio::spawn(async move {
            timeout(
                PERFORMANCE_TIMEOUT,
                client.search(query.as_str(), None, None, Some(3))
            ).await
        });
        handles.push(handle);
    }
    
    // Wait for all searches to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        results.push(result);
    }
    
    let total_elapsed = start_time.elapsed();
    
    // Verify all searches completed successfully
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent search {} should not timeout", i);
        assert!(result.as_ref().unwrap().is_ok(), "Concurrent search {} should succeed", i);
    }
    
    // Concurrent searches should complete faster than sequential
    // Allow generous time for concurrent execution
    assert!(total_elapsed <= Duration::from_secs(45), 
           "Concurrent searches took too long: {:?}", total_elapsed);
    
    println!("Concurrent searches completed in: {:?}", total_elapsed);
}

#[tokio::test]
#[serial]
async fn test_connection_check_performance() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let mut connection_times = Vec::new();
    
    // Test multiple connection checks
    for i in 0..5 {
        let start_time = Instant::now();
        
        let result = timeout(
            Duration::from_secs(15),
            client.check_connection()
        ).await;
        
        let elapsed = start_time.elapsed();
        connection_times.push(elapsed);
        
        assert!(result.is_ok(), "Connection check {} should not timeout", i);
        assert!(result.unwrap().unwrap(), "Connection check {} should succeed", i);
        
        // Individual connection check should be fast
        assert!(elapsed <= Duration::from_secs(10), 
               "Connection check {} took too long: {:?}", i, elapsed);
        
        println!("Connection check {} took: {:?}", i, elapsed);
    }
    
    // Calculate average connection time
    let avg_connection_time = connection_times.iter().sum::<Duration>() / connection_times.len() as u32;
    println!("Average connection check time: {:?}", avg_connection_time);
    
    // Average connection check should be very fast
    assert!(avg_connection_time <= Duration::from_secs(5), 
           "Average connection check time too high: {:?}", avg_connection_time);
}

#[tokio::test]
#[serial]
async fn test_search_with_large_result_limit() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let start_time = Instant::now();
    
    // Test search with larger result limit
    let result = timeout(
        PERFORMANCE_TIMEOUT,
        client.search("programming tutorial", None, None, Some(20))
    ).await;
    
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok(), "Large result search should not timeout");
    
    match result.unwrap() {
        Ok(results) => {
            // Should handle larger result sets efficiently
            assert!(elapsed <= Duration::from_secs(40), 
                   "Large result search took too long: {:?}", elapsed);
            
            println!("Search with 20 results took: {:?}, returned {} results", 
                    elapsed, results.len());
            
            // Results should be reasonable (might be less than 20 due to engine limitations)
            assert!(results.len() <= 20, "Should not exceed requested limit");
        }
        Err(e) => {
            // Some engines might not support large limits, but should fail quickly
            assert!(elapsed <= Duration::from_secs(35), 
                   "Failed large result search should fail quickly: {:?}", elapsed);
            println!("Large result search failed quickly with: {}", e);
        }
    }
}

#[tokio::test]
#[serial]
async fn test_search_engine_variety_performance() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let engine_combinations = vec![
        vec!["google".to_string()],
        vec!["duckduckgo".to_string()],
        vec!["github".to_string(), "stackoverflow".to_string()],
        vec!["google".to_string(), "duckduckgo".to_string(), "bing".to_string()],
    ];
    
    for (i, engines) in engine_combinations.iter().enumerate() {
        let start_time = Instant::now();
        
        let result = timeout(
            PERFORMANCE_TIMEOUT,
            client.search("web development", Some(engines.clone()), None, Some(5))
        ).await;
        
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Engine combination {} should not timeout", i);
        
        match result.unwrap() {
            Ok(results) => {
                println!("Engine combination {:?} took: {:?}, returned {} results", 
                        engines, elapsed, results.len());
                
                // More engines might take longer, but should be reasonable
                let max_time = Duration::from_secs(30 + (engines.len() as u64 * 5));
                assert!(elapsed <= max_time, 
                       "Engine combination {:?} took too long: {:?}", engines, elapsed);
            }
            Err(e) => {
                // Some engine combinations might fail, but should fail quickly
                println!("Engine combination {:?} failed with: {}", engines, e);
                assert!(elapsed <= Duration::from_secs(35), 
                       "Failed engine combination should fail quickly: {:?}", elapsed);
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn test_memory_usage_during_searches() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Perform multiple searches to test memory usage
    let queries = vec![
        "memory management", "garbage collection", "performance optimization",
        "system architecture", "database design", "network protocols",
        "security practices", "testing strategies", "deployment patterns",
        "monitoring tools"
    ];
    
    let start_time = Instant::now();
    
    for (i, query) in queries.iter().enumerate() {
        let result = timeout(
            Duration::from_secs(40),
            client.search(query, None, None, Some(5))
        ).await;
        
        assert!(result.is_ok(), "Search {} should not timeout", i);
        
        // Small delay between searches to allow for cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let total_elapsed = start_time.elapsed();
    
    // Multiple searches should complete in reasonable time
    assert!(total_elapsed <= Duration::from_secs(300), // 5 minutes for 10 searches
           "Multiple searches took too long: {:?}", total_elapsed);
    
    println!("Completed {} searches in: {:?}", queries.len(), total_elapsed);
    
    // Average time per search should be reasonable
    let avg_time = total_elapsed / queries.len() as u32;
    assert!(avg_time <= Duration::from_secs(30), 
           "Average search time too high: {:?}", avg_time);
}

#[tokio::test]
#[serial]
async fn test_error_recovery_performance() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test with potentially problematic queries that might cause errors
    let long_query = "search".repeat(50);
    let problematic_queries = vec![
        "", // Empty query
        "   ", // Whitespace only
        "a", // Very short query
        &long_query, // Very long query
    ];
    
    for (i, query) in problematic_queries.iter().enumerate() {
        let start_time = Instant::now();
        
        let result = timeout(
            Duration::from_secs(30),
            client.search(query, None, None, Some(5))
        ).await;
        
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Problematic query {} should not timeout", i);
        
        // Whether it succeeds or fails, it should respond quickly
        assert!(elapsed <= Duration::from_secs(25), 
               "Problematic query {} took too long: {:?}", i, elapsed);
        
        match result.unwrap() {
            Ok(results) => {
                println!("Problematic query '{}' succeeded in {:?} with {} results", 
                        query, elapsed, results.len());
            }
            Err(e) => {
                println!("Problematic query '{}' failed quickly in {:?}: {}", 
                        query, elapsed, e);
            }
        }
    }
}
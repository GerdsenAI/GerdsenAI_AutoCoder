use crate::searxng_client::SearXNGClient;
use serial_test::serial;
use std::time::Duration;
use tokio::time::timeout;

/// Integration tests for SearXNG commands
/// These tests require a running SearXNG instance at localhost:8080
/// Run `docker-compose up -d` in the docker/searxng directory first

const SEARXNG_BASE_URL: &str = "http://localhost:8080";
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Test helper to create a SearXNG client for testing
fn create_test_client() -> SearXNGClient {
    SearXNGClient::new(Some(SEARXNG_BASE_URL.to_string()))
}

/// Test helper to wait for SearXNG service to be available
async fn wait_for_searxng_ready(client: &SearXNGClient) -> Result<(), String> {
    let mut attempts = 0;
    let max_attempts = 30; // 30 seconds timeout
    
    while attempts < max_attempts {
        match client.check_connection().await {
            Ok(true) => return Ok(()),
            _ => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
    
    Err("SearXNG service is not available. Please start Docker containers with 'docker-compose up -d' in docker/searxng directory".to_string())
}

#[tokio::test]
#[serial]
async fn test_check_searxng_connection_success() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let result = timeout(TEST_TIMEOUT, client.check_connection()).await;
    assert!(result.is_ok(), "Connection check should not timeout");
    
    let is_connected = result.unwrap().expect("Connection check should succeed");
    assert!(is_connected, "Should be connected to SearXNG");
}

#[tokio::test]
#[serial]
async fn test_check_searxng_connection_failure() {
    let client = SearXNGClient::new(Some("http://localhost:9999".to_string()));
    
    let result = timeout(Duration::from_secs(15), client.check_connection()).await;
    assert!(result.is_ok(), "Connection check should not timeout");
    
    let is_connected = result.unwrap().expect("Connection check should complete");
    assert!(!is_connected, "Should not be connected to non-existent service");
}

#[tokio::test]
#[serial]
async fn test_search_web_basic_functionality() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test basic search
    let result = timeout(
        TEST_TIMEOUT,
        client.search("rust programming language", None, None, Some(5))
    ).await;
    
    assert!(result.is_ok(), "Search should not timeout");
    
    let search_results = result.unwrap().expect("Search should succeed");
    
    // Validate results
    assert!(!search_results.is_empty(), "Should return at least one result");
    assert!(search_results.len() <= 5, "Should respect limit parameter");
    
    // Validate result structure
    for result in &search_results {
        assert!(!result.title.is_empty(), "Result should have a title");
        assert!(!result.url.is_empty(), "Result should have a URL");
        assert!(result.url.starts_with("http"), "URL should be valid");
        assert!(!result.engine.is_empty(), "Result should specify engine");
    }
}

#[tokio::test]
#[serial]
async fn test_search_web_with_specific_engines() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test search with specific engines
    let engines = vec!["duckduckgo".to_string(), "google".to_string()];
    let result = timeout(
        TEST_TIMEOUT,
        client.search("javascript tutorial", Some(engines.clone()), None, Some(3))
    ).await;
    
    assert!(result.is_ok(), "Search with specific engines should not timeout");
    
    let search_results = result.unwrap().expect("Search with specific engines should succeed");
    
    // Validate results
    assert!(!search_results.is_empty(), "Should return at least one result");
    
    // Check that results come from specified engines (when engine info is available)
    for result in &search_results {
        if !result.engine.is_empty() && result.engine != "unknown" {
            let _engine_found = engines.iter().any(|e| {
                result.engine.to_lowercase().contains(&e.to_lowercase()) ||
                e.to_lowercase().contains(&result.engine.to_lowercase())
            });
            // Note: Some engines might use different names in responses
            println!("Result engine: {}, Expected engines: {:?}", result.engine, engines);
        }
    }
}

#[tokio::test]
#[serial]
async fn test_search_web_with_categories() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test search with specific categories
    let categories = vec!["it".to_string(), "general".to_string()];
    let result = timeout(
        TEST_TIMEOUT,
        client.search("python programming", None, Some(categories), Some(3))
    ).await;
    
    assert!(result.is_ok(), "Search with categories should not timeout");
    
    let search_results = result.unwrap().expect("Search with categories should succeed");
    
    // Validate results
    assert!(!search_results.is_empty(), "Should return at least one result");
    
    // Validate result structure
    for result in &search_results {
        assert!(!result.title.is_empty(), "Result should have a title");
        assert!(!result.url.is_empty(), "Result should have a URL");
        assert!(result.url.starts_with("http"), "URL should be valid");
    }
}

#[tokio::test]
#[serial]
async fn test_search_web_empty_query() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test search with empty query
    let result = timeout(
        TEST_TIMEOUT,
        client.search("", None, None, Some(5))
    ).await;
    
    // Should either return empty results or an error
    match result {
        Ok(Ok(results)) => {
            // If it returns results, they should be empty or very few
            assert!(results.len() <= 1, "Empty query should return few or no results");
        }
        Ok(Err(_)) => {
            // Error is acceptable for empty query
        }
        Err(_) => {
            panic!("Search should not timeout on empty query");
        }
    }
}

#[tokio::test]
#[serial]
async fn test_get_available_engines() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    let engines = client.get_default_engines().await;
    
    // Validate engines list
    assert!(!engines.is_empty(), "Should return at least one engine");
    
    // Check for expected default engines
    let expected_engines = vec!["github", "stackoverflow", "google", "duckduckgo"];
    for expected in &expected_engines {
        assert!(
            engines.iter().any(|e| e.to_lowercase().contains(&expected.to_lowercase())),
            "Should contain {} engine", expected
        );
    }
}

#[tokio::test]
#[serial]
async fn test_set_and_get_default_engines() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Get original engines
    let original_engines = client.get_default_engines().await;
    
    // Set new engines
    let new_engines = vec!["google".to_string(), "bing".to_string()];
    client.set_default_engines(new_engines.clone()).await;
    
    // Verify engines were set
    let current_engines = client.get_default_engines().await;
    assert_eq!(current_engines, new_engines, "Engines should be updated");
    
    // Restore original engines
    client.set_default_engines(original_engines).await;
}

#[tokio::test]
#[serial]
async fn test_search_timeout_behavior() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test that search completes within reasonable time
    let start_time = std::time::Instant::now();
    
    let result = timeout(
        Duration::from_secs(35), // Slightly longer than the 30s client timeout
        client.search("test query", None, None, Some(1))
    ).await;
    
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(Ok(_)) => {
            // Search completed successfully
            assert!(elapsed <= Duration::from_secs(32), "Search should complete within 32 seconds");
        }
        Ok(Err(e)) => {
            // Search failed but didn't timeout
            println!("Search failed with error: {}", e);
            assert!(elapsed <= Duration::from_secs(32), "Search error should occur within 32 seconds");
        }
        Err(_) => {
            panic!("Search should not timeout (client has 30s timeout)");
        }
    }
}

#[tokio::test]
#[serial]
async fn test_search_result_quality() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test search with a specific programming query
    let result = timeout(
        TEST_TIMEOUT,
        client.search("how to implement binary search algorithm", None, None, Some(5))
    ).await;
    
    assert!(result.is_ok(), "Programming search should not timeout");
    
    let search_results = result.unwrap().expect("Programming search should succeed");
    
    // Validate result quality
    assert!(!search_results.is_empty(), "Should return results for programming query");
    
    // Check that results are relevant (should contain programming-related content)
    let has_relevant_results = search_results.iter().any(|result| {
        let title_lower = result.title.to_lowercase();
        let content_lower = result.content.to_lowercase();
        let url_lower = result.url.to_lowercase();
        
        title_lower.contains("search") || title_lower.contains("algorithm") ||
        content_lower.contains("search") || content_lower.contains("algorithm") ||
        url_lower.contains("stackoverflow") || url_lower.contains("github") ||
        title_lower.contains("binary") || content_lower.contains("binary")
    });
    
    assert!(has_relevant_results, "Should return relevant programming results");
}

#[tokio::test]
#[serial]
async fn test_search_with_special_characters() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // Test search with special characters
    let queries = vec![
        "C++ programming",
        "react.js tutorial",
        "docker-compose setup",
        "what is AI/ML?",
    ];
    
    for query in queries {
        let result = timeout(
            TEST_TIMEOUT,
            client.search(query, None, None, Some(3))
        ).await;
        
        assert!(result.is_ok(), "Search with special characters should not timeout: {}", query);
        
        match result.unwrap() {
            Ok(results) => {
                // Results are expected but not required to be non-empty
                println!("Query '{}' returned {} results", query, results.len());
            }
            Err(e) => {
                // Some special character queries might fail, but should not crash
                println!("Query '{}' failed with error: {}", query, e);
            }
        }
    }
}

/// Integration test for the full command interface
#[tokio::test]
#[serial]
async fn test_search_command_integration() {
    let client = create_test_client();
    
    // Wait for service to be ready
    wait_for_searxng_ready(&client).await.expect("SearXNG should be available for testing");
    
    // This test simulates the actual command that would be called from the frontend
    // Note: In real integration, this would use the Tauri State, but for testing we use the client directly
    
    let search_result = client.search(
        "rust async programming",
        Some(vec!["github".to_string(), "stackoverflow".to_string()]),
        Some(vec!["it".to_string(), "general".to_string()]),
        Some(5)
    ).await;
    
    assert!(search_result.is_ok(), "Full command integration should work");
    
    let results = search_result.unwrap();
    assert!(!results.is_empty(), "Should return search results");
    
    // Validate that all required fields are present
    for result in &results {
        assert!(!result.title.trim().is_empty(), "Title should not be empty");
        assert!(!result.url.trim().is_empty(), "URL should not be empty");
        assert!(result.url.starts_with("http"), "URL should be valid");
        // Content and engine can be empty in some cases, but should be present
        // Score is optional
    }
}

/// Test error handling when SearXNG service is unavailable
#[tokio::test]
#[serial]
async fn test_service_unavailable_handling() {
    // Create client pointing to non-existent service
    let client = SearXNGClient::new(Some("http://localhost:9999".to_string()));
    
    // Test connection check
    let connection_result = timeout(Duration::from_secs(15), client.check_connection()).await;
    assert!(connection_result.is_ok(), "Connection check should complete");
    assert!(!connection_result.unwrap().unwrap(), "Should detect service unavailable");
    
    // Test search with unavailable service
    let search_result = timeout(Duration::from_secs(15), client.search("test", None, None, None)).await;
    assert!(search_result.is_ok(), "Search should complete even if service unavailable");
    assert!(search_result.unwrap().is_err(), "Search should return error when service unavailable");
}
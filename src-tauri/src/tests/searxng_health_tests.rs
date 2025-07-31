use crate::searxng_client::SearXNGClient;
use serial_test::serial;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Health monitoring tests for SearXNG integration
/// These tests verify that health monitoring and service status detection work correctly

const SEARXNG_BASE_URL: &str = "http://localhost:8080";
const HEALTH_TIMEOUT: Duration = Duration::from_secs(30);

/// Test helper to create a SearXNG client for testing
fn create_test_client() -> SearXNGClient {
    SearXNGClient::new(Some(SEARXNG_BASE_URL.to_string()))
}

#[tokio::test]
#[serial]
async fn test_health_check_when_service_running() {
    let client = create_test_client();
    
    // Wait for service to be ready
    let mut attempts = 0;
    let max_attempts = 30;
    
    let mut last_error = None;
    
    while attempts < max_attempts {
        match timeout(Duration::from_secs(10), client.check_connection()).await {
            Ok(Ok(true)) => break,
            Ok(Ok(false)) => {
                last_error = Some("Service reported unhealthy".to_string());
            }
            Ok(Err(e)) => {
                last_error = Some(format!("Health check error: {}", e));
            }
            Err(_) => {
                last_error = Some("Health check timeout".to_string());
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        attempts += 1;
    }
    
    if attempts >= max_attempts {
        panic!("SearXNG service is not available for health testing. Last error: {:?}. Please start Docker containers with 'docker-compose up -d' in docker/searxng directory", last_error);
    }
    
    // Now test health check response time
    let start_time = Instant::now();
    
    let result = timeout(HEALTH_TIMEOUT, client.check_connection()).await;
    
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok(), "Health check should not timeout");
    assert!(result.unwrap().unwrap(), "Service should be healthy");
    
    // Health check should be fast
    assert!(elapsed <= Duration::from_secs(10), 
           "Health check took too long: {:?}", elapsed);
    
    println!("Health check completed in: {:?}", elapsed);
}

#[tokio::test]
#[serial]
async fn test_health_check_when_service_down() {
    // Create client pointing to non-existent service
    let client = SearXNGClient::new(Some("http://localhost:9999".to_string()));
    
    let start_time = Instant::now();
    
    let result = timeout(Duration::from_secs(15), client.check_connection()).await;
    
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok(), "Health check should complete even when service is down");
    assert!(!result.unwrap().unwrap(), "Should detect that service is down");
    
    // Failed health check should still be reasonably fast
    assert!(elapsed <= Duration::from_secs(12), 
           "Failed health check took too long: {:?}", elapsed);
    
    println!("Failed health check completed in: {:?}", elapsed);
}

#[tokio::test]
#[serial]
async fn test_health_check_consistency() {
    let client = create_test_client();
    
    // Wait for service to be ready
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for consistency testing");
    }
    
    // Perform multiple health checks to ensure consistency
    let mut health_results = Vec::new();
    
    for i in 0..5 {
        let start_time = Instant::now();
        
        let result = timeout(HEALTH_TIMEOUT, client.check_connection()).await;
        
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Health check {} should not timeout", i);
        
        let is_healthy = result.unwrap().unwrap();
        health_results.push((is_healthy, elapsed));
        
        println!("Health check {}: healthy={}, time={:?}", i, is_healthy, elapsed);
        
        // Small delay between checks
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // All health checks should report the same status (healthy)
    let all_healthy = health_results.iter().all(|(healthy, _)| *healthy);
    assert!(all_healthy, "All health checks should report healthy status");
    
    // All health checks should be reasonably fast
    let all_fast = health_results.iter().all(|(_, elapsed)| *elapsed <= Duration::from_secs(10));
    assert!(all_fast, "All health checks should be fast");
    
    // Calculate average response time
    let avg_time = health_results.iter().map(|(_, elapsed)| *elapsed).sum::<Duration>() / health_results.len() as u32;
    println!("Average health check time: {:?}", avg_time);
    
    assert!(avg_time <= Duration::from_secs(5), 
           "Average health check time should be fast: {:?}", avg_time);
}

#[tokio::test]
#[serial]
async fn test_service_availability_detection() {
    // Test with known good service
    let good_client = create_test_client();
    
    // Wait for service to be ready
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), good_client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for availability testing");
    }
    
    // Test with known bad service
    let bad_client = SearXNGClient::new(Some("http://localhost:9998".to_string()));
    
    // Test both services
    let good_result = timeout(Duration::from_secs(10), good_client.check_connection()).await;
    let bad_result = timeout(Duration::from_secs(10), bad_client.check_connection()).await;
    
    // Good service should be available
    assert!(good_result.is_ok(), "Good service check should not timeout");
    assert!(good_result.unwrap().unwrap(), "Good service should be available");
    
    // Bad service should be detected as unavailable
    assert!(bad_result.is_ok(), "Bad service check should not timeout");
    assert!(!bad_result.unwrap().unwrap(), "Bad service should be detected as unavailable");
}

#[tokio::test]
#[serial]
async fn test_health_check_during_load() {
    let client = create_test_client();
    
    // Wait for service to be ready
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for load testing");
    }
    
    // Start some background search load
    let client_for_search = client.clone();
    let search_handle = tokio::spawn(async move {
        for i in 0..3 {
            let _ = client_for_search.search(&format!("test query {}", i), None, None, Some(2)).await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    
    // Perform health checks during load
    let mut health_results = Vec::new();
    
    for i in 0..6 {
        let start_time = Instant::now();
        
        let result = timeout(Duration::from_secs(15), client.check_connection()).await;
        
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Health check {} during load should not timeout", i);
        
        let is_healthy = result.unwrap().unwrap();
        health_results.push((is_healthy, elapsed));
        
        println!("Health check {} during load: healthy={}, time={:?}", i, is_healthy, elapsed);
        
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    // Wait for search load to complete
    let _ = search_handle.await;
    
    // Most health checks should succeed even under load
    let healthy_count = health_results.iter().filter(|(healthy, _)| *healthy).count();
    let total_checks = health_results.len();
    
    assert!(healthy_count >= total_checks * 2 / 3, 
           "At least 2/3 of health checks should succeed during load: {}/{}", 
           healthy_count, total_checks);
    
    // Health checks should still be reasonably fast even under load
    let avg_time = health_results.iter().map(|(_, elapsed)| *elapsed).sum::<Duration>() / health_results.len() as u32;
    println!("Average health check time during load: {:?}", avg_time);
    
    assert!(avg_time <= Duration::from_secs(8), 
           "Health checks should remain fast during load: {:?}", avg_time);
}

#[tokio::test]
#[serial]
async fn test_service_recovery_detection() {
    // This test simulates service recovery scenarios
    // In a real environment, you might stop and start the service
    
    let client = create_test_client();
    
    // First, verify service is running
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for recovery testing");
    }
    
    println!("Service is initially healthy");
    
    // Simulate checking an unhealthy service (using wrong port)
    let unhealthy_client = SearXNGClient::new(Some("http://localhost:9997".to_string()));
    
    let unhealthy_result = timeout(Duration::from_secs(10), unhealthy_client.check_connection()).await;
    assert!(unhealthy_result.is_ok(), "Unhealthy service check should complete");
    assert!(!unhealthy_result.unwrap().unwrap(), "Should detect unhealthy service");
    
    println!("Correctly detected unhealthy service");
    
    // Switch back to healthy service (simulating recovery)
    let recovered_result = timeout(Duration::from_secs(10), client.check_connection()).await;
    assert!(recovered_result.is_ok(), "Recovered service check should complete");
    assert!(recovered_result.unwrap().unwrap(), "Should detect recovered service");
    
    println!("Correctly detected service recovery");
}

#[tokio::test]
#[serial]
async fn test_health_metrics_collection() {
    let client = create_test_client();
    
    // Wait for service to be ready
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for metrics testing");
    }
    
    // Collect health check metrics
    let mut response_times = Vec::new();
    let mut success_count = 0;
    let total_checks = 10;
    
    for i in 0..total_checks {
        let start_time = Instant::now();
        
        let result = timeout(Duration::from_secs(10), client.check_connection()).await;
        
        let elapsed = start_time.elapsed();
        response_times.push(elapsed);
        
        if let Ok(Ok(true)) = result {
            success_count += 1;
        }
        
        println!("Health check {}: {:?}", i, elapsed);
        
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Calculate metrics
    let success_rate = success_count as f64 / total_checks as f64;
    let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let min_response_time = response_times.iter().min().unwrap();
    let max_response_time = response_times.iter().max().unwrap();
    
    println!("Health Check Metrics:");
    println!("  Success Rate: {:.1}%", success_rate * 100.0);
    println!("  Average Response Time: {:?}", avg_response_time);
    println!("  Min Response Time: {:?}", min_response_time);
    println!("  Max Response Time: {:?}", max_response_time);
    
    // Validate metrics
    assert!(success_rate >= 0.9, "Success rate should be at least 90%: {:.1}%", success_rate * 100.0);
    assert!(avg_response_time <= Duration::from_secs(5), "Average response time should be reasonable: {:?}", avg_response_time);
    assert!(*max_response_time <= Duration::from_secs(8), "Max response time should be reasonable: {:?}", max_response_time);
}

/// Test to verify that health checks work with different base URLs
#[tokio::test]
#[serial]
async fn test_health_check_url_variations() {
    // Test different URL formats that should work
    let valid_urls = vec![
        "http://localhost:8080",
        "http://localhost:8080/",
        "http://127.0.0.1:8080",
    ];
    
    // Wait for any service to be ready first
    let test_client = create_test_client();
    let mut service_ready = false;
    for _ in 0..30 {
        if let Ok(Ok(true)) = timeout(Duration::from_secs(5), test_client.check_connection()).await {
            service_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    if !service_ready {
        panic!("SearXNG service is not available for URL variation testing");
    }
    
    for url in valid_urls {
        let client = SearXNGClient::new(Some(url.to_string()));
        
        let result = timeout(Duration::from_secs(10), client.check_connection()).await;
        
        assert!(result.is_ok(), "Health check should not timeout for URL: {}", url);
        assert!(result.unwrap().unwrap(), "Should be healthy for URL: {}", url);
        
        println!("Health check successful for URL: {}", url);
    }
    
    // Test invalid URLs that should fail quickly
    let invalid_urls = vec![
        "http://localhost:9999",
        "http://invalid-host:8080",
        "http://localhost:8081",
    ];
    
    for url in invalid_urls {
        let client = SearXNGClient::new(Some(url.to_string()));
        
        let start_time = Instant::now();
        let result = timeout(Duration::from_secs(10), client.check_connection()).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Health check should complete for invalid URL: {}", url);
        assert!(!result.unwrap().unwrap(), "Should detect unhealthy for invalid URL: {}", url);
        
        // Should fail relatively quickly
        assert!(elapsed <= Duration::from_secs(8), 
               "Health check for invalid URL should fail quickly: {} took {:?}", url, elapsed);
        
        println!("Health check correctly failed for invalid URL: {} in {:?}", url, elapsed);
    }
}
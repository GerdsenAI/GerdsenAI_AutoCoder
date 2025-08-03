use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub engine: String,
    pub score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

/// Health monitoring configuration for SearXNG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearXNGHealthConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub max_retry_attempts: u32,
    pub retry_backoff_seconds: u64,
    pub auto_reconnect: bool,
    pub graceful_degradation: bool,
}

impl Default for SearXNGHealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60, // Check every minute (less frequent than Ollama)
            timeout_seconds: 15,        // Longer timeout for web service
            max_retry_attempts: 2,      // Fewer retries since it's optional
            retry_backoff_seconds: 3,   // Longer backoff
            auto_reconnect: true,       // Enable auto-reconnect
            graceful_degradation: true, // Enable graceful degradation
        }
    }
}

/// Health monitoring statistics for SearXNG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearXNGHealthStats {
    pub is_healthy: bool,
    pub is_degraded: bool, // Service available but degraded
    pub last_check_time: Option<u64>,
    pub last_successful_check: Option<u64>,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub total_failures: u64,
    pub average_response_time_ms: f64,
    pub degradation_reason: Option<String>,
}

impl Default for SearXNGHealthStats {
    fn default() -> Self {
        Self {
            is_healthy: false,
            is_degraded: false,
            last_check_time: None,
            last_successful_check: None,
            consecutive_failures: 0,
            total_checks: 0,
            total_failures: 0,
            average_response_time_ms: 0.0,
            degradation_reason: None,
        }
    }
}

/// Health monitoring for SearXNG with graceful degradation
pub struct SearXNGHealthMonitor {
    config: SearXNGHealthConfig,
    stats: Arc<Mutex<SearXNGHealthStats>>,
    is_monitoring: Arc<AtomicBool>,
}

impl SearXNGHealthMonitor {
    pub fn new(config: SearXNGHealthConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(SearXNGHealthStats::default())),
            is_monitoring: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get current health statistics
    pub async fn get_stats(&self) -> SearXNGHealthStats {
        self.stats.lock().await.clone()
    }

    /// Record a health check result with graceful degradation logic
    pub async fn record_check(&self, success: bool, response_time: Duration, error: Option<&str>) {
        let mut stats = self.stats.lock().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        stats.last_check_time = Some(now);
        stats.total_checks += 1;

        if success {
            stats.is_healthy = true;
            stats.is_degraded = false;
            stats.last_successful_check = Some(now);
            stats.consecutive_failures = 0;
            stats.degradation_reason = None;
        } else {
            stats.consecutive_failures += 1;
            stats.total_failures += 1;

            // Implement graceful degradation logic
            if self.config.graceful_degradation {
                if stats.consecutive_failures >= self.config.max_retry_attempts {
                    stats.is_healthy = false;
                    stats.is_degraded = true;
                    stats.degradation_reason = Some(format!(
                        "Service unavailable: {}",
                        error.unwrap_or("Connection failed")
                    ));
                } else {
                    // Still consider it healthy but degraded for the first few failures
                    stats.is_healthy = true;
                    stats.is_degraded = true;
                    stats.degradation_reason = Some("Intermittent connectivity issues".to_string());
                }
            } else {
                stats.is_healthy = false;
                stats.is_degraded = false;
            }
        }

        // Update running average response time
        let total_checks = stats.total_checks as f64;
        let new_time = response_time.as_millis() as f64;
        stats.average_response_time_ms = 
            ((stats.average_response_time_ms * (total_checks - 1.0)) + new_time) / total_checks;
    }

    /// Check if service should be considered available (healthy or degraded but functional)
    pub async fn is_available(&self) -> bool {
        let stats = self.stats.lock().await;
        if self.config.graceful_degradation {
            stats.is_healthy || (stats.is_degraded && stats.consecutive_failures < self.config.max_retry_attempts * 2)
        } else {
            stats.is_healthy
        }
    }

    /// Check if service is in degraded state
    pub async fn is_degraded(&self) -> bool {
        let stats = self.stats.lock().await;
        stats.is_degraded
    }
}

#[derive(Clone)]
pub struct SearXNGClient {
    base_url: Arc<Mutex<String>>,
    client: Client,
    default_engines: Arc<Mutex<Vec<String>>>,
    health_monitor: Arc<SearXNGHealthMonitor>,
}

impl SearXNGClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self::new_with_health_config(base_url, SearXNGHealthConfig::default())
    }

    pub fn new_with_health_config(base_url: Option<String>, health_config: SearXNGHealthConfig) -> Self {
        let health_monitor = Arc::new(SearXNGHealthMonitor::new(health_config));
        
        Self {
            base_url: Arc::new(Mutex::new(
                base_url.unwrap_or_else(|| "http://localhost:8080".to_string())
            )),
            client: Client::new(),
            default_engines: Arc::new(Mutex::new(vec![
                "github".to_string(),
                "stackoverflow".to_string(),
                "google".to_string(),
                "duckduckgo".to_string(),
            ])),
            health_monitor,
        }
    }

    pub async fn set_base_url(&self, base_url: String) {
        let mut url = self.base_url.lock().await;
        *url = base_url;
    }

    pub async fn get_base_url(&self) -> String {
        self.base_url.lock().await.clone()
    }

    pub async fn set_default_engines(&self, engines: Vec<String>) {
        let mut default_engines = self.default_engines.lock().await;
        *default_engines = engines;
    }

    pub async fn get_default_engines(&self) -> Vec<String> {
        self.default_engines.lock().await.clone()
    }

    pub async fn search(
        &self,
        query: &str,
        engines: Option<Vec<String>>,
        categories: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send>> {
        let base_url = self.base_url.lock().await.clone();
        let url = format!("{}/search", base_url);
        
        let default_engines = self.default_engines.lock().await.clone();
        let engines_str = engines
            .unwrap_or(default_engines)
            .join(",");
            
        let categories_str = categories
            .unwrap_or_else(|| vec!["general".to_string(), "it".to_string()])
            .join(",");
            
        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("format", "json");
        params.insert("engines", &engines_str);
        params.insert("categories", &categories_str);
        
        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.insert("limit", &limit_str);
        }
        
        let response = self.client.get(&url)
            .query(&params)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
            
        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to search: {}", response.status()))) as Box<dyn Error + Send>);
        }
        
        let search_response: serde_json::Value = response.json().await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
        
        // Parse the SearXNG response format
        let results = if let Some(results) = search_response.get("results").and_then(|r| r.as_array()) {
            results
                .iter()
                .map(|result| {
                    let title = result.get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("Untitled")
                        .to_string();
                        
                    let url = result.get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                        
                    let content = result.get("content")
                        .and_then(|c| c.as_str())
                        .or_else(|| result.get("snippet").and_then(|s| s.as_str()))
                        .unwrap_or("")
                        .to_string();
                        
                    let engine = result.get("engine")
                        .and_then(|e| e.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                        
                    let score = result.get("score")
                        .and_then(|s| s.as_f64())
                        .map(|s| s as f32);
                        
                    SearchResult {
                        title,
                        url,
                        content,
                        engine,
                        score,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        
        Ok(results)
    }

    /// Enhanced connection check with health monitoring and graceful degradation
    pub async fn check_connection(&self) -> Result<bool, Box<dyn Error + Send>> {
        self.check_connection_with_retry().await
    }

    /// Check connection with automatic retry and health monitoring
    pub async fn check_connection_with_retry(&self) -> Result<bool, Box<dyn Error + Send>> {
        let start_time = Instant::now();
        let mut last_error_msg = None;
        
        for attempt in 1..=self.health_monitor.config.max_retry_attempts {
            match self.perform_health_check().await {
                Ok(is_healthy) => {
                    let response_time = start_time.elapsed();
                    self.health_monitor.record_check(is_healthy, response_time, None).await;
                    return Ok(is_healthy);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    last_error_msg = Some(error_msg.clone());
                    
                    // If this wasn't the last attempt, wait before retrying
                    if attempt < self.health_monitor.config.max_retry_attempts {
                        let backoff_duration = Duration::from_secs(
                            self.health_monitor.config.retry_backoff_seconds * attempt as u64
                        );
                        tokio::time::sleep(backoff_duration).await;
                    }
                }
            }
        }
        
        // All attempts failed
        let response_time = start_time.elapsed();
        self.health_monitor.record_check(false, response_time, last_error_msg.as_deref()).await;
        
        // With graceful degradation, we might still be "available" even if unhealthy
        if self.health_monitor.config.graceful_degradation {
            Ok(self.health_monitor.is_available().await)
        } else {
            Ok(false)
        }
    }

    /// Perform actual health check against SearXNG API
    async fn perform_health_check(&self) -> Result<bool, Box<dyn Error + Send>> {
        let base_url = self.base_url.lock().await.clone();
        let url = format!("{}/healthz", base_url);
        let timeout_duration = Duration::from_secs(self.health_monitor.config.timeout_seconds);
        
        let response = match tokio::time::timeout(
            timeout_duration,
            self.client.get(&url).send()
        ).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(Box::new(e)),
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timed out")))
        };
        
        Ok(response.status().is_success())
    }

    /// Get health monitoring statistics
    pub async fn get_health_stats(&self) -> SearXNGHealthStats {
        self.health_monitor.get_stats().await
    }

    /// Check if the service is currently available (healthy or degraded but functional)
    pub async fn is_available(&self) -> bool {
        self.health_monitor.is_available().await
    }

    /// Check if the service is in degraded state
    pub async fn is_degraded(&self) -> bool {
        self.health_monitor.is_degraded().await
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) {
        if self.health_monitor.is_monitoring.swap(true, Ordering::SeqCst) {
            return; // Already monitoring
        }

        let client = self.clone();
        let health_monitor = self.health_monitor.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(health_monitor.config.check_interval_seconds)
            );
            
            while health_monitor.is_monitoring.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Perform health check
                let _ = client.check_connection_with_retry().await;
            }
        });
    }

    /// Stop background health monitoring
    pub fn stop_health_monitoring(&self) {
        self.health_monitor.is_monitoring.store(false, Ordering::SeqCst);
    }

    /// Execute search with graceful degradation
    pub async fn search_with_fallback(&self, query: &str, engines: Option<Vec<String>>) -> Result<SearchResult, Box<dyn Error + Send>> {
        // Check if service is available
        if !self.is_available().await {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "SearXNG service is currently unavailable")));
        }

        // If service is degraded, provide degraded response
        if self.is_degraded().await {
            let health_stats = self.get_health_stats().await;
            let degradation_reason = health_stats.degradation_reason
                .unwrap_or_else(|| "Service is experiencing issues".to_string());
            
            // Return a fallback result indicating degraded service
            return Ok(SearchResult {
                title: "Search Service Degraded".to_string(),
                url: "".to_string(),
                content: format!(
                    "Search functionality is currently limited due to service issues: {}. Please try again later.",
                    degradation_reason
                ),
                engine: "fallback".to_string(),
                score: Some(0.0),
            });
        }

        // Service is healthy, perform normal search
        let results = self.search(query, engines, None, None).await?;
        if let Some(first_result) = results.first() {
            Ok(first_result.clone())
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "No search results found")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    
    #[tokio::test]
    async fn test_search() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"query":"rust","results":[{"title":"Rust Programming Language","url":"https://www.rust-lang.org/","content":"A language empowering everyone to build reliable and efficient software.","engine":"google","score":0.95}]}"#)
            .create();
            
        let client = SearXNGClient::new(Some(server.url()));
        let results = client.search("rust", None, None, None).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].engine, "google");
        
        mock.assert();
    }
    
    #[tokio::test]
    async fn test_check_connection() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/healthz")
            .with_status(200)
            .create();
            
        let client = SearXNGClient::new(Some(server.url()));
        let is_connected = client.check_connection().await.unwrap();
        
        assert!(is_connected);
        
        mock.assert();
    }
}

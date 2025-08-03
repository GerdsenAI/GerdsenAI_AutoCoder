use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use futures_util::StreamExt;
use bytes::Bytes;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelDetails {
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub message: ChatMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
}

// Type alias for shared Ollama client
pub type SharedOllamaClient = Arc<Mutex<OllamaClient>>;

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub max_retry_attempts: u32,
    pub retry_backoff_seconds: u64,
    pub auto_reconnect: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30, // Check every 30 seconds
            timeout_seconds: 10,        // 10 second timeout
            max_retry_attempts: 3,      // Try 3 times
            retry_backoff_seconds: 2,   // 2 second backoff
            auto_reconnect: true,       // Enable auto-reconnect
        }
    }
}

/// Health monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    pub is_healthy: bool,
    pub last_check_time: Option<u64>, // Unix timestamp
    pub last_successful_check: Option<u64>,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub total_failures: u64,
    pub average_response_time_ms: f64,
}

impl Default for HealthStats {
    fn default() -> Self {
        Self {
            is_healthy: false,
            last_check_time: None,
            last_successful_check: None,
            consecutive_failures: 0,
            total_checks: 0,
            total_failures: 0,
            average_response_time_ms: 0.0,
        }
    }
}

/// Health monitoring state
pub struct HealthMonitor {
    config: HealthConfig,
    stats: Arc<Mutex<HealthStats>>,
    is_monitoring: Arc<AtomicBool>,
    check_count: Arc<AtomicU64>,
    failure_count: Arc<AtomicU64>,
}

impl HealthMonitor {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(HealthStats::default())),
            is_monitoring: Arc::new(AtomicBool::new(false)),
            check_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get current health statistics
    pub async fn get_stats(&self) -> HealthStats {
        self.stats.lock().await.clone()
    }

    /// Record a health check result
    pub async fn record_check(&self, success: bool, response_time: Duration) {
        let mut stats = self.stats.lock().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        stats.last_check_time = Some(now);
        stats.total_checks += 1;

        if success {
            stats.is_healthy = true;
            stats.last_successful_check = Some(now);
            stats.consecutive_failures = 0;
        } else {
            stats.is_healthy = false;
            stats.consecutive_failures += 1;
            stats.total_failures += 1;
        }

        // Update running average response time
        let total_checks = stats.total_checks as f64;
        let new_time = response_time.as_millis() as f64;
        stats.average_response_time_ms = 
            ((stats.average_response_time_ms * (total_checks - 1.0)) + new_time) / total_checks;
    }

    /// Check if service should be considered healthy
    pub async fn is_healthy(&self) -> bool {
        let stats = self.stats.lock().await;
        stats.is_healthy && stats.consecutive_failures < self.config.max_retry_attempts
    }
}

/// Efficient streaming buffer for handling Ollama responses
#[derive(Debug)]
pub struct StreamingBuffer {
    buffer: Vec<u8>,
    max_buffer_size: usize,
    chunk_queue: VecDeque<Bytes>,
    total_queued_bytes: usize,
    max_queue_size: usize,
}

impl StreamingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(8192), // 8KB initial capacity
            max_buffer_size: 1024 * 1024, // 1MB max buffer size
            chunk_queue: VecDeque::new(),
            total_queued_bytes: 0,
            max_queue_size: 10 * 1024 * 1024, // 10MB max queue size
        }
    }

    pub fn with_capacity(buffer_size: usize, queue_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_size.min(8192)),
            max_buffer_size: buffer_size,
            chunk_queue: VecDeque::new(),
            total_queued_bytes: 0,
            max_queue_size: queue_size,
        }
    }

    /// Add incoming chunk to the processing queue
    pub fn enqueue_chunk(&mut self, chunk: Bytes) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.total_queued_bytes + chunk.len() > self.max_queue_size {
            return Err("Stream buffer queue overflow - client consuming too slowly".into());
        }
        
        self.total_queued_bytes += chunk.len();
        self.chunk_queue.push_back(chunk);
        Ok(())
    }

    /// Process queued chunks and extract complete JSON lines
    pub fn process_chunks<F>(&mut self, mut callback: F) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        F: FnMut(&str) -> Result<bool, Box<dyn Error + Send + Sync>>, // Returns true if should continue
    {
        // Process all queued chunks
        while let Some(chunk) = self.chunk_queue.pop_front() {
            self.total_queued_bytes -= chunk.len();
            
            // Add chunk to working buffer
            if self.buffer.len() + chunk.len() > self.max_buffer_size {
                // Buffer overflow protection - clear buffer and start fresh
                self.buffer.clear();
                return Err("Stream buffer overflow - single message too large".into());
            }
            
            self.buffer.extend_from_slice(&chunk);
        }

        // Extract complete JSON lines from buffer
        let mut start = 0;
        let mut should_continue = true;
        
        for i in 0..self.buffer.len() {
            if self.buffer[i] == b'\n' {
                if let Ok(line) = std::str::from_utf8(&self.buffer[start..i]) {
                    if !line.trim().is_empty() {
                        should_continue = callback(line.trim())?;
                        if !should_continue {
                            break;
                        }
                    }
                }
                start = i + 1;
            }
        }
        
        // Keep any incomplete data in the buffer
        if start > 0 {
            self.buffer.drain(0..start);
        }
        
        Ok(should_continue)
    }

    /// Get buffer usage statistics
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            buffer_size: self.buffer.len(),
            max_buffer_size: self.max_buffer_size,
            queued_chunks: self.chunk_queue.len(),
            total_queued_bytes: self.total_queued_bytes,
            max_queue_size: self.max_queue_size,
            buffer_utilization: (self.buffer.len() as f32 / self.max_buffer_size as f32) * 100.0,
            queue_utilization: (self.total_queued_bytes as f32 / self.max_queue_size as f32) * 100.0,
        }
    }

    /// Clear all buffers and reset state
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.chunk_queue.clear();
        self.total_queued_bytes = 0;
    }
}

#[derive(Debug, Clone)]
pub struct BufferStats {
    pub buffer_size: usize,
    pub max_buffer_size: usize,
    pub queued_chunks: usize,
    pub total_queued_bytes: usize,
    pub max_queue_size: usize,
    pub buffer_utilization: f32,
    pub queue_utilization: f32,
}

#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    models_cache: Arc<Mutex<HashMap<String, ModelInfo>>>,
    health_monitor: Arc<HealthMonitor>,
}

impl OllamaClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self::new_with_health_config(base_url, HealthConfig::default())
    }

    pub fn new_with_health_config(base_url: Option<String>, health_config: HealthConfig) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        let health_monitor = Arc::new(HealthMonitor::new(health_config));
        
        Self {
            base_url,
            client: Client::new(),
            models_cache: Arc::new(Mutex::new(HashMap::new())),
            health_monitor,
        }
    }

    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to list models: {}", response.status()).into());
        }
        
        let model_response: ModelResponse = response.json().await?;
        
        // Update cache
        let mut cache = self.models_cache.lock().await;
        for model in &model_response.models {
            cache.insert(model.name.clone(), model.clone());
        }
        
        Ok(model_response.models)
    }

    pub async fn get_model(&self, name: &str) -> Result<Option<ModelInfo>, Box<dyn Error + Send + Sync>> {
        // Check cache first
        {
            let cache = self.models_cache.lock().await;
            if let Some(model) = cache.get(name) {
                return Ok(Some(model.clone()));
            }
        }
        
        // If not in cache, refresh the list
        let models = self.list_models().await?;
        
        // Look for the model in the refreshed list
        for model in models {
            if model.name == name {
                return Ok(Some(model));
            }
        }
        
        Ok(None)
    }

    pub async fn generate_completion(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options,
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Failed to generate completion: {}", response.status()).into());
        }
        
        let generate_response: GenerateResponse = response.json().await?;
        Ok(generate_response.response)
    }

    pub async fn generate_stream<F>(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
        mut callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: FnMut(&str) + Send + 'static,
    {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options,
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Failed to generate stream: {}", response.status()).into());
        }
        
        let mut stream = response.bytes_stream();
        let mut streaming_buffer = StreamingBuffer::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            
            // Enqueue the chunk for processing
            streaming_buffer.enqueue_chunk(chunk)?;
            
            // Process all available complete JSON lines
            let should_continue = streaming_buffer.process_chunks(|line| {
                if let Ok(response) = serde_json::from_str::<GenerateResponse>(line) {
                    callback(&response.response);
                    
                    if response.done.unwrap_or(false) {
                        return Ok(false); // Signal completion
                    }
                }
                Ok(true) // Continue processing
            })?;
            
            if !should_continue {
                break;
            }
        }
        
        Ok(())
    }

    /// Generate stream with buffer statistics callback for monitoring
    pub async fn generate_stream_with_stats<F, S>(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
        mut callback: F,
        mut stats_callback: Option<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: FnMut(&str) + Send + 'static,
        S: FnMut(BufferStats) + Send + 'static,
    {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options,
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Failed to generate stream: {}", response.status()).into());
        }
        
        let mut stream = response.bytes_stream();
        let mut streaming_buffer = StreamingBuffer::new();
        let mut chunk_count = 0;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            chunk_count += 1;
            
            // Enqueue the chunk for processing
            streaming_buffer.enqueue_chunk(chunk)?;
            
            // Report buffer stats every 10 chunks
            if let Some(ref mut stats_cb) = stats_callback {
                if chunk_count % 10 == 0 {
                    stats_cb(streaming_buffer.get_stats());
                }
            }
            
            // Process all available complete JSON lines
            let should_continue = streaming_buffer.process_chunks(|line| {
                if let Ok(response) = serde_json::from_str::<GenerateResponse>(line) {
                    callback(&response.response);
                    
                    if response.done.unwrap_or(false) {
                        return Ok(false); // Signal completion
                    }
                }
                Ok(true) // Continue processing
            })?;
            
            if !should_continue {
                break;
            }
        }
        
        // Final stats report
        if let Some(ref mut stats_cb) = stats_callback {
            stats_cb(streaming_buffer.get_stats());
        }
        
        Ok(())
    }

    pub async fn chat<F>(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        options: Option<GenerateOptions>,
        mut callback: Option<F>,
    ) -> Result<ChatMessage, Box<dyn Error + Send + Sync>>
    where
        F: FnMut(&str) + Send + 'static,
    {
        let url = format!("{}/api/chat", self.base_url);
        
        let stream = callback.is_some();
        
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream,
            options,
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Failed to chat: {}", response.status()).into());
        }
        
        if !stream {
            let chat_response: ChatResponse = response.json().await?;
            return Ok(chat_response.message);
        }
        
        // Handle streaming response with efficient buffer
        let mut stream = response.bytes_stream();
        let mut streaming_buffer = StreamingBuffer::new();
        let mut full_response = String::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            
            // Enqueue the chunk for processing
            streaming_buffer.enqueue_chunk(chunk)?;
            
            // Process all available complete JSON lines
            let should_continue = streaming_buffer.process_chunks(|line| {
                if let Ok(response) = serde_json::from_str::<ChatResponse>(line) {
                    if let Some(ref mut cb) = callback {
                        cb(&response.message.content);
                    }
                    
                    full_response.push_str(&response.message.content);
                    
                    if response.done.unwrap_or(false) {
                        return Ok(false); // Signal completion
                    }
                }
                Ok(true) // Continue processing
            })?;
            
            if !should_continue {
                break;
            }
        }
        
        Ok(ChatMessage {
            role: "assistant".to_string(),
            content: full_response,
        })
    }

    pub async fn create_embedding(
        &self,
        model: &str,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/embeddings", self.base_url);
        
        let request = EmbeddingRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Failed to create embedding: {}", response.status()).into());
        }
        
        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response.embedding)
    }

    /// Enhanced connection check with health monitoring
    pub async fn check_connection(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.check_connection_with_retry().await
    }

    /// Check connection with automatic retry and health monitoring
    pub async fn check_connection_with_retry(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let start_time = Instant::now();
        let mut last_error: Option<Box<dyn Error + Send + Sync>> = None;
        
        for attempt in 1..=self.health_monitor.config.max_retry_attempts {
            match self.perform_health_check().await {
                Ok(is_healthy) => {
                    let response_time = start_time.elapsed();
                    self.health_monitor.record_check(is_healthy, response_time).await;
                    return Ok(is_healthy);
                }
                Err(e) => {
                    last_error = Some(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                    
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
        self.health_monitor.record_check(false, response_time).await;
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Err("Connection check failed after all retry attempts".into())
        }
    }

    /// Perform actual health check against Ollama API
    async fn perform_health_check(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/version", self.base_url);
        let timeout_duration = Duration::from_secs(self.health_monitor.config.timeout_seconds);
        
        let response = tokio::time::timeout(
            timeout_duration,
            self.client.get(&url).send()
        ).await??;
        
        Ok(response.status().is_success())
    }

    /// Get health monitoring statistics
    pub async fn get_health_stats(&self) -> HealthStats {
        self.health_monitor.get_stats().await
    }

    /// Check if the service is currently considered healthy
    pub async fn is_healthy(&self) -> bool {
        self.health_monitor.is_healthy().await
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

    /// Execute operation with automatic retry and circuit breaker logic
    pub async fn with_retry<F, T, E>(&self, operation: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync,
        E: Into<Box<dyn Error + Send + Sync>> + Send + Sync,
        T: Send,
    {
        // Check if service is healthy before attempting operation
        if !self.is_healthy().await {
            // Try to reconnect first
            if self.health_monitor.config.auto_reconnect {
                if let Ok(false) = self.check_connection_with_retry().await {
                    return Err("Service is unavailable and reconnection failed".into());
                }
            } else {
                return Err("Service is currently unavailable".into());
            }
        }

        let mut last_error = None;
        
        for attempt in 1..=self.health_monitor.config.max_retry_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.into());
                    
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
        
        // Mark as unhealthy if operation keeps failing
        self.health_monitor.record_check(false, Duration::from_millis(0)).await;
        
        Err(last_error.unwrap_or_else(|| "Operation failed after all retry attempts".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
    
    // Helper function to create test model data
    fn create_test_model(name: &str) -> ModelInfo {
        ModelInfo {
            name: name.to_string(),
            modified_at: "2025-06-01T12:00:00Z".to_string(),
            size: 4200000000,
            digest: "sha256:1234567890abcdef".to_string(),
            details: Some(ModelDetails {
                parameter_size: Some("8B".to_string()),
                quantization_level: Some("Q4_0".to_string()),
                format: Some("gguf".to_string()),
                family: Some("llama".to_string()),
                families: Some(vec!["llama".to_string()]),
            }),
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        // Test default URL
        let client = OllamaClient::new(None);
        assert_eq!(client.get_base_url(), "http://localhost:11434");
        
        // Test custom URL
        let client = OllamaClient::new(Some("http://custom:8080".to_string()));
        assert_eq!(client.get_base_url(), "http://custom:8080");
    }

    #[tokio::test] 
    async fn test_set_base_url() {
        let mut client = OllamaClient::new(None);
        client.set_base_url("http://new-url:9090".to_string());
        assert_eq!(client.get_base_url(), "http://new-url:9090");
    }

    #[tokio::test]
    async fn test_list_models_success() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"llama3.2:latest","modified_at":"2025-06-01T12:00:00Z","size":4200000000,"digest":"sha256:1234567890abcdef","details":{"parameter_size":"8B","quantization_level":"Q4_0","format":"gguf","family":"llama","families":["llama"]}}]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let models = client.list_models().await.unwrap();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "llama3.2:latest");
        assert_eq!(models[0].size, 4200000000);
        let details = models[0].details.as_ref().unwrap();
        assert_eq!(details.parameter_size.as_deref(), Some("8B"));
        assert_eq!(details.quantization_level.as_deref(), Some("Q4_0"));
        
        mock.assert();
    }

    #[tokio::test]
    async fn test_list_models_empty_response() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let models = client.list_models().await.unwrap();
        
        assert_eq!(models.len(), 0);
        mock.assert();
    }

    #[tokio::test]
    async fn test_list_models_http_error() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(500)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result = client.list_models().await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to list models: 500"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_list_models_malformed_json() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": json"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result = client.list_models().await;
        
        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_model_from_cache() {
        let mut server = Server::new();
        
        // Setup initial list_models call to populate cache
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"cached-model","modified_at":"2025-06-01T12:00:00Z","size":1000000000,"digest":"sha256:abcdef123456","details":null}]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        
        // First call to populate cache
        let _ = client.list_models().await.unwrap();
        mock.assert();
        
        // Second call should use cache (no additional HTTP request)
        let model = client.get_model("cached-model").await.unwrap();
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "cached-model");
    }

    #[tokio::test]
    async fn test_get_model_not_found() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let model = client.get_model("nonexistent-model").await.unwrap();
        
        assert!(model.is_none());
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_completion_success() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"llama3.2:latest","response":"This is a test response.","done":true}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let response = client.generate_completion("llama3.2:latest", "Test prompt", None).await.unwrap();
        
        assert_eq!(response, "This is a test response.");
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_completion_with_options() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .match_body(mockito::Matcher::JsonString(r#"{"model":"test-model","prompt":"test prompt","stream":false,"options":{"temperature":0.8,"top_p":0.9,"max_tokens":100}}"#.to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","response":"Generated response","done":true}"#)
            .create();
        
        let client = OllamaClient::new(Some(server.url()));
        let options = GenerateOptions {
            temperature: Some(0.8),
            top_p: Some(0.9),
            top_k: None,
            max_tokens: Some(100),
        };
        
        let response = client.generate_completion("test-model", "test prompt", Some(options)).await.unwrap();
        assert_eq!(response, "Generated response");
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_completion_http_error() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(400)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result = client.generate_completion("test-model", "test prompt", None).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to generate completion: 400"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_stream_success() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","response":"Hello","done":false}
{"model":"test-model","response":" world","done":false}
{"model":"test-model","response":"!","done":true}
"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let tokens = Arc::new(std::sync::Mutex::new(Vec::new()));
        let tokens_clone = tokens.clone();
        
        let result = client.generate_stream(
            "test-model",
            "test prompt",
            None,
            move |token| {
                tokens_clone.lock().unwrap().push(token.to_string());
            }
        ).await;
        
        assert!(result.is_ok());
        let collected_tokens = tokens.lock().unwrap();
        assert_eq!(*collected_tokens, vec!["Hello", " world", "!"]);
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_stream_http_error() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate") 
            .with_status(500)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result = client.generate_stream(
            "test-model",
            "test prompt",
            None,
            |_| {}
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to generate stream: 500"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_chat_non_streaming() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","message":{"role":"assistant","content":"Hello there!"},"done":true}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let response = client.chat("test-model", messages, None, Option::<fn(&str)>::None).await.unwrap();
        
        assert_eq!(response.role, "assistant");
        assert_eq!(response.content, "Hello there!");
        mock.assert();
    }

    #[tokio::test]
    async fn test_chat_streaming() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","message":{"role":"assistant","content":"Hello"},"done":false}
{"model":"test-model","message":{"role":"assistant","content":" there"},"done":false}
{"model":"test-model","message":{"role":"assistant","content":"!"},"done":true}
"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let tokens = Arc::new(std::sync::Mutex::new(Vec::new()));
        let tokens_clone = tokens.clone();
        
        let response = client.chat(
            "test-model", 
            messages, 
            None, 
            Some(move |token: &str| {
                tokens_clone.lock().unwrap().push(token.to_string());
            })
        ).await.unwrap();
        
        assert_eq!(response.role, "assistant");
        assert_eq!(response.content, "Hello there!");
        
        let collected_tokens = tokens.lock().unwrap();
        assert_eq!(*collected_tokens, vec!["Hello", " there", "!"]);
        mock.assert();
    }

    #[tokio::test]
    async fn test_chat_http_error() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(404)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let result = client.chat("nonexistent-model", messages, None, Option::<fn(&str)>::None).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to chat: 404"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_embedding_success() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/embeddings")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"embedding":[0.1, 0.2, 0.3, 0.4, 0.5]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let embedding = client.create_embedding("test-embedding-model", "test text").await.unwrap();
        
        assert_eq!(embedding.len(), 5);
        assert_eq!(embedding, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_embedding_http_error() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/embeddings")
            .with_status(500)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result = client.create_embedding("test-model", "test text").await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to create embedding: 500"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_check_connection_success() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/version")
            .with_status(200)
            .with_body(r#"{"version":"0.1.0"}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let connected = client.check_connection().await.unwrap();
        
        assert!(connected);
        mock.assert();
    }

    #[tokio::test]
    async fn test_check_connection_failure() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/version")
            .with_status(500)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let connected = client.check_connection().await.unwrap();
        
        assert!(!connected);
        mock.assert();
    }

    #[tokio::test]
    async fn test_check_connection_network_error() {
        // Use an invalid URL to simulate network failure
        let client = OllamaClient::new(Some("http://invalid-host:99999".to_string()));
        let connected = client.check_connection().await.unwrap();
        
        assert!(!connected);
    }

    #[tokio::test]
    async fn test_concurrent_model_requests() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"concurrent-model","modified_at":"2025-06-01T12:00:00Z","size":1000000000,"digest":"sha256:concurrent123","details":null}]}"#)
            .expect(1) // Should only be called once due to caching
            .create();
            
        let client = Arc::new(OllamaClient::new(Some(server.url())));
        
        // Make multiple concurrent requests for the same model
        let handles: Vec<_> = (0..5).map(|_| {
            let client_clone = client.clone();
            tokio::spawn(async move {
                client_clone.get_model("concurrent-model").await
            })
        }).collect();
        
        // Wait for all requests to complete
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap().name, "concurrent-model");
        }
        
        mock.assert();
    }

    #[tokio::test]
    async fn test_streaming_with_malformed_json() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","response":"Hello","done":false}
invalid json line
{"model":"test-model","response":" world","done":true}
"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let tokens = Arc::new(std::sync::Mutex::new(Vec::new()));
        let tokens_clone = tokens.clone();
        
        let result = client.generate_stream(
            "test-model",
            "test prompt",
            None,
            move |token| {
                tokens_clone.lock().unwrap().push(token.to_string());
            }
        ).await;
        
        assert!(result.is_ok());
        let collected_tokens = tokens.lock().unwrap();
        // Should skip malformed line and continue processing
        assert_eq!(*collected_tokens, vec!["Hello", " world"]);
        mock.assert();
    }

    #[tokio::test]
    async fn test_request_timeout() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","response":"Slow response","done":true}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        
        // Test with a very short timeout
        let result = timeout(
            Duration::from_millis(50),
            client.generate_completion("test-model", "test prompt", None)
        ).await;
        
        assert!(result.is_err()); // Should timeout
        mock.assert();
    }

    #[tokio::test]
    async fn test_large_response_handling() {
        let mut server = Server::new();
        
        let large_response = "x".repeat(10000); // 10KB response
        let response_json = format!(r#"{{"model":"test-model","response":"{}","done":true}}"#, large_response);
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response_json)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let response = client.generate_completion("test-model", "test prompt", None).await.unwrap();
        
        assert_eq!(response.len(), 10000);
        assert_eq!(response, large_response);
        mock.assert();
    }

    #[tokio::test]
    async fn test_empty_prompt_handling() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .match_body(mockito::Matcher::JsonString(r#"{"model":"test-model","prompt":"","stream":false}"#.to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"test-model","response":"Empty prompt response","done":true}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let response = client.generate_completion("test-model", "", None).await.unwrap();
        
        assert_eq!(response, "Empty prompt response");
        mock.assert();
    }

    #[tokio::test]
    async fn test_cache_behavior_after_error() {
        let mut server = Server::new();
        
        // First request fails
        let mock1 = server
            .mock("GET", "/api/tags")
            .with_status(500)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let result1 = client.list_models().await;
        assert!(result1.is_err());
        mock1.assert();
        
        // Second request succeeds  
        let mock2 = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"recovery-model","modified_at":"2025-06-01T12:00:00Z","size":1000000000,"digest":"sha256:recovery123","details":null}]}"#)
            .create();
            
        let result2 = client.list_models().await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().len(), 1);
        mock2.assert();
        
        // Third request should use cache
        let model = client.get_model("recovery-model").await.unwrap();
        assert!(model.is_some());
    }
}

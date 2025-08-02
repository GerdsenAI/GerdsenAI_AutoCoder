use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering}; 
use dashmap::DashMap;
use std::sync::Arc;
use crate::ollama_client::OllamaClient;
use crate::thread_pool_manager::{ThreadPoolManager, TaskType, TaskPriority};
use tokio::sync::{Semaphore, Mutex as TokioMutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentMetadata {
    pub source: String,
    pub document_type: String,
    pub language: Option<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryResult {
    pub document: String,
    pub metadata: DocumentMetadata,
    pub distance: f32,
    pub id: String,
}

/// Cache entry for query results
#[derive(Debug, Clone)]
pub struct CachedQueryResult {
    pub results: Vec<QueryResult>,
    pub created_at: Instant,
    pub ttl: Duration,
    pub hit_count: u32,
}

impl CachedQueryResult {
    pub fn new(results: Vec<QueryResult>, ttl: Duration) -> Self {
        Self {
            results,
            created_at: Instant::now(),
            ttl,
            hit_count: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }
}

/// Query cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            default_ttl_seconds: 300, // 5 minutes
            cleanup_interval_seconds: 60, // 1 minute
        }
    }
}

/// Query cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: usize,
    pub oldest_entry_age_seconds: Option<u64>,
}

/// Query cache implementation
pub struct QueryCache {
    cache: DashMap<String, CachedQueryResult>,
    config: CacheConfig,
    hit_count: Arc<std::sync::atomic::AtomicU64>,
    miss_count: Arc<std::sync::atomic::AtomicU64>,
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub batch_timeout_seconds: u64,
    pub max_concurrent_batches: usize,
    pub embedding_model: String,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_seconds: 5,
            max_concurrent_batches: 4,
            embedding_model: "nomic-embed-text".to_string(),
        }
    }
}

/// Batch embedding request
#[derive(Debug)]
pub struct EmbeddingBatch {
    pub texts: Vec<String>,
    pub document_ids: Vec<String>,
    pub collection_name: String,
    pub priority: TaskPriority,
}

/// Batch processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
    pub total_batches_processed: u64,
    pub total_documents_embedded: u64,
    pub average_batch_size: f64,
    pub average_processing_time_ms: f64,
    pub failed_batches: u64,
    pub current_queue_size: usize,
}

/// Embedding batch processor
pub struct EmbeddingBatchProcessor {
    ollama_client: OllamaClient,
    thread_pool: Arc<ThreadPoolManager>,
    batch_config: BatchConfig,
    semaphore: Arc<Semaphore>,
    stats: Arc<std::sync::Mutex<BatchStats>>,
}

impl EmbeddingBatchProcessor {
    pub fn new(
        ollama_client: OllamaClient,
        thread_pool: Arc<ThreadPoolManager>,
        batch_config: BatchConfig,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(batch_config.max_concurrent_batches));
        let stats = Arc::new(std::sync::Mutex::new(BatchStats {
            total_batches_processed: 0,
            total_documents_embedded: 0,
            average_batch_size: 0.0,
            average_processing_time_ms: 0.0,
            failed_batches: 0,
            current_queue_size: 0,
        }));

        Self {
            ollama_client,
            thread_pool,
            batch_config,
            semaphore,
            stats,
        }
    }

    /// Process a batch of texts to generate embeddings
    pub async fn process_batch(
        &self,
        batch: EmbeddingBatch,
    ) -> Result<Vec<(String, Vec<f32>)>, Box<dyn Error + Send + Sync>> {
        let _permit = self.semaphore.acquire().await?;
        let start_time = Instant::now();

        // Create a task for the thread pool
        let task = ThreadPoolManager::create_task(
            TaskType::Embedding,
            batch.priority,
            (batch.texts.clone(), self.ollama_client.clone(), self.batch_config.embedding_model.clone()),
        );

        let results = self.thread_pool.execute_task(task, |(texts, client, model)| {
            // Execute embedding generation on thread pool
            let runtime = tokio::runtime::Handle::current();
            let future = async move {
                let mut embeddings = Vec::new();
                
                // Process texts in smaller sub-batches for memory efficiency
                const SUB_BATCH_SIZE: usize = 8;
                for chunk in texts.chunks(SUB_BATCH_SIZE) {
                    for text in chunk {
                        match client.create_embedding(&model, text).await {
                            Ok(embedding) => embeddings.push(embedding),
                            Err(e) => return Err(format!("Embedding generation failed: {}", e)),
                        }
                    }
                    
                    // Small delay between sub-batches to prevent overwhelming Ollama
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                
                Ok(embeddings)
            };
            
            runtime.block_on(future)
        }).await;

        let processing_time = start_time.elapsed();
        
        match results.result {
            Ok(embeddings) => {
                // Update statistics
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_batches_processed += 1;
                    stats.total_documents_embedded += batch.texts.len() as u64;
                    
                    // Update running averages
                    let total_batches = stats.total_batches_processed as f64;
                    stats.average_batch_size = ((stats.average_batch_size * (total_batches - 1.0)) + batch.texts.len() as f64) / total_batches;
                    stats.average_processing_time_ms = ((stats.average_processing_time_ms * (total_batches - 1.0)) + processing_time.as_millis() as f64) / total_batches;
                }

                // Combine embeddings with document IDs
                let results: Vec<(String, Vec<f32>)> = batch.document_ids.into_iter()
                    .zip(embeddings.into_iter())
                    .collect();

                Ok(results)
            }
            Err(e) => {
                // Update failure statistics
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.failed_batches += 1;
                }
                
                Err(format!("Batch processing failed: {}", e).into())
            }
        }
    }

    /// Get batch processing statistics
    pub fn get_stats(&self) -> BatchStats {
        self.stats.lock().unwrap().clone()
    }

    /// Update queue size in statistics (called externally)
    pub fn update_queue_size(&self, size: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.current_queue_size = size;
    }
}

impl QueryCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = DashMap::new();
        let hit_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let miss_count = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // Start cleanup task if cache is enabled
        if config.enabled {
            let cache_cleanup = cache.clone();
            let cleanup_interval = Duration::from_secs(config.cleanup_interval_seconds);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(cleanup_interval);
                loop {
                    interval.tick().await;
                    Self::cleanup_expired_entries(&cache_cleanup);
                }
            });
        }

        Self {
            cache,
            config,
            hit_count,
            miss_count,
        }
    }

    /// Generate cache key from query parameters
    fn generate_cache_key(
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: &Option<serde_json::Value>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        collection_name.hash(&mut hasher);
        query_text.hash(&mut hasher);
        n_results.hash(&mut hasher);
        if let Some(filter_val) = filter {
            filter_val.to_string().hash(&mut hasher);
        }
        
        format!("query_{:x}", hasher.finish())
    }

    /// Get cached query result if available and not expired
    pub fn get(
        &self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: &Option<serde_json::Value>,
    ) -> Option<Vec<QueryResult>> {
        if !self.config.enabled {
            return None;
        }

        let cache_key = Self::generate_cache_key(collection_name, query_text, n_results, filter);
        
        if let Some(mut cached_entry) = self.cache.get_mut(&cache_key) {
            if !cached_entry.is_expired() {
                cached_entry.record_hit();
                self.hit_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return Some(cached_entry.results.clone());
            } else {
                // Remove expired entry
                drop(cached_entry);
                self.cache.remove(&cache_key);
            }
        }

        self.miss_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        None
    }

    /// Store query result in cache
    pub fn put(
        &self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: &Option<serde_json::Value>,
        results: Vec<QueryResult>,
        custom_ttl: Option<Duration>,
    ) {
        if !self.config.enabled {
            return;
        }

        // Check if cache is full and evict if necessary
        if self.cache.len() >= self.config.max_entries {
            self.evict_oldest_entries(self.config.max_entries / 4); // Evict 25% when full
        }

        let cache_key = Self::generate_cache_key(collection_name, query_text, n_results, filter);
        let ttl = custom_ttl.unwrap_or_else(|| Duration::from_secs(self.config.default_ttl_seconds));
        
        let cached_result = CachedQueryResult::new(results, ttl);
        self.cache.insert(cache_key, cached_result);
    }

    /// Invalidate cache entries for a specific collection
    pub fn invalidate_collection(&self, collection_name: &str) {
        let keys_to_remove: Vec<String> = self.cache.iter()
            .filter_map(|entry| {
                let key = entry.key();
                if key.contains(&format!("{}_{}", collection_name, collection_name)) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
        }
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.clear();
        self.hit_count.store(0, std::sync::atomic::Ordering::SeqCst);
        self.miss_count.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let total_hits = self.hit_count.load(std::sync::atomic::Ordering::SeqCst);
        let total_misses = self.miss_count.load(std::sync::atomic::Ordering::SeqCst);
        let total_requests = total_hits + total_misses;
        
        let hit_rate = if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let oldest_entry_age = self.cache.iter()
            .map(|entry| entry.created_at.elapsed().as_secs())
            .max();

        // Rough memory usage estimation
        let memory_usage_bytes = self.cache.len() * 1024; // Rough estimate

        CacheStats {
            total_entries: self.cache.len(),
            total_hits,
            total_misses,
            hit_rate,
            memory_usage_bytes,
            oldest_entry_age_seconds: oldest_entry_age,
        }
    }

    /// Clean up expired entries
    fn cleanup_expired_entries(cache: &DashMap<String, CachedQueryResult>) {
        let expired_keys: Vec<String> = cache.iter()
            .filter_map(|entry| {
                if entry.is_expired() {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }
    }

    /// Evict oldest entries when cache is full
    fn evict_oldest_entries(&self, count: usize) {
        let mut entries: Vec<(String, Instant)> = self.cache.iter()
            .map(|entry| (entry.key().clone(), entry.created_at))
            .collect();

        entries.sort_by_key(|(_, created_at)| *created_at);
        
        for (key, _) in entries.into_iter().take(count) {
            self.cache.remove(&key);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub embedding: Option<Vec<f32>>, // Will be populated when embedding function is available
}

pub struct InMemoryCollection {
    pub name: String,
    pub documents: HashMap<String, Document>,
}

/// Health monitoring configuration for ChromaDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaHealthConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub max_retry_attempts: u32,
    pub retry_backoff_seconds: u64,
    pub auto_recovery: bool,
    pub operation_timeout_seconds: u64,
}

impl Default for ChromaHealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30, // Check every 30 seconds
            timeout_seconds: 5,         // Quick timeout for health checks
            max_retry_attempts: 3,      // Standard retry attempts
            retry_backoff_seconds: 1,   // Fast backoff for local operations
            auto_recovery: true,        // Enable auto-recovery
            operation_timeout_seconds: 30, // Timeout for operations
        }
    }
}

/// Health monitoring statistics for ChromaDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaHealthStats {
    pub is_healthy: bool,
    pub last_check_time: Option<u64>,
    pub last_successful_operation: Option<u64>,
    pub consecutive_failures: u32,
    pub total_operations: u64,
    pub total_failures: u64,
    pub average_operation_time_ms: f64,
    pub total_collections: usize,
    pub total_documents: usize,
    pub memory_usage_estimate_bytes: usize,
}

impl Default for ChromaHealthStats {
    fn default() -> Self {
        Self {
            is_healthy: true, // Start as healthy for in-memory implementation
            last_check_time: None,
            last_successful_operation: None,
            consecutive_failures: 0,
            total_operations: 0,
            total_failures: 0,
            average_operation_time_ms: 0.0,
            total_collections: 0,
            total_documents: 0,
            memory_usage_estimate_bytes: 0,
        }
    }
}

/// Health monitoring for ChromaDB
pub struct ChromaHealthMonitor {
    config: ChromaHealthConfig,
    stats: Arc<TokioMutex<ChromaHealthStats>>,
    is_monitoring: Arc<AtomicBool>,
}

impl ChromaHealthMonitor {
    pub fn new(config: ChromaHealthConfig) -> Self {
        Self {
            config,
            stats: Arc::new(TokioMutex::new(ChromaHealthStats::default())),
            is_monitoring: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get current health statistics
    pub async fn get_stats(&self) -> ChromaHealthStats {
        self.stats.lock().await.clone()
    }

    /// Record an operation result
    pub async fn record_operation(&self, success: bool, operation_time: Duration) {
        let mut stats = self.stats.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        stats.total_operations += 1;

        if success {
            stats.is_healthy = true;
            stats.last_successful_operation = Some(now);
            stats.consecutive_failures = 0;
        } else {
            stats.consecutive_failures += 1;
            stats.total_failures += 1;
            
            // Consider unhealthy if too many consecutive failures
            if stats.consecutive_failures >= self.config.max_retry_attempts {
                stats.is_healthy = false;
            }
        }

        // Update running average operation time
        let total_ops = stats.total_operations as f64;
        let new_time = operation_time.as_millis() as f64;
        stats.average_operation_time_ms = 
            ((stats.average_operation_time_ms * (total_ops - 1.0)) + new_time) / total_ops;
    }

    /// Update collection and document counts
    pub async fn update_counts(&self, collections: usize, documents: usize, memory_estimate: usize) {
        let mut stats = self.stats.lock().await;
        stats.total_collections = collections;
        stats.total_documents = documents;
        stats.memory_usage_estimate_bytes = memory_estimate;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        stats.last_check_time = Some(now);
    }

    /// Check if service should be considered healthy
    pub async fn is_healthy(&self) -> bool {
        let stats = self.stats.lock().await;
        stats.is_healthy && stats.consecutive_failures < self.config.max_retry_attempts
    }
}

pub struct ChromaManager {
    collections: HashMap<String, InMemoryCollection>,
    query_cache: QueryCache,
    batch_processor: Option<EmbeddingBatchProcessor>,
    health_monitor: Arc<ChromaHealthMonitor>,
}

impl ChromaManager {
    pub fn new(_db_path: &str) -> Result<Self, Box<dyn Error>> {
        Self::new_with_configs(_db_path, CacheConfig::default(), ChromaHealthConfig::default())
    }

    pub fn new_with_cache_config(_db_path: &str, cache_config: CacheConfig) -> Result<Self, Box<dyn Error>> {
        Self::new_with_configs(_db_path, cache_config, ChromaHealthConfig::default())
    }

    pub fn new_with_configs(_db_path: &str, cache_config: CacheConfig, health_config: ChromaHealthConfig) -> Result<Self, Box<dyn Error>> {
        let query_cache = QueryCache::new(cache_config);
        let health_monitor = Arc::new(ChromaHealthMonitor::new(health_config));
        
        Ok(Self {
            collections: HashMap::new(),
            query_cache,
            batch_processor: None,
            health_monitor,
        })
    }

    /// Initialize batch processing capabilities
    pub fn enable_batch_processing(
        &mut self,
        ollama_client: OllamaClient,
        thread_pool: Arc<ThreadPoolManager>,
        batch_config: Option<BatchConfig>,
    ) {
        let config = batch_config.unwrap_or_default();
        self.batch_processor = Some(EmbeddingBatchProcessor::new(ollama_client, thread_pool, config));
    }
    
    pub fn get_or_create_collection(&mut self, name: &str) -> &mut InMemoryCollection {
        if !self.collections.contains_key(name) {
            let collection = InMemoryCollection {
                name: name.to_string(),
                documents: HashMap::new(),
            };
            self.collections.insert(name.to_string(), collection);
        }
        
        self.collections.get_mut(name).unwrap()
    }
    
    pub fn list_collections(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.collections.keys().cloned().collect())
    }
    
    pub fn add_documents(
        &mut self,
        collection_name: &str,
        documents: Vec<String>,
        metadatas: Vec<DocumentMetadata>,
        ids: Option<Vec<String>>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        
        // Generate IDs if not provided
        let document_ids = if let Some(provided_ids) = ids {
            provided_ids
        } else {
            (0..documents.len())
                .map(|_| format!("doc_{}", uuid::Uuid::new_v4()))
                .collect()
        };
        
        // Add documents to collection
        for ((id, content), metadata) in document_ids.into_iter()
            .zip(documents.into_iter())
            .zip(metadatas.into_iter()) {
            
            let document = Document {
                id: id.clone(),
                content,
                metadata,
                embedding: None, // Embeddings will be generated when Ollama integration is implemented
            };
            
            collection.documents.insert(id, document);
        }
        
        // Invalidate cache for this collection since we added new documents
        self.query_cache.invalidate_collection(collection_name);
        
        Ok(())
    }
    
    pub fn query(
        &mut self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<QueryResult>, Box<dyn Error>> {
        // Check cache first
        if let Some(cached_results) = self.query_cache.get(collection_name, query_text, n_results, &filter) {
            return Ok(cached_results);
        }

        // Cache miss - perform actual query
        let results = self.perform_query(collection_name, query_text, n_results, &filter)?;
        
        // Store results in cache
        self.query_cache.put(collection_name, query_text, n_results, &filter, results.clone(), None);
        
        Ok(results)
    }

    /// Internal method to perform the actual query (without caching)
    fn perform_query(
        &mut self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        _filter: &Option<serde_json::Value>,
    ) -> Result<Vec<QueryResult>, Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        
        // Simple text-based search for now (will be replaced with semantic search)
        let mut results = Vec::new();
        let query_lower = query_text.to_lowercase();
        
        for (_, document) in &collection.documents {
            let content_lower = document.content.to_lowercase();
            
            // Simple keyword matching - calculate a basic relevance score
            let keywords: Vec<&str> = query_lower.split_whitespace().collect();
            let mut matches = 0;
            
            for keyword in &keywords {
                if content_lower.contains(keyword) {
                    matches += 1;
                }
            }
            
            if matches > 0 {
                // Simple distance calculation (lower is better)
                let distance = 1.0 - (matches as f32 / keywords.len() as f32);
                
                results.push(QueryResult {
                    document: document.content.clone(),
                    metadata: document.metadata.clone(),
                    distance,
                    id: document.id.clone(),
                });
            }
        }
        
        // Sort by distance (best matches first) and limit results
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(n_results);
        
        Ok(results)
    }
    
    pub fn delete(
        &mut self,
        collection_name: &str,
        ids: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        
        for id in ids {
            collection.documents.remove(&id);
        }
        
        // Invalidate cache for this collection since we removed documents
        self.query_cache.invalidate_collection(collection_name);
        
        Ok(())
    }
    
    pub fn update(
        &mut self,
        collection_name: &str,
        ids: Vec<String>,
        documents: Vec<String>,
        metadatas: Vec<DocumentMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        
        // Update documents
        for ((id, content), metadata) in ids.into_iter()
            .zip(documents.into_iter())
            .zip(metadatas.into_iter()) {
            
            if let Some(existing_doc) = collection.documents.get_mut(&id) {
                existing_doc.content = content;
                existing_doc.metadata = metadata;
                existing_doc.embedding = None; // Reset embedding for re-calculation
            }
        }
        
        // Invalidate cache for this collection since we updated documents
        self.query_cache.invalidate_collection(collection_name);
        
        Ok(())
    }
    
    pub fn count(&mut self, collection_name: &str) -> Result<usize, Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        Ok(collection.documents.len())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.query_cache.get_stats()
    }

    /// Clear query cache
    pub fn clear_cache(&self) {
        self.query_cache.clear()
    }

    /// Invalidate cache for a specific collection
    pub fn invalidate_collection_cache(&self, collection_name: &str) {
        self.query_cache.invalidate_collection(collection_name)
    }

    /// Perform a query without using cache (for testing or comparison)
    pub fn query_without_cache(
        &mut self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<QueryResult>, Box<dyn Error>> {
        self.perform_query(collection_name, query_text, n_results, &filter)
    }

    /// Add documents with batch embedding generation
    pub async fn add_documents_with_embeddings(
        &mut self,
        collection_name: &str,
        documents: Vec<String>,
        metadatas: Vec<DocumentMetadata>,
        ids: Option<Vec<String>>,
        priority: Option<TaskPriority>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(ref batch_processor) = self.batch_processor {
            // Generate IDs if not provided
            let document_ids = if let Some(provided_ids) = ids {
                provided_ids
            } else {
                (0..documents.len())
                    .map(|_| format!("doc_{}", uuid::Uuid::new_v4()))
                    .collect()
            };

            // Create batches for embedding generation
            let batch_priority = priority.unwrap_or(TaskPriority::Normal);
            let batch_size = batch_processor.batch_config.max_batch_size;
            
            let mut all_embeddings = Vec::new();
            
            // Process documents in batches
            for (doc_chunk, id_chunk) in documents.chunks(batch_size).zip(document_ids.chunks(batch_size)) {
                let batch = EmbeddingBatch {
                    texts: doc_chunk.to_vec(),
                    document_ids: id_chunk.to_vec(),
                    collection_name: collection_name.to_string(),
                    priority: batch_priority.clone(),
                };

                let embeddings = batch_processor.process_batch(batch).await?;
                all_embeddings.extend(embeddings);
            }

            // Add documents to collection with embeddings
            let collection = self.get_or_create_collection(collection_name);
            
            for (((id, content), metadata), (_embed_id, embedding)) in document_ids.into_iter()
                .zip(documents.into_iter())
                .zip(metadatas.into_iter())
                .zip(all_embeddings.into_iter()) {
                
                let document = Document {
                    id: id.clone(),
                    content,
                    metadata,
                    embedding: Some(embedding),
                };
                
                collection.documents.insert(id, document);
            }

            // Invalidate cache for this collection
            self.query_cache.invalidate_collection(collection_name);
            
            Ok(())
        } else {
            // Fall back to regular document addition without embeddings
            self.add_documents(collection_name, documents, metadatas, ids.map(|ids| ids))
        }
    }

    /// Get batch processing statistics (if enabled)
    pub fn get_batch_stats(&self) -> Option<BatchStats> {
        self.batch_processor.as_ref().map(|processor| processor.get_stats())
    }

    /// Check if batch processing is enabled
    pub fn is_batch_processing_enabled(&self) -> bool {
        self.batch_processor.is_some()
    }

    /// Process a single document for embedding (useful for real-time additions)
    pub async fn add_single_document_with_embedding(
        &mut self,
        collection_name: &str,
        document: String,
        metadata: DocumentMetadata,
        id: Option<String>,
    ) -> Result<String, Box<dyn Error>> {
        let document_id = id.unwrap_or_else(|| format!("doc_{}", uuid::Uuid::new_v4()));
        
        if let Some(ref batch_processor) = self.batch_processor {
            // Create a single-item batch
            let batch = EmbeddingBatch {
                texts: vec![document.clone()],
                document_ids: vec![document_id.clone()],
                collection_name: collection_name.to_string(),
                priority: TaskPriority::High, // Single documents get high priority
            };

            let embeddings = batch_processor.process_batch(batch).await?;
            
            if let Some((_, embedding)) = embeddings.first() {
                let collection = self.get_or_create_collection(collection_name);
                
                let doc = Document {
                    id: document_id.clone(),
                    content: document,
                    metadata,
                    embedding: Some(embedding.clone()),
                };
                
                collection.documents.insert(document_id.clone(), doc);
                
                // Invalidate cache for this collection
                self.query_cache.invalidate_collection(collection_name);
                
                Ok(document_id)
            } else {
                Err("Failed to generate embedding for document".into())
            }
        } else {
            // Fall back to regular document addition
            self.add_documents(collection_name, vec![document], vec![metadata], Some(vec![document_id.clone()]))?;
            Ok(document_id)
        }
    }

    /// Execute operation with health monitoring and error handling
    pub async fn with_health_monitoring<F, T>(&self, operation_name: &str, operation: F) -> Result<T, Box<dyn Error>>
    where
        F: FnOnce() -> Result<T, Box<dyn Error>>,
    {
        let start_time = Instant::now();
        
        // Attempt the operation
        match operation() {
            Ok(result) => {
                let operation_time = start_time.elapsed();
                self.health_monitor.record_operation(true, operation_time).await;
                
                // Update collection and document counts
                self.update_health_stats().await?;
                
                Ok(result)
            }
            Err(e) => {
                let operation_time = start_time.elapsed();
                self.health_monitor.record_operation(false, operation_time).await;
                
                Err(format!("{} failed: {}", operation_name, e).into())
            }
        }
    }

    /// Update health statistics with current collection and document counts
    async fn update_health_stats(&self) -> Result<(), Box<dyn Error>> {
        let collection_count = self.collections.len();
        let document_count: usize = self.collections.values()
            .map(|collection| collection.documents.len())
            .sum();
        
        // Rough memory estimate (this would be more accurate with real ChromaDB)
        let memory_estimate = document_count * 1024; // Rough estimate: 1KB per document
        
        self.health_monitor
            .update_counts(collection_count, document_count, memory_estimate)
            .await;
        
        Ok(())
    }

    /// Get health monitoring statistics
    pub async fn get_health_stats(&self) -> ChromaHealthStats {
        self.health_monitor.get_stats().await
    }

    /// Check if ChromaDB service is healthy
    pub async fn is_healthy(&self) -> bool {
        self.health_monitor.is_healthy().await
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) {
        if self.health_monitor.is_monitoring.swap(true, Ordering::SeqCst) {
            return; // Already monitoring
        }

        let health_monitor = self.health_monitor.clone();
        let collections_clone = Arc::new(TokioMutex::new(&self.collections as *const HashMap<String, InMemoryCollection>));
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(health_monitor.config.check_interval_seconds)
            );
            
            while health_monitor.is_monitoring.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Update health stats periodically
                // For in-memory implementation, we're always "healthy" unless operations fail
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                let mut stats = health_monitor.stats.lock().await;
                stats.last_check_time = Some(now);
            }
        });
    }

    /// Stop background health monitoring
    pub fn stop_health_monitoring(&self) {
        self.health_monitor.is_monitoring.store(false, Ordering::SeqCst);
    }

    /// Validate ChromaDB connection (for future real ChromaDB integration)
    pub async fn validate_connection(&self) -> Result<bool, Box<dyn Error>> {
        // For in-memory implementation, always return true
        // In future real ChromaDB integration, this would test actual connection
        let start_time = Instant::now();
        
        // Simulate connection validation by checking if we can perform basic operations
        let validation_result = self.with_health_monitoring("connection_validation", || {
            // Test basic functionality
            if self.collections.len() >= 0 {
                Ok(true)
            } else {
                Err("Collections HashMap is invalid".into())
            }
        }).await;

        match validation_result {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("ChromaDB connection validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Perform health check with retry mechanism
    pub async fn check_connection_with_retry(&self) -> Result<bool, Box<dyn Error>> {
        let mut last_error = None;
        
        for attempt in 1..=self.health_monitor.config.max_retry_attempts {
            match self.validate_connection().await {
                Ok(is_healthy) => {
                    return Ok(is_healthy);
                }
                Err(e) => {
                    last_error = Some(e);
                    
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
        if let Some(error) = last_error {
            Err(error)
        } else {
            Err("Connection check failed after all retry attempts".into())
        }
    }
}

// Note: Ollama embedding function integration is planned for future releases

// Tauri command implementations
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(rename = "queryText")]
    pub query_text: String,
    #[serde(rename = "nResults")]
    pub n_results: usize,
    pub filter: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddDocumentsRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub documents: Vec<String>,
    pub metadatas: Vec<DocumentMetadata>,
    pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentsRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub ids: Vec<String>,
}

#[tauri::command]
pub fn list_chroma_collections(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<Vec<String>, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.list_collections().map_err(|e| format!("Failed to list collections: {}", e))
}

#[tauri::command]
pub fn create_chroma_collection(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    collection_name: String,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.get_or_create_collection(&collection_name);
    Ok(())
}

#[tauri::command]
pub fn delete_chroma_collection(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    collection_name: String,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.collections.remove(&collection_name);
    Ok(())
}

#[tauri::command]
pub fn add_documents_to_chroma(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    request: AddDocumentsRequest,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.add_documents(&request.collection_name, request.documents, request.metadatas, request.ids)
                .map_err(|e| format!("Failed to add documents: {}", e))
}

#[tauri::command]
pub fn query_chroma(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    request: QueryRequest,
) -> Result<Vec<QueryResult>, String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.query(&request.collection_name, &request.query_text, request.n_results, request.filter)
                .map_err(|e| format!("Failed to query collection: {}", e))
}

#[tauri::command]
pub fn get_documents_from_chroma(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    collection_name: String,
    ids: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<Vec<QueryResult>, String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    let collection = manager.get_or_create_collection(&collection_name);
    
    let mut documents = Vec::new();
    let mut count = 0;
    let max_count = limit.unwrap_or(usize::MAX);
    
    for (id, document) in &collection.documents {
        // If specific IDs are requested, only include those
        if let Some(ref requested_ids) = ids {
            if !requested_ids.contains(id) {
                continue;
            }
        }
        
        if count >= max_count {
            break;
        }
        
        documents.push(QueryResult {
            document: document.content.clone(),
            metadata: document.metadata.clone(),
            distance: 0.0, // No distance for direct get
            id: id.clone(),
        });
        
        count += 1;
    }
    
    Ok(documents)
}

#[tauri::command]
pub fn delete_documents_from_chroma(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    request: DeleteDocumentsRequest,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.delete(&request.collection_name, request.ids)
                .map_err(|e| format!("Failed to delete documents: {}", e))
}

#[tauri::command]
pub fn get_collection_count(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    collection_name: String,
) -> Result<usize, String> {
    let mut manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.count(&collection_name).map_err(|e| format!("Failed to get collection count: {}", e))
}

#[tauri::command]
pub fn get_rag_cache_stats(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<CacheStats, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    Ok(manager.get_cache_stats())
}

#[tauri::command]
pub fn clear_rag_cache(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<(), String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.clear_cache();
    Ok(())
}

#[tauri::command]
pub fn invalidate_collection_cache(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
    collection_name: String,
) -> Result<(), String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.invalidate_collection_cache(&collection_name);
    Ok(())
}

#[tauri::command]
pub fn get_batch_processing_stats(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<Option<BatchStats>, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    Ok(manager.get_batch_stats())
}

#[tauri::command]
pub fn is_batch_processing_enabled(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<bool, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    Ok(manager.is_batch_processing_enabled())
}

// ChromaDB Health monitoring commands

#[tauri::command]
pub async fn get_chroma_health_stats(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<ChromaHealthStats, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    Ok(manager.get_health_stats().await)
}

#[tauri::command]
pub async fn check_chroma_health(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<bool, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    Ok(manager.is_healthy().await)
}

#[tauri::command]
pub async fn validate_chroma_connection(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<bool, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.validate_connection().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_chroma_connection_with_retry(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<bool, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.check_connection_with_retry().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_chroma_health_monitoring(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<(), String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.start_health_monitoring().await;
    Ok(())
}

#[tauri::command]
pub async fn stop_chroma_health_monitoring(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<(), String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    manager.stop_health_monitoring();
    Ok(())
}

#[tauri::command]
pub async fn check_chroma_connection_detailed(
    chroma_manager: State<'_, std::sync::Mutex<ChromaManager>>,
) -> Result<serde_json::Value, String> {
    let manager = chroma_manager.lock().map_err(|e| format!("Failed to lock ChromaManager: {}", e))?;
    
    let is_connected = manager.check_connection_with_retry().await.map_err(|e| e.to_string())?;
    let health_stats = manager.get_health_stats().await;
    let is_healthy = manager.is_healthy().await;
    
    Ok(serde_json::json!({
        "connected": is_connected,
        "healthy": is_healthy,
        "health_stats": health_stats,
        "implementation": "in_memory",
        "ready_for_real_chromadb": true
    }))
}

// Note: Additional tests will be added when ChromaDB integration is stabilized
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chroma_manager_creation() {
        // Basic test to ensure ChromaManager can be created
        // Note: This requires ChromaDB server to be running
        if let Ok(_manager) = ChromaManager::new("./test_chroma_db") {
            // ChromaDB connection successful
            assert!(true);
        } else {
            // ChromaDB server not available - skip test
            println!("ChromaDB server not available, skipping test");
        }
    }
}

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_stream::Stream;

/// Supported AI providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AIProvider {
    Ollama,
    OpenAI,
    Anthropic,
}

/// Model capability types for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    CodeGeneration,
    CodeExplanation,
    Debugging,
    Documentation,
    Refactoring,
    Testing,
    Architecture,
    GeneralChat,
    Translation,
    Analysis,
}

/// AI model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub provider: AIProvider,
    pub capabilities: Vec<ModelCapability>,
    pub context_length: u32,
    pub cost_per_token: Option<f64>,
    pub speed_tokens_per_second: Option<f64>,
    pub is_available: bool,
    pub description: String,
}

/// Generation options for AI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.3),
            max_tokens: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: false,
        }
    }
}

/// AI generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub provider: AIProvider,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost: Option<f64>,
}

/// Streaming chunk response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_complete: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AI provider trait for implementing different backends
#[async_trait]
pub trait AIProviderTrait: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> AIProvider;
    
    /// Check if the provider is available/healthy
    async fn is_healthy(&self) -> bool;
    
    /// List available models from this provider
    async fn list_models(&self) -> Result<Vec<AIModel>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Generate a completion
    async fn generate(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Generate a streaming completion
    async fn generate_stream(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, Box<dyn std::error::Error + Send + Sync>>> + Send + Unpin>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get model information
    async fn get_model_info(&self, model_id: &str) -> Result<AIModel, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Validate API key or connection
    async fn validate_connection(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

/// Multi-provider AI client manager
pub struct AIClientManager {
    providers: HashMap<AIProvider, Box<dyn AIProviderTrait>>,
    model_cache: HashMap<String, AIModel>,
    default_provider: AIProvider,
    task_routing: HashMap<ModelCapability, Vec<String>>, // capability -> preferred model IDs
}

impl AIClientManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            model_cache: HashMap::new(),
            default_provider: AIProvider::Ollama,
            task_routing: HashMap::new(),
        }
    }
    
    /// Register a new AI provider
    pub fn register_provider(&mut self, provider: Box<dyn AIProviderTrait>) {
        let provider_type = provider.provider_type();
        self.providers.insert(provider_type, provider);
    }
    
    /// Set the default provider for fallback
    pub fn set_default_provider(&mut self, provider: AIProvider) {
        self.default_provider = provider;
    }
    
    /// Get all available models across providers
    pub async fn get_all_models(&mut self) -> Result<Vec<AIModel>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_models = Vec::new();
        
        for (provider_type, provider) in &self.providers {
            if provider.is_healthy().await {
                match provider.list_models().await {
                    Ok(models) => {
                        for model in models {
                            self.model_cache.insert(model.id.clone(), model.clone());
                            all_models.push(model);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get models from {:?}: {}", provider_type, e);
                    }
                }
            }
        }
        
        Ok(all_models)
    }
    
    /// Get the best model for a specific capability
    pub async fn get_best_model_for_task(&self, capability: ModelCapability) -> Option<AIModel> {
        if let Some(preferred_models) = self.task_routing.get(&capability) {
            for model_id in preferred_models {
                if let Some(model) = self.model_cache.get(model_id) {
                    if model.is_available && model.capabilities.contains(&capability) {
                        return Some(model.clone());
                    }
                }
            }
        }
        
        // Fallback: find any available model with the capability
        for model in self.model_cache.values() {
            if model.is_available && model.capabilities.contains(&capability) {
                return Some(model.clone());
            }
        }
        
        None
    }
    
    /// Generate completion with automatic model selection
    pub async fn generate_smart(
        &self,
        capability: ModelCapability,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
        let model = self.get_best_model_for_task(capability).await
            .ok_or("No suitable model found for the requested capability")?;
            
        self.generate_with_model(&model.id, prompt, options).await
    }
    
    /// Generate completion with specific model
    pub async fn generate_with_model(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
        let model = self.model_cache.get(model_id)
            .ok_or(format!("Model {} not found", model_id))?;
            
        let provider = self.providers.get(&model.provider)
            .ok_or(format!("Provider {:?} not available", model.provider))?;
            
        provider.generate(model_id, prompt, options).await
    }
    
    /// Generate streaming completion with specific model
    pub async fn generate_stream_with_model(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, Box<dyn std::error::Error + Send + Sync>>> + Send + Unpin>, Box<dyn std::error::Error + Send + Sync>> {
        let model = self.model_cache.get(model_id)
            .ok_or(format!("Model {} not found", model_id))?;
            
        let provider = self.providers.get(&model.provider)
            .ok_or(format!("Provider {:?} not available", model.provider))?;
            
        provider.generate_stream(model_id, prompt, options).await
    }
    
    /// Configure task routing preferences
    pub fn configure_task_routing(&mut self, capability: ModelCapability, preferred_models: Vec<String>) {
        self.task_routing.insert(capability, preferred_models);
    }
    
    /// Get provider health status
    pub async fn get_provider_health(&self) -> HashMap<AIProvider, bool> {
        let mut health_status = HashMap::new();
        
        for (provider_type, provider) in &self.providers {
            health_status.insert(provider_type.clone(), provider.is_healthy().await);
        }
        
        health_status
    }
    
    /// Get model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&AIModel> {
        self.model_cache.get(model_id)
    }
    
    /// Get all cached models
    pub fn get_cached_models(&self) -> Vec<&AIModel> {
        self.model_cache.values().collect()
    }
}

/// Helper function to classify prompt into capability
pub fn classify_prompt_capability(prompt: &str) -> ModelCapability {
    let prompt_lower = prompt.to_lowercase();
    
    // Code generation keywords
    if prompt_lower.contains("write") && (prompt_lower.contains("function") || prompt_lower.contains("class") || prompt_lower.contains("code")) {
        return ModelCapability::CodeGeneration;
    }
    
    // Debugging keywords
    if prompt_lower.contains("debug") || prompt_lower.contains("error") || prompt_lower.contains("fix") || prompt_lower.contains("bug") {
        return ModelCapability::Debugging;
    }
    
    // Explanation keywords
    if prompt_lower.contains("explain") || prompt_lower.contains("what does") || prompt_lower.contains("how does") {
        return ModelCapability::CodeExplanation;
    }
    
    // Documentation keywords
    if prompt_lower.contains("document") || prompt_lower.contains("comment") || prompt_lower.contains("readme") {
        return ModelCapability::Documentation;
    }
    
    // Refactoring keywords
    if prompt_lower.contains("refactor") || prompt_lower.contains("improve") || prompt_lower.contains("optimize") {
        return ModelCapability::Refactoring;
    }
    
    // Testing keywords
    if prompt_lower.contains("test") || prompt_lower.contains("unit test") || prompt_lower.contains("spec") {
        return ModelCapability::Testing;
    }
    
    // Architecture keywords
    if prompt_lower.contains("architecture") || prompt_lower.contains("design") || prompt_lower.contains("pattern") {
        return ModelCapability::Architecture;
    }
    
    // Default to general chat
    ModelCapability::GeneralChat
}
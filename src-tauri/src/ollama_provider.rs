use crate::ai_providers::*;
use crate::ollama_client::{OllamaClient, GenerateOptions as OllamaOptions, ModelInfo, ModelResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};
use futures_util::stream;
use async_stream;

/// Ollama provider adapter
pub struct OllamaProvider {
    client: OllamaClient,
    model_cache: HashMap<String, AIModel>,
}

impl OllamaProvider {
    pub fn new(client: OllamaClient) -> Self {
        Self {
            client,
            model_cache: HashMap::new(),
        }
    }
    
    /// Convert Ollama model info to our AIModel format
    fn convert_model_info(&self, ollama_model: &ModelInfo) -> AIModel {
        let capabilities = Self::get_model_capabilities(&ollama_model.name);
        let context_length = Self::estimate_context_length(&ollama_model.name);
        
        AIModel {
            id: ollama_model.name.clone(),
            name: ollama_model.name.clone(),
            provider: AIProvider::Ollama,
            capabilities,
            context_length,
            cost_per_token: None, // Ollama models are free
            speed_tokens_per_second: Some(Self::estimate_speed(&ollama_model.name)),
            is_available: true,
            description: format!("Ollama {} model", ollama_model.name),
        }
    }
    
    /// Get model capabilities based on model name
    fn get_model_capabilities(model_name: &str) -> Vec<ModelCapability> {
        let name_lower = model_name.to_lowercase();
        
        // Code-specific models
        if name_lower.contains("code") || name_lower.contains("coder") {
            vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Refactoring,
                ModelCapability::Testing,
                ModelCapability::Documentation,
            ]
        }
        // Chat models like llama, mistral, etc.
        else if name_lower.contains("llama") || name_lower.contains("mistral") || 
                name_lower.contains("qwen") || name_lower.contains("deepseek") {
            vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::GeneralChat,
                ModelCapability::Analysis,
                ModelCapability::Documentation,
            ]
        }
        // Specialized models
        else if name_lower.contains("phi") {
            vec![
                ModelCapability::GeneralChat,
                ModelCapability::CodeExplanation,
                ModelCapability::Analysis,
            ]
        }
        else if name_lower.contains("wizard") {
            vec![
                ModelCapability::CodeGeneration,
                ModelCapability::Debugging,
                ModelCapability::Refactoring,
            ]
        }
        // Default capabilities
        else {
            vec![
                ModelCapability::GeneralChat,
                ModelCapability::CodeExplanation,
            ]
        }
    }
    
    /// Estimate context length based on model name
    fn estimate_context_length(model_name: &str) -> u32 {
        let name_lower = model_name.to_lowercase();
        
        // Check for explicit context length in name
        if name_lower.contains("32k") {
            32768
        } else if name_lower.contains("16k") {
            16384
        } else if name_lower.contains("8k") {
            8192
        } else if name_lower.contains("4k") {
            4096
        }
        // Model-specific defaults
        else if name_lower.contains("llama2") {
            4096
        } else if name_lower.contains("llama3") || name_lower.contains("llama-3") {
            8192
        } else if name_lower.contains("codellama") {
            16384
        } else if name_lower.contains("mistral") {
            8192
        } else if name_lower.contains("qwen") {
            32768
        } else if name_lower.contains("deepseek") {
            16384
        } else if name_lower.contains("phi") {
            2048
        }
        // Default fallback
        else {
            4096
        }
    }
    
    /// Estimate speed based on model size/type
    fn estimate_speed(model_name: &str) -> f64 {
        let name_lower = model_name.to_lowercase();
        
        // Extract parameter size if present
        if name_lower.contains("70b") || name_lower.contains("72b") {
            5.0  // Large models are slower
        } else if name_lower.contains("34b") || name_lower.contains("33b") {
            12.0 // Medium-large models
        } else if name_lower.contains("13b") || name_lower.contains("14b") {
            25.0 // Medium models
        } else if name_lower.contains("7b") || name_lower.contains("8b") {
            40.0 // Smaller models are faster
        } else if name_lower.contains("3b") || name_lower.contains("1b") {
            60.0 // Very small models
        }
        // Model family defaults
        else if name_lower.contains("phi") {
            50.0 // Phi models are generally fast
        } else if name_lower.contains("code") {
            20.0 // Code models tend to be larger
        }
        // Default speed
        else {
            30.0
        }
    }
    
    /// Convert our GenerationOptions to Ollama format
    fn convert_options(options: Option<GenerationOptions>) -> Option<OllamaOptions> {
        options.map(|opts| OllamaOptions {
            temperature: opts.temperature,
            top_p: opts.top_p,
            top_k: opts.top_k.map(|k| k as i32),
            max_tokens: opts.max_tokens.map(|t| t as i32),
        })
    }
}

#[async_trait]
impl AIProviderTrait for OllamaProvider {
    fn provider_type(&self) -> AIProvider {
        AIProvider::Ollama
    }
    
    async fn is_healthy(&self) -> bool {
        self.client.is_healthy().await
    }
    
    async fn list_models(&self) -> Result<Vec<AIModel>, Box<dyn std::error::Error + Send + Sync>> {
        match self.client.list_models().await {
            Ok(models) => {
                let ai_models: Vec<AIModel> = models
                    .iter()
                    .map(|model| self.convert_model_info(model))
                    .collect();
                Ok(ai_models)
            }
            Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        }
    }
    
    async fn generate(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
        let ollama_options = Self::convert_options(options);
        
        match self.client.generate_completion(model_id, prompt, ollama_options).await {
            Ok(response) => {
                // Parse the response to extract any usage information if available
                let mut metadata = HashMap::new();
                metadata.insert("provider".to_string(), serde_json::Value::String("ollama".to_string()));
                
                Ok(AIResponse {
                    content: response,
                    model: model_id.to_string(),
                    provider: AIProvider::Ollama,
                    usage: None, // Ollama doesn't provide token usage by default
                    finish_reason: Some("stop".to_string()),
                    metadata,
                })
            }
            Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        }
    }
    
    async fn generate_stream(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, Box<dyn std::error::Error + Send + Sync>>> + Send + Unpin>, Box<dyn std::error::Error + Send + Sync>> {
        let ollama_options = Self::convert_options(options);
        
        // Create a channel-based streaming adapter
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<StreamChunk, Box<dyn std::error::Error + Send + Sync>>>();
        
        // Clone the client and spawn the callback-based generation
        let client = self.client.clone();
        let model_id = model_id.to_string();
        let prompt = prompt.to_string();
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            let result = client.generate_stream(
                &model_id,
                &prompt,
                ollama_options,
                move |chunk: &str| {
                    let mut metadata = HashMap::new();
                    metadata.insert("provider".to_string(), serde_json::Value::String("ollama".to_string()));
                    
                    // Check if this is the final chunk  
                    let is_complete = chunk.trim().is_empty() || chunk.contains("\"done\":true");
                    
                    let stream_chunk = StreamChunk {
                        content: chunk.to_string(),
                        is_complete,
                        metadata,
                    };
                    
                    // Send the chunk through the channel
                    if tx_clone.send(Ok(stream_chunk)).is_err() {
                        // Receiver dropped, stop streaming
                        return;
                    }
                }
            ).await;
            
            // Handle any errors from the generation
            if let Err(e) = result {
                let _ = tx.send(Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))));
            }
        });
        
        // Create an async stream from the receiver
        let converted_stream = async_stream::stream! {
            while let Some(chunk_result) = rx.recv().await {
                yield chunk_result;
            }
        };
        
        Ok(Box::new(Box::pin(converted_stream)))
    }
    
    async fn get_model_info(&self, model_id: &str) -> Result<AIModel, Box<dyn std::error::Error + Send + Sync>> {
        // Try to get from cache first
        if let Some(model) = self.model_cache.get(model_id) {
            return Ok(model.clone());
        }
        
        // If not in cache, fetch all models and find the one we want
        let models = self.list_models().await?;
        for model in models {
            if model.id == model_id {
                return Ok(model);
            }
        }
        
        Err(format!("Model {} not found", model_id).into())
    }
    
    async fn validate_connection(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.client.is_healthy().await)
    }
}

// Add async_stream to dependencies for the stream! macro
// This should be added to Cargo.toml: async-stream = "0.3"
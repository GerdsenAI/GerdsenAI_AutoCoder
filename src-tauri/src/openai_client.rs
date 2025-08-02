use crate::ai_providers::*;
use async_trait::async_trait;
use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_stream::{Stream, StreamExt};
use futures_util::stream;

/// OpenAI API client
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    model_cache: HashMap<String, AIModel>,
    enabled: bool,
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: Option<OpenAIMessage>,
    delta: Option<OpenAIDelta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI Models API response
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    object: String,
    data: Vec<OpenAIModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", api_key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            api_key: api_key.clone(),
            base_url: "https://api.openai.com/v1".to_string(),
            model_cache: HashMap::new(),
            enabled: !api_key.is_empty(),
        }
    }
    
    pub fn new_with_base_url(api_key: String, base_url: String) -> Self {
        let mut client = Self::new(api_key);
        client.base_url = base_url;
        client
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Get model capabilities based on model name
    fn get_model_capabilities(model_name: &str) -> Vec<ModelCapability> {
        match model_name {
            name if name.starts_with("gpt-4") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Documentation,
                ModelCapability::Refactoring,
                ModelCapability::Testing,
                ModelCapability::Architecture,
                ModelCapability::GeneralChat,
                ModelCapability::Analysis,
            ],
            name if name.starts_with("gpt-3.5") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Documentation,
                ModelCapability::GeneralChat,
            ],
            name if name.contains("code") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Refactoring,
            ],
            _ => vec![ModelCapability::GeneralChat],
        }
    }
    
    /// Get context length for model
    fn get_context_length(model_name: &str) -> u32 {
        match model_name {
            "gpt-4-turbo-preview" | "gpt-4-1106-preview" => 128000,
            "gpt-4" | "gpt-4-0613" => 8192,
            "gpt-4-32k" | "gpt-4-32k-0613" => 32768,
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0125" => 16385,
            "gpt-3.5-turbo-1106" => 16385,
            "gpt-3.5-turbo-instruct" => 4096,
            _ => 4096, // Default fallback
        }
    }
    
    /// Get cost per token (input/output average)
    fn get_cost_per_token(model_name: &str) -> Option<f64> {
        match model_name {
            "gpt-4-turbo-preview" => Some(0.00002), // $0.01 input + $0.03 output / 1000 tokens
            "gpt-4" => Some(0.045), // $0.03 input + $0.06 output / 1000 tokens
            "gpt-4-32k" => Some(0.09), // $0.06 input + $0.12 output / 1000 tokens
            "gpt-3.5-turbo" => Some(0.00175), // $0.0005 input + $0.0015 output / 1000 tokens
            "gpt-3.5-turbo-instruct" => Some(0.00175),
            _ => None,
        }
    }
    
    /// Convert OpenAI model info to our AIModel format
    fn convert_model_info(&self, openai_model: &OpenAIModelInfo) -> AIModel {
        AIModel {
            id: openai_model.id.clone(),
            name: openai_model.id.clone(),
            provider: AIProvider::OpenAI,
            capabilities: Self::get_model_capabilities(&openai_model.id),
            context_length: Self::get_context_length(&openai_model.id),
            cost_per_token: Self::get_cost_per_token(&openai_model.id),
            speed_tokens_per_second: Some(50.0), // Approximate
            is_available: true,
            description: format!("OpenAI {} model", openai_model.id),
        }
    }
}

#[async_trait]
impl AIProviderTrait for OpenAIClient {
    fn provider_type(&self) -> AIProvider {
        AIProvider::OpenAI
    }
    
    async fn is_healthy(&self) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Test with a simple API call
        match self.validate_connection().await {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }
    
    async fn list_models(&self) -> Result<Vec<AIModel>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(vec![]);
        }
        
        let url = format!("{}/models", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("OpenAI API error: {}", response.status()).into());
        }
        
        let models_response: OpenAIModelsResponse = response.json().await?;
        
        let mut models = Vec::new();
        for model_info in models_response.data {
            // Filter to only include chat/completion models
            if model_info.id.starts_with("gpt-") && !model_info.id.contains("embedding") {
                models.push(self.convert_model_info(&model_info));
            }
        }
        
        Ok(models)
    }
    
    async fn generate(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Err("OpenAI provider is not enabled".into());
        }
        
        let opts = options.unwrap_or_default();
        
        let request = OpenAIRequest {
            model: model_id.to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
            top_p: opts.top_p,
            stop: opts.stop_sequences,
            stream: Some(false),
        };
        
        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }
        
        let openai_response: OpenAIResponse = response.json().await?;
        
        let content = openai_response.choices
            .first()
            .and_then(|choice| choice.message.as_ref())
            .map(|msg| msg.content.clone())
            .unwrap_or_default();
            
        let finish_reason = openai_response.choices
            .first()
            .and_then(|choice| choice.finish_reason.clone());
            
        let usage = openai_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            estimated_cost: Self::get_cost_per_token(model_id)
                .map(|cost| (u.total_tokens as f64) * cost / 1000.0),
        });
        
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::Value::String(openai_response.id));
        metadata.insert("created".to_string(), serde_json::Value::Number(openai_response.created.into()));
        
        Ok(AIResponse {
            content,
            model: model_id.to_string(),
            provider: AIProvider::OpenAI,
            usage,
            finish_reason,
            metadata,
        })
    }
    
    async fn generate_stream(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, Box<dyn std::error::Error + Send + Sync>>> + Send + Unpin>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Err("OpenAI provider is not enabled".into());
        }
        
        let opts = options.unwrap_or_default();
        
        let request = OpenAIRequest {
            model: model_id.to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
            top_p: opts.top_p,
            stop: opts.stop_sequences,
            stream: Some(true),
        };
        
        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }
        
        let bytes_stream = response.bytes_stream();
        let text_stream = bytes_stream.map(|chunk_result| {
            chunk_result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });
        
        let parsed_stream = text_stream.map(|chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    
                    // Parse SSE format: "data: {...}\n\n"
                    for line in text.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if json_str == "[DONE]" {
                                return Ok(StreamChunk {
                                    content: String::new(),
                                    is_complete: true,
                                    metadata: HashMap::new(),
                                });
                            }
                            
                            match serde_json::from_str::<OpenAIResponse>(json_str) {
                                Ok(response) => {
                                    let content = response.choices
                                        .first()
                                        .and_then(|choice| choice.delta.as_ref())
                                        .and_then(|delta| delta.content.as_ref())
                                        .cloned()
                                        .unwrap_or_default();
                                        
                                    let is_complete = response.choices
                                        .first()
                                        .and_then(|choice| choice.finish_reason.as_ref())
                                        .is_some();
                                        
                                    let mut metadata = HashMap::new();
                                    metadata.insert("id".to_string(), serde_json::Value::String(response.id));
                                    
                                    return Ok(StreamChunk {
                                        content,
                                        is_complete,
                                        metadata,
                                    });
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    
                    // If no valid data found, return empty chunk
                    Ok(StreamChunk {
                        content: String::new(),
                        is_complete: false,
                        metadata: HashMap::new(),
                    })
                }
                Err(e) => Err(e),
            }
        });
        
        Ok(Box::new(Box::pin(parsed_stream)))
    }
    
    async fn get_model_info(&self, model_id: &str) -> Result<AIModel, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Err("OpenAI provider is not enabled".into());
        }
        
        // Create model info from known data
        let model_info = OpenAIModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "openai".to_string(),
        };
        
        Ok(self.convert_model_info(&model_info))
    }
    
    async fn validate_connection(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(false);
        }
        
        let url = format!("{}/models", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        Ok(response.status().is_success())
    }
}
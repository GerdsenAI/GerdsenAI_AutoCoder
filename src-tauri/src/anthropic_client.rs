use crate::ai_providers::*;
use async_trait::async_trait;
use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_stream::{Stream, StreamExt};
use futures_util::stream;

/// Anthropic Claude API client
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
    model_cache: HashMap<String, AIModel>,
    enabled: bool,
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic streaming response
#[derive(Debug, Deserialize)]
struct AnthropicStreamResponse {
    #[serde(rename = "type")]
    event_type: String,
    message: Option<AnthropicResponse>,
    delta: Option<AnthropicDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
    stop_reason: Option<String>,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", api_key.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            api_key: api_key.clone(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model_cache: HashMap::new(),
            enabled: !api_key.is_empty(),
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Get model capabilities based on model name
    fn get_model_capabilities(model_name: &str) -> Vec<ModelCapability> {
        match model_name {
            name if name.starts_with("claude-3") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Documentation,
                ModelCapability::Refactoring,
                ModelCapability::Testing,
                ModelCapability::Architecture,
                ModelCapability::GeneralChat,
                ModelCapability::Analysis,
                ModelCapability::Translation,
            ],
            name if name.starts_with("claude-2") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::CodeExplanation,
                ModelCapability::Debugging,
                ModelCapability::Documentation,
                ModelCapability::GeneralChat,
                ModelCapability::Analysis,
            ],
            name if name.starts_with("claude-instant") => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::GeneralChat,
                ModelCapability::CodeExplanation,
            ],
            _ => vec![ModelCapability::GeneralChat],
        }
    }
    
    /// Get context length for model
    fn get_context_length(model_name: &str) -> u32 {
        match model_name {
            name if name.starts_with("claude-3") => 200000,
            name if name.starts_with("claude-2.1") => 200000,
            name if name.starts_with("claude-2") => 100000,
            name if name.starts_with("claude-instant") => 100000,
            _ => 100000, // Default fallback
        }
    }
    
    /// Get cost per token (input/output average)
    fn get_cost_per_token(model_name: &str) -> Option<f64> {
        match model_name {
            "claude-3-opus-20240229" => Some(0.0225), // $15 input + $75 output / 1M tokens
            "claude-3-sonnet-20240229" => Some(0.006), // $3 input + $15 output / 1M tokens
            "claude-3-haiku-20240307" => Some(0.0015), // $0.25 input + $1.25 output / 1M tokens
            "claude-2.1" => Some(0.024), // $8 input + $24 output / 1M tokens
            "claude-2.0" => Some(0.024),
            "claude-instant-1.2" => Some(0.004), // $0.8 input + $2.4 output / 1M tokens
            _ => None,
        }
    }
    
    /// Get available Claude models
    fn get_available_models() -> Vec<AIModel> {
        vec![
            AIModel {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                provider: AIProvider::Anthropic,
                capabilities: Self::get_model_capabilities("claude-3-opus"),
                context_length: 200000,
                cost_per_token: Self::get_cost_per_token("claude-3-opus-20240229"),
                speed_tokens_per_second: Some(25.0),
                is_available: true,
                description: "Most powerful Claude model for complex tasks".to_string(),
            },
            AIModel {
                id: "claude-3-sonnet-20240229".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                provider: AIProvider::Anthropic,
                capabilities: Self::get_model_capabilities("claude-3-sonnet"),
                context_length: 200000,
                cost_per_token: Self::get_cost_per_token("claude-3-sonnet-20240229"),
                speed_tokens_per_second: Some(40.0),
                is_available: true,
                description: "Balanced Claude model for most tasks".to_string(),
            },
            AIModel {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                provider: AIProvider::Anthropic,
                capabilities: Self::get_model_capabilities("claude-3-haiku"),
                context_length: 200000,
                cost_per_token: Self::get_cost_per_token("claude-3-haiku-20240307"),
                speed_tokens_per_second: Some(60.0),
                is_available: true,
                description: "Fastest Claude model for simple tasks".to_string(),
            },
            AIModel {
                id: "claude-2.1".to_string(),
                name: "Claude 2.1".to_string(),
                provider: AIProvider::Anthropic,
                capabilities: Self::get_model_capabilities("claude-2.1"),
                context_length: 200000,
                cost_per_token: Self::get_cost_per_token("claude-2.1"),
                speed_tokens_per_second: Some(30.0),
                is_available: true,
                description: "Previous generation Claude model".to_string(),
            },
            AIModel {
                id: "claude-instant-1.2".to_string(),
                name: "Claude Instant".to_string(),
                provider: AIProvider::Anthropic,
                capabilities: Self::get_model_capabilities("claude-instant"),
                context_length: 100000,
                cost_per_token: Self::get_cost_per_token("claude-instant-1.2"),
                speed_tokens_per_second: Some(80.0),
                is_available: true,
                description: "Fast and cost-effective Claude model".to_string(),
            },
        ]
    }
}

#[async_trait]
impl AIProviderTrait for AnthropicClient {
    fn provider_type(&self) -> AIProvider {
        AIProvider::Anthropic
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
        
        // Anthropic doesn't have a models endpoint, return predefined models
        Ok(Self::get_available_models())
    }
    
    async fn generate(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Err("Anthropic provider is not enabled".into());
        }
        
        let opts = options.unwrap_or_default();
        
        let request = AnthropicRequest {
            model: model_id.to_string(),
            max_tokens: opts.max_tokens.unwrap_or(4096),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: opts.temperature,
            top_p: opts.top_p,
            top_k: opts.top_k,
            stop_sequences: opts.stop_sequences,
            stream: Some(false),
        };
        
        let url = format!("{}/messages", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        
        let content = anthropic_response.content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();
            
        let usage = TokenUsage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            estimated_cost: Self::get_cost_per_token(model_id)
                .map(|cost| ((anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens) as f64) * cost / 1_000_000.0),
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::Value::String(anthropic_response.id));
        metadata.insert("type".to_string(), serde_json::Value::String(anthropic_response.response_type));
        if let Some(stop_seq) = anthropic_response.stop_sequence {
            metadata.insert("stop_sequence".to_string(), serde_json::Value::String(stop_seq));
        }
        
        Ok(AIResponse {
            content,
            model: model_id.to_string(),
            provider: AIProvider::Anthropic,
            usage: Some(usage),
            finish_reason: anthropic_response.stop_reason,
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
            return Err("Anthropic provider is not enabled".into());
        }
        
        let opts = options.unwrap_or_default();
        
        let request = AnthropicRequest {
            model: model_id.to_string(),
            max_tokens: opts.max_tokens.unwrap_or(4096),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: opts.temperature,
            top_p: opts.top_p,
            top_k: opts.top_k,
            stop_sequences: opts.stop_sequences,
            stream: Some(true),
        };
        
        let url = format!("{}/messages", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let bytes_stream = response.bytes_stream();
        let text_stream = bytes_stream.map(|chunk_result| {
            chunk_result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });
        
        let parsed_stream = text_stream.map(|chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    
                    // Parse SSE format: "event: ...\ndata: {...}\n\n"
                    let mut content = String::new();
                    let mut is_complete = false;
                    let mut metadata = HashMap::new();
                    
                    for line in text.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<AnthropicStreamResponse>(json_str) {
                                Ok(stream_response) => {
                                    match stream_response.event_type.as_str() {
                                        "content_block_delta" => {
                                            if let Some(delta) = stream_response.delta {
                                                if let Some(text) = delta.text {
                                                    content = text;
                                                }
                                                if delta.stop_reason.is_some() {
                                                    is_complete = true;
                                                }
                                            }
                                        }
                                        "message_stop" => {
                                            is_complete = true;
                                        }
                                        _ => {}
                                    }
                                    
                                    if let Some(message) = stream_response.message {
                                        metadata.insert("id".to_string(), serde_json::Value::String(message.id));
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    
                    Ok(StreamChunk {
                        content,
                        is_complete,
                        metadata,
                    })
                }
                Err(e) => Err(e),
            }
        });
        
        Ok(Box::new(Box::pin(parsed_stream)))
    }
    
    async fn get_model_info(&self, model_id: &str) -> Result<AIModel, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Err("Anthropic provider is not enabled".into());
        }
        
        let models = Self::get_available_models();
        models.into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model {} not found", model_id).into())
    }
    
    async fn validate_connection(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(false);
        }
        
        // Test with a minimal request
        let test_request = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 1,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
        };
        
        let url = format!("{}/messages", self.base_url);
        let response = self.client.post(&url).json(&test_request).send().await?;
        
        Ok(response.status().is_success())
    }
}
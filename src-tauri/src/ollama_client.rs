use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

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

#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    models_cache: Arc<Mutex<HashMap<String, ModelInfo>>>,
}

impl OllamaClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: Client::new(),
            models_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error>> {
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

    pub async fn get_model(&self, name: &str) -> Result<Option<ModelInfo>, Box<dyn Error>> {
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
    ) -> Result<String, Box<dyn Error>> {
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
    ) -> Result<(), Box<dyn Error>>
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
        let mut buffer = Vec::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            
            // Process complete JSON objects from the buffer
            let mut start = 0;
            for i in 0..buffer.len() {
                if buffer[i] == b'\n' {
                    if let Ok(line) = std::str::from_utf8(&buffer[start..i]) {
                        if !line.is_empty() {
                            if let Ok(response) = serde_json::from_str::<GenerateResponse>(line) {
                                callback(&response.response);
                                
                                if response.done.unwrap_or(false) {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    start = i + 1;
                }
            }
            
            // Keep any incomplete data in the buffer
            if start > 0 {
                buffer.drain(0..start);
            }
        }
        
        Ok(())
    }

    pub async fn chat<F>(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        options: Option<GenerateOptions>,
        mut callback: Option<F>,
    ) -> Result<ChatMessage, Box<dyn Error>>
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
        
        // Handle streaming response
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut full_response = String::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            
            // Process complete JSON objects from the buffer
            let mut start = 0;
            for i in 0..buffer.len() {
                if buffer[i] == b'\n' {
                    if let Ok(line) = std::str::from_utf8(&buffer[start..i]) {
                        if !line.is_empty() {
                            if let Ok(response) = serde_json::from_str::<ChatResponse>(line) {
                                if let Some(ref mut cb) = callback {
                                    cb(&response.message.content);
                                }
                                
                                full_response.push_str(&response.message.content);
                                
                                if response.done.unwrap_or(false) {
                                    return Ok(ChatMessage {
                                        role: "assistant".to_string(),
                                        content: full_response,
                                    });
                                }
                            }
                        }
                    }
                    start = i + 1;
                }
            }
            
            // Keep any incomplete data in the buffer
            if start > 0 {
                buffer.drain(0..start);
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
    ) -> Result<Vec<f32>, Box<dyn Error>> {
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

    pub async fn check_connection(&self) -> Result<bool, Box<dyn Error>> {
        let url = format!("{}/api/version", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

use futures_util::StreamExt;

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    
    #[tokio::test]
    async fn test_list_models() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"llama3.2:latest","modified_at":"2025-06-01T12:00:00Z","size":4200000000,"digest":"sha256:1234567890abcdef","details":{"parameter_size":"8B","quantization_level":"Q4_0","format":"gguf","family":"llama"}}]}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let models = client.list_models().await.unwrap();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "llama3.2:latest");
        assert_eq!(models[0].details.as_ref().unwrap().parameter_size.as_deref(), Some("8B"));
        
        mock.assert();
    }
    
    #[tokio::test]
    async fn test_generate_completion() {
        let mut server = Server::new();
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"model":"llama3.2:latest","response":"This is a test response."}"#)
            .create();
            
        let client = OllamaClient::new(Some(server.url()));
        let response = client.generate_completion("llama3.2:latest", "Test prompt", None).await.unwrap();
        
        assert_eq!(response, "This is a test response.");
        
        mock.assert();
    }
}

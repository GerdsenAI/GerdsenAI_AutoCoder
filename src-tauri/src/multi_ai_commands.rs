use crate::ai_providers::*;
use crate::ollama_provider::OllamaProvider;
use crate::openai_client::OpenAIClient;
use crate::anthropic_client::AnthropicClient;
use crate::ollama_client::OllamaClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Multi-AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAIConfig {
    pub default_provider: AIProvider,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub task_routing: HashMap<String, String>, // capability -> preferred model ID
    pub enabled_providers: Vec<AIProvider>,
}

impl Default for MultiAIConfig {
    fn default() -> Self {
        Self {
            default_provider: AIProvider::Ollama,
            openai_api_key: None,
            anthropic_api_key: None,
            task_routing: HashMap::new(),
            enabled_providers: vec![AIProvider::Ollama],
        }
    }
}

/// AI generation request
#[derive(Debug, Deserialize)]
pub struct AIGenerationRequest {
    pub prompt: String,
    pub model_id: Option<String>,
    pub capability: Option<String>, // For smart routing
    pub options: Option<GenerationOptions>,
    pub stream: Option<bool>,
}

/// AI generation response
#[derive(Debug, Serialize)]
pub struct AIGenerationResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model list response
#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub models: Vec<AIModel>,
    pub provider_health: HashMap<String, bool>,
}

/// Provider health status
#[derive(Debug, Serialize)]
pub struct ProviderHealthResponse {
    pub providers: HashMap<String, ProviderStatus>,
}

#[derive(Debug, Serialize)]
pub struct ProviderStatus {
    pub healthy: bool,
    pub model_count: usize,
    pub last_checked: String,
}

/// Multi-AI manager state
pub struct MultiAIManager {
    client_manager: Arc<Mutex<AIClientManager>>,
    config: Arc<Mutex<MultiAIConfig>>,
}

impl MultiAIManager {
    pub fn new() -> Self {
        Self {
            client_manager: Arc::new(Mutex::new(AIClientManager::new())),
            config: Arc::new(Mutex::new(MultiAIConfig::default())),
        }
    }
    
    /// Initialize providers based on configuration
    pub async fn initialize_providers(&self, config: MultiAIConfig) {
        let mut manager = self.client_manager.lock().await;
        let mut stored_config = self.config.lock().await;
        
        // Always register Ollama provider
        let ollama_client = OllamaClient::new(Some("http://localhost:11434".to_string()));
        let ollama_provider = OllamaProvider::new(ollama_client);
        manager.register_provider(Box::new(ollama_provider));
        
        // Register OpenAI if API key is provided
        if let Some(api_key) = &config.openai_api_key {
            if !api_key.is_empty() {
                let openai_client = OpenAIClient::new(api_key.clone());
                manager.register_provider(Box::new(openai_client));
            }
        }
        
        // Register Anthropic if API key is provided
        if let Some(api_key) = &config.anthropic_api_key {
            if !api_key.is_empty() {
                let anthropic_client = AnthropicClient::new(api_key.clone());
                manager.register_provider(Box::new(anthropic_client));
            }
        }
        
        // Set default provider
        manager.set_default_provider(config.default_provider.clone());
        
        // Configure task routing
        for (capability_str, model_id) in &config.task_routing {
            if let Ok(capability) = serde_json::from_str::<ModelCapability>(&format!("\"{}\"", capability_str)) {
                manager.configure_task_routing(capability, vec![model_id.clone()]);
            }
        }
        
        *stored_config = config;
    }
}

/// Initialize multi-AI system with configuration
#[tauri::command]
pub async fn initialize_multi_ai(
    config: MultiAIConfig,
    state: State<'_, MultiAIManager>,
) -> Result<(), String> {
    state.initialize_providers(config).await;
    Ok(())
}

/// Get all available models from all providers
#[tauri::command]
pub async fn get_all_ai_models(
    state: State<'_, MultiAIManager>,
) -> Result<ModelListResponse, String> {
    let mut manager = state.client_manager.lock().await;
    
    match manager.get_all_models().await {
        Ok(models) => {
            let health = manager.get_provider_health().await;
            let health_strings: HashMap<String, bool> = health
                .into_iter()
                .map(|(provider, status)| (format!("{:?}", provider), status))
                .collect();
                
            Ok(ModelListResponse {
                models,
                provider_health: health_strings,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Generate AI completion with smart model selection
#[tauri::command]
pub async fn generate_ai_smart(
    request: AIGenerationRequest,
    state: State<'_, MultiAIManager>,
) -> Result<AIGenerationResponse, String> {
    let manager = state.client_manager.lock().await;
    
    let result = if let Some(model_id) = request.model_id {
        // Use specific model
        manager.generate_with_model(&model_id, &request.prompt, request.options).await
    } else if let Some(capability_str) = request.capability {
        // Use smart routing based on capability
        if let Ok(capability) = serde_json::from_str::<ModelCapability>(&format!("\"{}\"", capability_str)) {
            manager.generate_smart(capability, &request.prompt, request.options).await
        } else {
            return Err("Invalid capability specified".to_string());
        }
    } else {
        // Use default model/capability
        let capability = classify_prompt_capability(&request.prompt);
        manager.generate_smart(capability, &request.prompt, request.options).await
    };
    
    match result {
        Ok(response) => Ok(AIGenerationResponse {
            content: response.content,
            model: response.model,
            provider: format!("{:?}", response.provider),
            usage: response.usage,
            finish_reason: response.finish_reason,
            metadata: response.metadata,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Generate streaming AI completion
#[tauri::command]
pub async fn generate_ai_stream(
    request: AIGenerationRequest,
    state: State<'_, MultiAIManager>,
) -> Result<String, String> {
    // For streaming, we'll return a session ID that the frontend can use
    // to subscribe to stream events. This is a simplified implementation.
    // In a full implementation, you'd use Tauri events or WebSockets.
    
    let _manager = state.client_manager.lock().await;
    let _model_id = request.model_id.unwrap_or_else(|| {
        // Get best model for the task
        let _capability = if let Some(cap_str) = request.capability {
            serde_json::from_str::<ModelCapability>(&format!("\"{}\"", cap_str))
                .unwrap_or_else(|_| classify_prompt_capability(&request.prompt))
        } else {
            classify_prompt_capability(&request.prompt)
        };
        
        // This is a simplified approach - in practice, you'd need async handling
        "default".to_string()
    });
    
    // Return a session ID for now - streaming would require WebSocket implementation
    Ok(format!("stream_session_{}", uuid::Uuid::new_v4()))
}

/// Get model information by ID
#[tauri::command]
pub async fn get_ai_model_info(
    model_id: String,
    state: State<'_, MultiAIManager>,
) -> Result<AIModel, String> {
    let manager = state.client_manager.lock().await;
    
    if let Some(model) = manager.get_model(&model_id) {
        Ok(model.clone())
    } else {
        Err(format!("Model {} not found", model_id))
    }
}

/// Get provider health status
#[tauri::command]
pub async fn get_provider_health(
    state: State<'_, MultiAIManager>,
) -> Result<ProviderHealthResponse, String> {
    let manager = state.client_manager.lock().await;
    let health = manager.get_provider_health().await;
    let models = manager.get_cached_models();
    
    let mut providers = HashMap::new();
    
    for (provider, healthy) in health {
        let model_count = models.iter()
            .filter(|m| m.provider == provider)
            .count();
            
        providers.insert(
            format!("{:?}", provider),
            ProviderStatus {
                healthy,
                model_count,
                last_checked: chrono::Utc::now().to_rfc3339(),
            },
        );
    }
    
    Ok(ProviderHealthResponse { providers })
}

/// Update multi-AI configuration
#[tauri::command]
pub async fn update_multi_ai_config(
    config: MultiAIConfig,
    state: State<'_, MultiAIManager>,
) -> Result<(), String> {
    state.initialize_providers(config).await;
    Ok(())
}

/// Get current multi-AI configuration
#[tauri::command]
pub async fn get_multi_ai_config(
    state: State<'_, MultiAIManager>,
) -> Result<MultiAIConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

/// Classify prompt to determine best capability
#[tauri::command]
pub fn classify_prompt(prompt: String) -> Result<String, String> {
    let capability = classify_prompt_capability(&prompt);
    Ok(format!("{:?}", capability))
}

/// Get available model capabilities
#[tauri::command]
pub fn get_model_capabilities() -> Result<Vec<String>, String> {
    let capabilities = vec![
        "CodeGeneration",
        "CodeExplanation", 
        "Debugging",
        "Documentation",
        "Refactoring",
        "Testing",
        "Architecture",
        "GeneralChat",
        "Translation",
        "Analysis",
    ];
    
    Ok(capabilities.into_iter().map(String::from).collect())
}

/// Get supported AI providers
#[tauri::command]
pub fn get_supported_providers() -> Result<Vec<String>, String> {
    let providers = vec!["Ollama", "OpenAI", "Anthropic"];
    Ok(providers.into_iter().map(String::from).collect())
}
use crate::ollama_client::{OllamaClient, ChatMessage, GenerateOptions, HealthStats};
use crate::chroma_manager::ChromaManager;
use crate::operation_manager::{Operation, OperationStatus};
use crate::analysis_engine::{AnalysisEngine, AnalysisMode, AnalysisConfig, should_suggest_deep_analysis};
use crate::user_errors::ToUserError;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Emitter};
use tokio::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub parameter_size: Option<String>,
    pub size_mb: u64,
    pub quantization: Option<String>,
}

#[tauri::command]
pub async fn check_ollama_connection(
    _base_url: Option<String>,
    ollama_client: State<'_, OllamaClient>,
) -> Result<bool, String> {
    let client = ollama_client.inner();
    
    match client.check_connection().await {
        Ok(result) => Ok(result),
        Err(e) => {
            // Convert to user-friendly error
            let user_error = e.to_user_error();
            Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message))
        }
    }
}

#[tauri::command]
pub async fn list_models(
    ollama_client: State<'_, OllamaClient>,
) -> Result<Vec<ModelInfo>, String> {
    let client = ollama_client.inner();
    
    let models = match client.list_models().await {
        Ok(models) => models,
        Err(e) => {
            let user_error = e.to_user_error();
            return Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message));
        }
    };
        
    let model_infos = models
        .into_iter()
        .map(|model| {
            let details = model.details.unwrap_or_default();
            ModelInfo {
                name: model.name,
                parameter_size: details.parameter_size,
                size_mb: model.size / 1_000_000, // Convert to MB
                quantization: details.quantization_level,
            }
        })
        .collect();
        
    Ok(model_infos)
}

#[tauri::command]
pub async fn generate_completion(
    model: String,
    prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    ollama_client: State<'_, OllamaClient>,
) -> Result<String, String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens,
        top_p: None,
        top_k: None,
    };
    
    match client.generate_completion(&model, &prompt, Some(options)).await {
        Ok(result) => Ok(result),
        Err(e) => {
            let user_error = e.to_user_error();
            Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message))
        }
    }
}

#[tauri::command]
pub async fn generate_stream(
    model: String,
    prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    app_handle: AppHandle,
    ollama_client: State<'_, OllamaClient>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens,
        top_p: None,
        top_k: None,
    };
    
    let app_handle_clone = app_handle.clone();
    
    client
        .generate_stream(
            &model,
            &prompt,
            Some(options),
            move |token: &str| {
                let _ = app_handle_clone.emit("ollama-stream", token);
            },
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat(
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    stream: bool,
    app_handle: AppHandle,
    ollama_client: State<'_, OllamaClient>,
) -> Result<ChatMessage, String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens,
        top_p: None,
        top_k: None,
    };
    
    if !stream {
        return client
            .chat(&model, messages, Some(options), None::<fn(&str)>)
            .await
            .map_err(|e| e.to_string());
    }
    
    let app_handle_clone = app_handle.clone();
    
    client
        .chat(
            &model,
            messages,
            Some(options),
            Some(move |token: &str| {
                let _ = app_handle_clone.emit("ollama-stream", token);
            }),
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_embedding(
    model: String,
    text: String,
    ollama_client: State<'_, OllamaClient>,
) -> Result<Vec<f32>, String> {
    let client = ollama_client.inner();
    
    client
        .create_embedding(&model, &text)
        .await
        .map_err(|e| e.to_string())
}

// Commands specifically expected by main.rs

#[tauri::command]
pub async fn chat_with_ollama(
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    ollama_client: State<'_, OllamaClient>,
) -> Result<ChatMessage, String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens: None,
        top_p: None,
        top_k: None,
    };
    
    client
        .chat(&model, messages, Some(options), None::<fn(&str)>)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_stream_with_ollama(
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    app_handle: AppHandle,
    ollama_client: State<'_, OllamaClient>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens: None,
        top_p: None,
        top_k: None,
    };
    
    let app_handle_clone = app_handle.clone();
    
    client
        .chat(
            &model,
            messages,
            Some(options),
            Some(move |token: &str| {
                let _ = app_handle_clone.emit("ollama-stream", token);
            }),
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_with_ollama(
    model: String,
    prompt: String,
    temperature: Option<f32>,
    ollama_client: State<'_, OllamaClient>,
) -> Result<String, String> {
    let client = ollama_client.inner();
    
    let options = GenerateOptions {
        temperature,
        max_tokens: None,
        top_p: None,
        top_k: None,
    };
    
    client
        .generate_completion(&model, &prompt, Some(options))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_stream_with_ollama(
    model: String,
    prompt: String,
    use_rag: Option<bool>,
    session_id: Option<String>,
    _context: Option<Vec<String>>,
    temperature: Option<f32>,
    collection: Option<String>,
    analysis_mode: Option<String>,
    max_rounds: Option<usize>,
    save_to_rag: Option<bool>,
    app_handle: AppHandle,
    ollama_client: State<'_, OllamaClient>,
    chroma_manager: State<'_, tokio::sync::Mutex<ChromaManager>>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    let use_rag = use_rag.unwrap_or(false);
    
    // Parse analysis mode
    let analysis_mode = match analysis_mode.as_deref() {
        Some("socratic") => AnalysisMode::Socratic,
        Some("systematic") => AnalysisMode::Systematic,
        _ => AnalysisMode::Standard,
    };
    
    // Check if deep analysis is suggested for this prompt
    let suggest_deep_analysis = should_suggest_deep_analysis(&prompt);
    if suggest_deep_analysis && matches!(analysis_mode, AnalysisMode::Standard) {
        // Emit suggestion to frontend
        let _ = app_handle.emit("deep-analysis-suggestion", serde_json::json!({
            "session_id": session_id.as_ref().unwrap_or(&String::new()),
            "suggested": true,
            "reason": "Complex problem detected - consider using Socratic or Systematic analysis mode"
        }));
    }
    
    // Build enhanced prompt with RAG context if enabled
    let enhanced_prompt = if use_rag {
        // Query ChromaDB for relevant documents
        let mut manager = chroma_manager.lock().await;
        
        // Use the collection from frontend or default
        let default_collection = "default".to_string();
        let collection_name = collection.as_ref().unwrap_or(&default_collection);
        let n_results = 3; // Get top 3 most relevant documents
        
        match manager.query(collection_name, &prompt, n_results, None) {
            Ok(results) => {
                if !results.is_empty() {
                    let mut context_text = String::from("Based on the following relevant information:\n\n");
                    
                    for (i, result) in results.iter().enumerate() {
                        context_text.push_str(&format!("[Document {}]\n", i + 1));
                        context_text.push_str(&result.document);
                        context_text.push_str("\n\n");
                    }
                    
                    context_text.push_str(&format!("User Question: {}\n\nAnswer:", prompt));
                    
                    // Emit RAG context info
                    let _ = app_handle.emit("rag-context", serde_json::json!({
                        "session_id": session_id.as_ref().unwrap_or(&String::new()),
                        "documents_used": results.len(),
                        "collection": collection_name
                    }));
                    
                    context_text
                } else {
                    // No relevant documents found, use original prompt
                    prompt.clone()
                }
            }
            Err(e) => {
                // Log error but continue with original prompt
                eprintln!("RAG query error: {}", e);
                prompt.clone()
            }
        }
    } else {
        prompt.clone()
    };
    
    // Use Deep Analysis if mode is not Standard
    if !matches!(analysis_mode, AnalysisMode::Standard) {
        // Create analysis engine
        let chroma_manager_clone = {
            let _manager = chroma_manager.lock().await;
            // We can't clone ChromaManager, so we'll pass None for now
            // In a real implementation, we'd need to handle this differently
            None
        };
        
        let mut analysis_engine = AnalysisEngine::new(client.clone(), chroma_manager_clone);
        
        let analysis_config = AnalysisConfig {
            mode: analysis_mode.clone(),
            max_rounds: max_rounds.unwrap_or(5),
            time_limit: Duration::from_secs(300),
            save_to_rag: save_to_rag.unwrap_or(true),
        };
        
        // Emit analysis start event
        let _ = app_handle.emit("deep-analysis-start", serde_json::json!({
            "session_id": session_id.as_ref().unwrap_or(&String::new()),
            "mode": format!("{:?}", analysis_mode),
            "max_rounds": analysis_config.max_rounds
        }));
        
        match analysis_engine.analyze(&enhanced_prompt, &model, analysis_config).await {
            Ok(result) => {
                // Emit reasoning chain for UI display
                let _ = app_handle.emit("deep-analysis-reasoning", serde_json::json!({
                    "session_id": session_id.as_ref().unwrap_or(&String::new()),
                    "reasoning": result.reasoning,
                    "confidence": result.confidence
                }));
                
                // Stream the final solution
                let app_handle_clone = app_handle.clone();
                let solution_chars: Vec<char> = result.solution.chars().collect();
                
                // Stream solution character by character for smooth UX
                for (i, char) in solution_chars.iter().enumerate() {
                    let _ = app_handle_clone.emit("ollama-stream", serde_json::json!({
                        "token": char.to_string(),
                        "done": false
                    }));
                    
                    // Small delay to simulate streaming
                    if i % 10 == 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                }
                
                // Emit completion event
                let _ = app_handle.emit("ollama-stream", serde_json::json!({
                    "token": "",
                    "done": true
                }));
                
                // Emit final analysis result
                let _ = app_handle.emit("deep-analysis-complete", serde_json::json!({
                    "session_id": session_id.as_ref().unwrap_or(&String::new()),
                    "result": result
                }));
                
                Ok(())
            }
            Err(e) => {
                // Emit error and fall back to standard generation
                let _ = app_handle.emit("deep-analysis-error", serde_json::json!({
                    "session_id": session_id.as_ref().unwrap_or(&String::new()),
                    "error": e.clone()
                }));
                
                // Fall back to standard streaming
                standard_streaming_generation(client, &model, &enhanced_prompt, temperature, app_handle).await
            }
        }
    } else {
        // Standard streaming generation
        standard_streaming_generation(client, &model, &enhanced_prompt, temperature, app_handle).await
    }
}

/// Helper function for standard streaming generation
async fn standard_streaming_generation(
    client: &OllamaClient,
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let options = GenerateOptions {
        temperature,
        max_tokens: None,
        top_p: None,
        top_k: None,
    };
    
    let app_handle_clone = app_handle.clone();
    
    client
        .generate_stream(
            model,
            prompt,
            Some(options),
            move |token: &str| {
                let _ = app_handle_clone.emit("ollama-stream", serde_json::json!({
                    "token": token,
                    "done": false
                }));
            },
        )
        .await
        .map(|_| {
            // Emit completion event
            let _ = app_handle.emit("ollama-stream", serde_json::json!({
                "token": "",
                "done": true
            }));
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enqueue_operation(
    operation: Operation,
    op_manager: State<'_, crate::operation_manager::OperationManager>,
) -> Result<String, String> {
    op_manager.enqueue_operation(operation).await
}

#[tauri::command]
pub fn get_operation_status(
    operation_id: String,
    op_manager: State<'_, crate::operation_manager::OperationManager>,
) -> Option<OperationStatus> {
    op_manager.get_operation_status(&operation_id)
}

#[tauri::command]
pub fn cancel_operation(
    operation_id: String,
    op_manager: State<'_, crate::operation_manager::OperationManager>,
) -> Result<(), String> {
    op_manager.cancel_operation(&operation_id)
}

// Health monitoring commands

#[tauri::command]
pub async fn get_ollama_health_stats(
    ollama_client: State<'_, OllamaClient>,
) -> Result<HealthStats, String> {
    let client = ollama_client.inner();
    Ok(client.get_health_stats().await)
}

#[tauri::command]
pub async fn check_ollama_health(
    ollama_client: State<'_, OllamaClient>,
) -> Result<bool, String> {
    let client = ollama_client.inner();
    client.is_healthy().await.then_some(true).ok_or_else(|| "Ollama service is unhealthy".to_string())
}

#[tauri::command]
pub async fn start_ollama_health_monitoring(
    ollama_client: State<'_, OllamaClient>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    client.start_health_monitoring().await;
    Ok(())
}

#[tauri::command]
pub async fn stop_ollama_health_monitoring(
    ollama_client: State<'_, OllamaClient>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    client.stop_health_monitoring();
    Ok(())
}

#[tauri::command]
pub async fn check_ollama_connection_detailed(
    ollama_client: State<'_, OllamaClient>,
) -> Result<serde_json::Value, String> {
    let client = ollama_client.inner();
    
    let is_connected = client.check_connection_with_retry().await.map_err(|e| e.to_string())?;
    let health_stats = client.get_health_stats().await;
    
    Ok(serde_json::json!({
        "connected": is_connected,
        "health_stats": health_stats,
        "service_url": client.get_base_url()
    }))
}

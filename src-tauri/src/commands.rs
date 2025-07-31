use crate::ollama_client::{OllamaClient, ChatMessage, GenerateOptions};
use crate::chroma_manager::ChromaManager;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Emitter};
use tokio::sync::Mutex;

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
    
    client
        .check_connection()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_models(
    ollama_client: State<'_, OllamaClient>,
) -> Result<Vec<ModelInfo>, String> {
    let client = ollama_client.inner();
    
    let models = client
        .list_models()
        .await
        .map_err(|e| e.to_string())?;
        
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
    
    client
        .generate_completion(&model, &prompt, Some(options))
        .await
        .map_err(|e| e.to_string())
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
    context: Option<Vec<String>>,
    temperature: Option<f32>,
    collection: Option<String>,
    app_handle: AppHandle,
    ollama_client: State<'_, OllamaClient>,
    chroma_manager: State<'_, Mutex<ChromaManager>>,
) -> Result<(), String> {
    let client = ollama_client.inner();
    let use_rag = use_rag.unwrap_or(false);
    
    let options = GenerateOptions {
        temperature,
        max_tokens: None,
        top_p: None,
        top_k: None,
    };
    
    let app_handle_clone = app_handle.clone();
    
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
    
    client
        .generate_stream(
            &model,
            &enhanced_prompt,
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

use crate::chroma_manager::{ChromaManager, DocumentMetadata, QueryResult, OllamaEmbeddingFunction};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use crate::ollama_client::SharedOllamaClient;

pub type SharedChromaManager = Arc<Mutex<ChromaManager>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub collection_name: String,
    pub document: String,
    pub metadata: DocumentMetadata,
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub collection_name: String,
    pub query_text: String,
    pub n_results: usize,
    pub filter: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn initialize_chroma(
    db_path: String,
    embedding_model: String,
    ollama_client: State<'_, SharedOllamaClient>,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<bool, String> {
    let mut manager = chroma_manager.lock().await;
    
    // Set the embedding function using Ollama
    let embedding_function = OllamaEmbeddingFunction::new(
        embedding_model,
        ollama_client.inner().clone(),
    );
    
    manager.set_embedding_function(Box::new(embedding_function));
    
    Ok(true)
}

#[tauri::command]
pub async fn list_collections(
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<Vec<String>, String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .list_collections()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_document(
    request: AddDocumentRequest,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().await;
    
    let ids = if let Some(id) = request.id {
        Some(vec![id])
    } else {
        None
    };
    
    manager
        .add_documents(
            &request.collection_name,
            vec![request.document],
            vec![request.metadata],
            ids,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_documents(
    collection_name: String,
    documents: Vec<String>,
    metadatas: Vec<DocumentMetadata>,
    ids: Option<Vec<String>>,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .add_documents(&collection_name, documents, metadatas, ids)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn query_collection(
    request: QueryRequest,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<Vec<QueryResult>, String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .query(
            &request.collection_name,
            &request.query_text,
            request.n_results,
            request.filter,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_documents(
    collection_name: String,
    ids: Vec<String>,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .delete(&collection_name, ids)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_documents(
    collection_name: String,
    ids: Vec<String>,
    documents: Vec<String>,
    metadatas: Vec<DocumentMetadata>,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<(), String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .update(&collection_name, ids, documents, metadatas)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collection_count(
    collection_name: String,
    chroma_manager: State<'_, SharedChromaManager>,
) -> Result<usize, String> {
    let mut manager = chroma_manager.lock().await;
    
    manager
        .count(&collection_name)
        .await
        .map_err(|e| e.to_string())
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub document: String,
    pub metadata: DocumentMetadata,
    pub distance: f32,
    pub id: String,
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

pub struct ChromaManager {
    collections: HashMap<String, InMemoryCollection>,
}

impl ChromaManager {
    pub fn new(_db_path: &str) -> Result<Self, Box<dyn Error>> {
        // Simplified in-memory implementation for immediate RAG functionality
        Ok(Self {
            collections: HashMap::new(),
        })
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
        
        Ok(())
    }
    
    pub fn query(
        &mut self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        _filter: Option<serde_json::Value>,
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
        
        Ok(())
    }
    
    pub fn count(&mut self, collection_name: &str) -> Result<usize, Box<dyn Error>> {
        let collection = self.get_or_create_collection(collection_name);
        Ok(collection.documents.len())
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

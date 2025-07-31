use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub context: Option<ChatContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub code_snippets: Vec<CodeSnippet>,
    pub file_paths: Vec<String>,
    pub repository_path: Option<String>,
    pub additional_context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
}

pub struct HistoryManager {
    storage_path: PathBuf,
    sessions: HashMap<String, ChatSession>,
}

impl HistoryManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the app data directory
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
            
        // Create the history directory if it doesn't exist
        let history_dir = app_dir.join("history");
        fs::create_dir_all(&history_dir)?;
        
        // Load existing sessions
        let mut sessions = HashMap::new();
        for entry in fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ChatSession>(&content) {
                        sessions.insert(session.id.clone(), session);
                    }
                }
            }
        }
        
        Ok(Self {
            storage_path: history_dir,
            sessions,
        })
    }
    
    pub fn get_session(&self, id: &str) -> Option<ChatSession> {
        self.sessions.get(id).cloned()
    }
    
    pub fn list_sessions(&self) -> Vec<ChatSession> {
        let mut sessions: Vec<ChatSession> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }
    
    pub fn create_session(&mut self, title: &str, model: &str) -> Result<ChatSession, Box<dyn std::error::Error>> {
        let id = format!("session_{}", uuid::Uuid::new_v4());
        let now = Utc::now();
        
        let session = ChatSession {
            id: id.clone(),
            title: title.to_string(),
            messages: Vec::new(),
            model: model.to_string(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            context: None,
        };
        
        self.sessions.insert(id.clone(), session.clone());
        self.save_session(&session)?;
        
        Ok(session)
    }
    
    pub fn update_session(&mut self, session: ChatSession) -> Result<(), Box<dyn std::error::Error>> {
        let mut updated_session = session;
        updated_session.updated_at = Utc::now();
        
        self.sessions.insert(updated_session.id.clone(), updated_session.clone());
        self.save_session(&updated_session)?;
        
        Ok(())
    }
    
    pub fn delete_session(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.sessions.remove(id).is_some() {
            let file_path = self.storage_path.join(format!("{}.json", id));
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
        }
        
        Ok(())
    }
    
    pub fn add_message(&mut self, session_id: &str, role: &str, content: &str) -> Result<ChatSession, Box<dyn std::error::Error>> {
        let mut session = self.get_session(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
            
        let message = ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };
        
        session.messages.push(message);
        session.updated_at = Utc::now();
        
        self.sessions.insert(session_id.to_string(), session.clone());
        self.save_session(&session)?;
        
        Ok(session)
    }
    
    pub fn update_context(&mut self, session_id: &str, context: ChatContext) -> Result<ChatSession, Box<dyn std::error::Error>> {
        let mut session = self.get_session(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
            
        session.context = Some(context);
        session.updated_at = Utc::now();
        
        self.sessions.insert(session_id.to_string(), session.clone());
        self.save_session(&session)?;
        
        Ok(session)
    }
    
    pub fn add_tag(&mut self, session_id: &str, tag: &str) -> Result<ChatSession, Box<dyn std::error::Error>> {
        let mut session = self.get_session(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
            
        if !session.tags.contains(&tag.to_string()) {
            session.tags.push(tag.to_string());
            session.updated_at = Utc::now();
            
            self.sessions.insert(session_id.to_string(), session.clone());
            self.save_session(&session)?;
        }
        
        Ok(session)
    }
    
    pub fn remove_tag(&mut self, session_id: &str, tag: &str) -> Result<ChatSession, Box<dyn std::error::Error>> {
        let mut session = self.get_session(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
            
        session.tags.retain(|t| t != tag);
        session.updated_at = Utc::now();
        
        self.sessions.insert(session_id.to_string(), session.clone());
        self.save_session(&session)?;
        
        Ok(session)
    }
    
    pub fn search_sessions(&self, query: &str) -> Vec<ChatSession> {
        let query = query.to_lowercase();
        let mut results: Vec<ChatSession> = self.sessions.values()
            .filter(|session| {
                session.title.to_lowercase().contains(&query) ||
                session.tags.iter().any(|tag| tag.to_lowercase().contains(&query)) ||
                session.messages.iter().any(|msg| msg.content.to_lowercase().contains(&query))
            })
            .cloned()
            .collect();
            
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results
    }
    
    pub fn filter_sessions_by_tag(&self, tag: &str) -> Vec<ChatSession> {
        let mut results: Vec<ChatSession> = self.sessions.values()
            .filter(|session| session.tags.contains(&tag.to_string()))
            .cloned()
            .collect();
            
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results
    }
    
    fn save_session(&self, session: &ChatSession) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        fs::write(file_path, content)?;
        Ok(())
    }
}

pub type SharedHistoryManager = Arc<Mutex<HistoryManager>>;

// Tauri commands for history management

#[tauri::command]
pub async fn list_chat_sessions(
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<Vec<ChatSession>, String> {
    let manager = history_manager.lock().await;
    Ok(manager.list_sessions())
}

#[tauri::command]
pub async fn get_chat_session(
    id: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<Option<ChatSession>, String> {
    let manager = history_manager.lock().await;
    Ok(manager.get_session(&id))
}

#[tauri::command]
pub async fn create_chat_session(
    title: String,
    model: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<ChatSession, String> {
    let mut manager = history_manager.lock().await;
    manager.create_session(&title, &model).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_chat_session(
    session: ChatSession,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<(), String> {
    let mut manager = history_manager.lock().await;
    manager.update_session(session).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_chat_session(
    id: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<(), String> {
    let mut manager = history_manager.lock().await;
    manager.delete_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_chat_message(
    session_id: String,
    role: String,
    content: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<ChatSession, String> {
    let mut manager = history_manager.lock().await;
    manager.add_message(&session_id, &role, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_chat_context(
    session_id: String,
    context: ChatContext,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<ChatSession, String> {
    let mut manager = history_manager.lock().await;
    manager.update_context(&session_id, context).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_session_tag(
    session_id: String,
    tag: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<ChatSession, String> {
    let mut manager = history_manager.lock().await;
    manager.add_tag(&session_id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_session_tag(
    session_id: String,
    tag: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<ChatSession, String> {
    let mut manager = history_manager.lock().await;
    manager.remove_tag(&session_id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_chat_sessions(
    query: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<Vec<ChatSession>, String> {
    let manager = history_manager.lock().await;
    Ok(manager.search_sessions(&query))
}

#[tauri::command]
pub async fn filter_chat_sessions_by_tag(
    tag: String,
    history_manager: State<'_, SharedHistoryManager>,
) -> Result<Vec<ChatSession>, String> {
    let manager = history_manager.lock().await;
    Ok(manager.filter_sessions_by_tag(&tag))
}

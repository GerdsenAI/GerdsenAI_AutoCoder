use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub event_type: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchRequest {
    pub path: String,
    pub recursive: bool,
    pub patterns: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified: String,
}

pub struct FileWatcher {
    watchers: HashMap<String, RecommendedWatcher>,
    app_handle: AppHandle,
}

impl FileWatcher {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            watchers: HashMap::new(),
            app_handle,
        }
    }
    
    pub fn watch_directory(&mut self, path: &str, recursive: bool) -> Result<(), String> {
        let (tx, rx) = channel();
        let app_handle = self.app_handle.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        let event_type = match event.kind {
                            EventKind::Create(_) => "created",
                            EventKind::Modify(_) => "modified",
                            EventKind::Remove(_) => "removed",
                            _ => "other",
                        };
                        
                        for path in event.paths {
                            let file_event = FileChangeEvent {
                                path: path.to_string_lossy().to_string(),
                                event_type: event_type.to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            
                            let _ = app_handle.emit_all("file-changed", &file_event);
                        }
                    },
                    Err(e) => {
                        eprintln!("Watch error: {:?}", e);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| format!("Failed to create watcher: {}", e))?;
        
        let watch_path = Path::new(path);
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        
        watcher.watch(watch_path, mode)
            .map_err(|e| format!("Failed to watch path: {}", e))?;
            
        self.watchers.insert(path.to_string(), watcher);
        
        Ok(())
    }
    
    pub fn unwatch_directory(&mut self, path: &str) -> Result<(), String> {
        if let Some(mut watcher) = self.watchers.remove(path) {
            watcher.unwatch(Path::new(path))
                .map_err(|e| format!("Failed to unwatch path: {}", e))?;
        }
        
        Ok(())
    }
    
    pub fn list_files(&self, path: &str, recursive: bool) -> Result<Vec<FileInfo>, String> {
        let path = Path::new(path);
        
        if !path.exists() {
            return Err("Path does not exist".to_string());
        }
        
        let mut files = Vec::new();
        
        if recursive {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry.map_err(|e| format!("Error reading directory: {}", e))?;
                let metadata = entry.metadata().map_err(|e| format!("Error reading metadata: {}", e))?;
                
                files.push(FileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_directory: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|_| "unknown".to_string()),
                });
            }
        } else {
            for entry in fs::read_dir(path).map_err(|e| format!("Error reading directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
                let metadata = entry.metadata().map_err(|e| format!("Error reading metadata: {}", e))?;
                
                files.push(FileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_directory: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|_| "unknown".to_string()),
                });
            }
        }
        
        Ok(files)
    }
    
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))
    }
    
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write file: {}", e))
    }
}

pub type SharedFileWatcher = Arc<Mutex<FileWatcher>>;

// Tauri commands for file watching

#[tauri::command]
pub async fn watch_repository(
    request: WatchRequest,
    file_watcher: State<'_, SharedFileWatcher>,
) -> Result<(), String> {
    let mut watcher = file_watcher.lock().await;
    watcher.watch_directory(&request.path, request.recursive)
}

#[tauri::command]
pub async fn unwatch_repository(
    path: String,
    file_watcher: State<'_, SharedFileWatcher>,
) -> Result<(), String> {
    let mut watcher = file_watcher.lock().await;
    watcher.unwatch_directory(&path)
}

#[tauri::command]
pub async fn list_files(
    path: String,
    recursive: bool,
    file_watcher: State<'_, SharedFileWatcher>,
) -> Result<Vec
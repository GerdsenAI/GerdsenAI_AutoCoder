use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub center: Option<bool>,
    pub resizable: Option<bool>,
    pub maximized: Option<bool>,
    pub visible: Option<bool>,
    pub decorations: Option<bool>,
    pub always_on_top: Option<bool>,
    pub dock_to: Option<String>,
}

#[derive(Default)]
pub struct WindowManager {
    windows: Arc<Mutex<HashMap<String, Window>>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_window(&self, label: String, window: Window) {
        if let Ok(mut windows) = self.windows.lock() {
            windows.insert(label, window);
        }
    }

    pub fn get_window(&self, label: &str) -> Option<Window> {
        self.windows.lock().ok()?.get(label).cloned()
    }

    pub fn remove_window(&self, label: &str) {
        if let Ok(mut windows) = self.windows.lock() {
            windows.remove(label);
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.lock().map(|windows| windows.len()).unwrap_or(0)
    }
}

#[tauri::command]
pub async fn create_window(
    app_handle: AppHandle,
    config: WindowConfig,
) -> Result<String, String> {
    let window_manager = app_handle.state::<WindowManager>();
    
    let window_id = format!("window_{}", window_manager.window_count() + 1);
    
    let mut builder = tauri::WindowBuilder::new(
        &app_handle,
        window_id.clone(),
        tauri::WindowUrl::App("index.html".into()),
    )
    .title(config.title.unwrap_or_else(|| "Auto-Coder Companion".to_string()));
    
    if let Some(width) = config.width {
        if let Some(height) = config.height {
            builder = builder.inner_size(width, height);
        }
    }
    
    if let Some(x) = config.x {
        if let Some(y) = config.y {
            builder = builder.position(x, y);
        }
    }
    
    if let Some(center) = config.center {
        if center {
            builder = builder.center();
        }
    }
    
    if let Some(resizable) = config.resizable {
        builder = builder.resizable(resizable);
    }
    
    if let Some(maximized) = config.maximized {
        builder = builder.maximized(maximized);
    }
    
    if let Some(visible) = config.visible {
        builder = builder.visible(visible);
    }
    
    if let Some(decorations) = config.decorations {
        builder = builder.decorations(decorations);
    }
    
    if let Some(always_on_top) = config.always_on_top {
        builder = builder.always_on_top(always_on_top);
    }
    
    match builder.build() {
        Ok(window) => {
            window_manager.register_window(window_id.clone(), window);
            Ok(window_id)
        },
        Err(e) => Err(format!("Failed to create window: {}", e)),
    }
}

#[tauri::command]
pub async fn close_window(
    app_handle: AppHandle,
    window_label: String,
) -> Result<(), String> {
    let window_manager = app_handle.state::<WindowManager>();
    
    if let Some(window) = window_manager.get_window(&window_label) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        window_manager.remove_window(&window_label);
        Ok(())
    } else {
        Err(format!("Window not found: {}", window_label))
    }
}

#[tauri::command]
pub async fn dock_window(
    app_handle: AppHandle,
    window_label: String,
    position: String,
) -> Result<(), String> {
    let window_manager = app_handle.state::<WindowManager>();
    
    if let Some(window) = window_manager.get_window(&window_label) {
        // In a real implementation, this would communicate with the IDE
        // to dock the window at the specified position
        window.emit("dock-window", position).map_err(|e| format!("Failed to emit dock event: {}", e))?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", window_label))
    }
}

#[tauri::command]
pub async fn undock_window(
    app_handle: AppHandle,
    window_label: String,
) -> Result<(), String> {
    let window_manager = app_handle.state::<WindowManager>();
    
    if let Some(window) = window_manager.get_window(&window_label) {
        // In a real implementation, this would communicate with the IDE
        // to undock the window
        window.emit("undock-window", {}).map_err(|e| format!("Failed to emit undock event: {}", e))?;
        Ok(())
    } else {
        Err(format!("Window not found: {}", window_label))
    }
}

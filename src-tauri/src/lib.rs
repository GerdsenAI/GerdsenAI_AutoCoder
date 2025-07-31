// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import modules for testing
pub mod searxng_client;
pub mod searxng_commands;

#[cfg(test)]
mod tests;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .plugin(tauri_plugin_opener::init()) // Commented out - using main.rs setup instead
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

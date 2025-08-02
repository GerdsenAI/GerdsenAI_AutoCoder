// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import modules for testing
pub mod searxng_client;
pub mod searxng_commands;
pub mod operation_manager;
pub mod commands;
pub mod user_errors;
pub mod ollama_client;
pub mod chroma_manager;
pub mod analysis_engine;
pub mod thread_pool_manager;
pub mod ai_providers;
pub mod ollama_provider;
pub mod openai_client;
pub mod anthropic_client;
pub mod multi_ai_commands;

#[cfg(test)]
mod tests;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use operation_manager::{OperationManager, ResourceLimits};
use multi_ai_commands::MultiAIManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let num_cpus = num_cpus::get();
    let resource_limits = ResourceLimits {
        max_concurrent_operations: num_cpus * 2,
        max_memory_usage: 1024, // 1GB
        max_cpu_usage: 0.8,     // 80%
        io_throttling: true,
    };
    let operation_manager = OperationManager::new(resource_limits);
    let multi_ai_manager = MultiAIManager::new();
    
    tauri::Builder::default()
        // .plugin(tauri_plugin_opener::init()) // Commented out - using main.rs setup instead
        .manage(operation_manager)
        .manage(multi_ai_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            crate::commands::enqueue_operation,
            crate::commands::get_operation_status,
            crate::commands::cancel_operation,
            crate::multi_ai_commands::initialize_multi_ai,
            crate::multi_ai_commands::get_all_ai_models,
            crate::multi_ai_commands::generate_ai_smart,
            crate::multi_ai_commands::generate_ai_stream,
            crate::multi_ai_commands::get_ai_model_info,
            crate::multi_ai_commands::get_provider_health,
            crate::multi_ai_commands::update_multi_ai_config,
            crate::multi_ai_commands::get_multi_ai_config,
            crate::multi_ai_commands::classify_prompt,
            crate::multi_ai_commands::get_model_capabilities,
            crate::multi_ai_commands::get_supported_providers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

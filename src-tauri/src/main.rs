mod commands;
mod ollama_client;
mod searxng_client;
mod searxng_commands;
mod chroma_manager;
mod lsp_server;
mod code_analysis;
mod context_manager;
// mod doc_scraper;
// mod window_manager;
mod history_manager;
// mod file_watcher;

use tauri::Manager;
// use window_manager::WindowManager;
use ollama_client::{OllamaClient, SharedOllamaClient};
use searxng_client::SearXNGClient;
use chroma_manager::ChromaManager;
use code_analysis::CodeAnalysisService;
use context_manager::ContextManager;
use history_manager::{HistoryManager, SharedHistoryManager};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(debug_assertions)]
const LOG_TARGETS: [&str; 9] = [
    "auto_coder_companion::commands",
    "auto_coder_companion::ollama_client",
    "auto_coder_companion::searxng_client",
    "auto_coder_companion::chroma_manager",
    "auto_coder_companion::lsp_server",
    "auto_coder_companion::code_analysis",
    "auto_coder_companion::doc_scraper",
    "auto_coder_companion::window_manager",
    "auto_coder_companion::history_manager",
];

fn main() {
    #[cfg(debug_assertions)]
    {
        use tracing_subscriber::{fmt, EnvFilter};
        
        let filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into());
        
        let subscriber = fmt::Subscriber::builder()
            .with_env_filter(filter)
            .finish();
        
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    }
    
    // Temporarily disable menu for focus on ChromaDB implementation
    // TODO: Reimplement menu using Tauri 2.0 API later
    // Initialize ChromaManager with proper error handling
    let chroma_manager = ChromaManager::new("./chroma_db")
        .map_err(|e| format!("Failed to initialize ChromaDB: {}", e))
        .expect("ChromaDB initialization failed");
    
    // Initialize OllamaClient for AI services
    let ollama_client = OllamaClient::new(None);
    let shared_ollama_client: SharedOllamaClient = Arc::new(Mutex::new(ollama_client.clone()));
    
    // Initialize CodeAnalysisService with shared Ollama client
    let code_analysis_service = Arc::new(CodeAnalysisService::new(shared_ollama_client.clone()));

    // Initialize ContextManager with default settings (128k tokens, 25k reserved)
    let context_manager = ContextManager::default();

    tauri::Builder::default()
        // .manage(WindowManager::new())
        .manage(ollama_client)
        .manage(shared_ollama_client)
        .manage(SearXNGClient::new(None))
        .manage(Mutex::new(chroma_manager))
        .manage(code_analysis_service)
        .manage(context_manager)
        .setup(|app| {
            // Initialize HistoryManager
            let history_manager = HistoryManager::new(&app.handle())
                .map_err(|e| format!("Failed to initialize HistoryManager: {}", e))?;
            let shared_history: SharedHistoryManager = Arc::new(Mutex::new(history_manager));
            app.manage(shared_history);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_ollama_connection,
            commands::list_models,
            commands::chat_with_ollama,
            commands::chat_stream_with_ollama,
            commands::generate_with_ollama,
            commands::generate_stream_with_ollama,
            searxng_commands::check_searxng_connection,
            searxng_commands::search_web,
            searxng_commands::get_available_engines,
            searxng_commands::set_default_engines,
            searxng_commands::get_available_categories,
            chroma_manager::list_chroma_collections,
            chroma_manager::create_chroma_collection,
            chroma_manager::delete_chroma_collection,
            chroma_manager::add_documents_to_chroma,
            chroma_manager::query_chroma,
            chroma_manager::get_documents_from_chroma,
            chroma_manager::delete_documents_from_chroma,
            chroma_manager::get_collection_count,
            lsp_server::initialize_lsp_server,
            lsp_server::shutdown_lsp_server,
            lsp_server::lsp_open_document,
            lsp_server::lsp_close_document,
            lsp_server::lsp_update_document,
            lsp_server::lsp_get_diagnostics,
            lsp_server::lsp_get_completions,
            lsp_server::lsp_get_hover,
            lsp_server::lsp_get_code_actions,
            lsp_server::lsp_execute_command,
            code_analysis::analyze_code,
            code_analysis::analyze_repository,
            code_analysis::fix_code,
            code_analysis::generate_code,
            context_manager::get_context_budget,
            context_manager::pin_file,
            context_manager::unpin_file,
            context_manager::calculate_file_relevance,
            context_manager::build_context,
            context_manager::get_pinned_files,
            context_manager::count_file_tokens,
            // doc_scraper::scrape_documentation,
            // doc_scraper::batch_scrape_documentation,
            // doc_scraper::search_documentation,
            // doc_scraper::scrape_from_search,
            // window_manager::create_window,
            // window_manager::close_window,
            // window_manager::dock_window,
            // window_manager::undock_window,
            history_manager::list_chat_sessions,
            history_manager::get_chat_session,
            history_manager::create_chat_session,
            history_manager::update_chat_session,
            history_manager::delete_chat_session,
            history_manager::add_chat_message,
            // file_watcher::watch_repository,
            // file_watcher::unwatch_repository,
            // file_watcher::list_files,
            // file_watcher::read_file,
            // file_watcher::write_file,
        ])
        // .menu(menu) // Disabled for now
        .run(tauri::generate_context!())
        .expect("Error while running Auto-Coder Companion");
}

// fn create_new_window(app_handle: AppHandle) {
//     let window_manager = app_handle.state::<WindowManager>();
//     
//     let window_id = format!("window_{}", window_manager.window_count() + 1);
//     
//     let window = WindowBuilder::new(
//         &app_handle,
//         window_id.clone(),
//         WindowUrl::App("index.html".into())
//     )
//     .title("Auto-Coder Companion")
//     .inner_size(800.0, 600.0)
//     .build()
//     .unwrap();
//     
//     window_manager.register_window(window_id, window);
// }

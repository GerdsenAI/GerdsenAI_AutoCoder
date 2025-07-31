use crate::searxng_client::{SearXNGClient, SearchResult};
use tauri::State;

#[tauri::command]
pub async fn check_searxng_connection(
    base_url: Option<String>,
    searxng_client: State<'_, SearXNGClient>,
) -> Result<bool, String> {
    let client = searxng_client.inner().clone();
    
    if let Some(url) = base_url {
        client.set_base_url(url).await;
    }
    
    client
        .check_connection()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_engines(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<Vec<String>, String> {
    let client = searxng_client.inner();
    Ok(client.get_default_engines().await)
}

#[tauri::command]
pub async fn set_default_engines(
    engines: Vec<String>,
    _searxng_client: State<'_, SearXNGClient>,
) -> Result<(), String> {
    let client = _searxng_client.inner().clone();
    client.set_default_engines(engines).await;
    Ok(())
}

#[tauri::command]
pub async fn search_web(
    query: String,
    engines: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    limit: Option<usize>,
    searxng_client: State<'_, SearXNGClient>,
) -> Result<Vec<SearchResult>, String> {
    let client = searxng_client.inner();
    
    client
        .search(&query, engines, categories, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_categories(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<Vec<String>, String> {
    // Return commonly used SearXNG categories
    Ok(vec![
        "general".to_string(),
        "it".to_string(),
        "science".to_string(),
        "news".to_string(),
        "images".to_string(),
        "videos".to_string(),
        "music".to_string(),
        "files".to_string(),
        "social media".to_string(),
    ])
}

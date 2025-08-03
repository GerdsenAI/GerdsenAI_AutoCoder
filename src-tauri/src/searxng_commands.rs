use crate::searxng_client::{SearXNGClient, SearchResult, SearXNGHealthStats};
use crate::user_errors::{ToUserError, common};
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
    
    match client.check_connection().await {
        Ok(result) => Ok(result),
        Err(_e) => {
            let user_error = common::service_temporarily_unavailable("SearXNG");
            Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message))
        }
    }
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
    
    match client.search(&query, engines, categories, limit).await {
        Ok(results) => Ok(results),
        Err(e) => {
            let user_error = e.to_user_error();
            Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message))
        }
    }
}

#[tauri::command]
pub async fn get_available_categories(
    _searxng_client: State<'_, SearXNGClient>,
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

// SearXNG Health monitoring commands

#[tauri::command]
pub async fn get_searxng_health_stats(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<SearXNGHealthStats, String> {
    let client = searxng_client.inner();
    Ok(client.get_health_stats().await)
}

#[tauri::command]
pub async fn check_searxng_health(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<bool, String> {
    let client = searxng_client.inner();
    Ok(client.is_available().await)
}

#[tauri::command]
pub async fn check_searxng_degraded(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<bool, String> {
    let client = searxng_client.inner();
    Ok(client.is_degraded().await)
}

#[tauri::command]
pub async fn start_searxng_health_monitoring(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<(), String> {
    let client = searxng_client.inner();
    client.start_health_monitoring().await;
    Ok(())
}

#[tauri::command]
pub async fn stop_searxng_health_monitoring(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<(), String> {
    let client = searxng_client.inner();
    client.stop_health_monitoring();
    Ok(())
}

#[tauri::command]
pub async fn check_searxng_connection_detailed(
    searxng_client: State<'_, SearXNGClient>,
) -> Result<serde_json::Value, String> {
    let client = searxng_client.inner();
    
    let is_connected = client.check_connection_with_retry().await.map_err(|e| e.to_string())?;
    let health_stats = client.get_health_stats().await;
    let is_available = client.is_available().await;
    let is_degraded = client.is_degraded().await;
    
    Ok(serde_json::json!({
        "connected": is_connected,
        "available": is_available,
        "degraded": is_degraded,
        "health_stats": health_stats,
        "graceful_degradation": true
    }))
}

#[tauri::command]
pub async fn search_web_with_fallback(
    query: String,
    engines: Option<Vec<String>>,
    searxng_client: State<'_, SearXNGClient>,
) -> Result<SearchResult, String> {
    let client = searxng_client.inner();
    
    match client.search_with_fallback(&query, engines).await {
        Ok(result) => Ok(result),
        Err(e) => {
            let user_error = e.to_user_error();
            Err(serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message))
        }
    }
}

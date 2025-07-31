use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub engine: String,
    pub score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[derive(Clone)]
pub struct SearXNGClient {
    base_url: Arc<Mutex<String>>,
    client: Client,
    default_engines: Arc<Mutex<Vec<String>>>,
}

impl SearXNGClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: Arc::new(Mutex::new(
                base_url.unwrap_or_else(|| "http://localhost:8080".to_string())
            )),
            client: Client::new(),
            default_engines: Arc::new(Mutex::new(vec![
                "github".to_string(),
                "stackoverflow".to_string(),
                "google".to_string(),
                "duckduckgo".to_string(),
            ])),
        }
    }

    pub async fn set_base_url(&self, base_url: String) {
        let mut url = self.base_url.lock().await;
        *url = base_url;
    }

    pub async fn get_base_url(&self) -> String {
        self.base_url.lock().await.clone()
    }

    pub async fn set_default_engines(&self, engines: Vec<String>) {
        let mut default_engines = self.default_engines.lock().await;
        *default_engines = engines;
    }

    pub async fn get_default_engines(&self) -> Vec<String> {
        self.default_engines.lock().await.clone()
    }

    pub async fn search(
        &self,
        query: &str,
        engines: Option<Vec<String>>,
        categories: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send>> {
        let base_url = self.base_url.lock().await.clone();
        let url = format!("{}/search", base_url);
        
        let default_engines = self.default_engines.lock().await.clone();
        let engines_str = engines
            .unwrap_or(default_engines)
            .join(",");
            
        let categories_str = categories
            .unwrap_or_else(|| vec!["general".to_string(), "it".to_string()])
            .join(",");
            
        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("format", "json");
        params.insert("engines", &engines_str);
        params.insert("categories", &categories_str);
        
        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.insert("limit", &limit_str);
        }
        
        let response = self.client.get(&url)
            .query(&params)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
            
        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to search: {}", response.status()))) as Box<dyn Error + Send>);
        }
        
        let search_response: serde_json::Value = response.json().await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
        
        // Parse the SearXNG response format
        let results = if let Some(results) = search_response.get("results").and_then(|r| r.as_array()) {
            results
                .iter()
                .map(|result| {
                    let title = result.get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("Untitled")
                        .to_string();
                        
                    let url = result.get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                        
                    let content = result.get("content")
                        .and_then(|c| c.as_str())
                        .or_else(|| result.get("snippet").and_then(|s| s.as_str()))
                        .unwrap_or("")
                        .to_string();
                        
                    let engine = result.get("engine")
                        .and_then(|e| e.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                        
                    let score = result.get("score")
                        .and_then(|s| s.as_f64())
                        .map(|s| s as f32);
                        
                    SearchResult {
                        title,
                        url,
                        content,
                        engine,
                        score,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        
        Ok(results)
    }

    pub async fn check_connection(&self) -> Result<bool, Box<dyn Error + Send>> {
        let base_url = self.base_url.lock().await.clone();
        let url = format!("{}/healthz", base_url);
        
        match self.client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await 
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    
    #[tokio::test]
    async fn test_search() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"query":"rust","results":[{"title":"Rust Programming Language","url":"https://www.rust-lang.org/","content":"A language empowering everyone to build reliable and efficient software.","engine":"google","score":0.95}]}"#)
            .create();
            
        let client = SearXNGClient::new(Some(server.url()));
        let results = client.search("rust", None, None, None).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].engine, "google");
        
        mock.assert();
    }
    
    #[tokio::test]
    async fn test_check_connection() {
        let mut server = Server::new();
        
        let mock = server
            .mock("GET", "/healthz")
            .with_status(200)
            .create();
            
        let client = SearXNGClient::new(Some(server.url()));
        let is_connected = client.check_connection().await.unwrap();
        
        assert!(is_connected);
        
        mock.assert();
    }
}

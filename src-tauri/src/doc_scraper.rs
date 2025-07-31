use crate::chroma_manager::{ChromaManager, DocumentMetadata, QueryResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use crate::searxng_client::{SearXNGClient, SearchResult};

pub type SharedDocScraper = Arc<Mutex<DocumentationScraper>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapingRequest {
    pub url: String;
    pub collection_name: String;
    pub document_type: String;
    pub max_depth: Option<usize>,
    pub max_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapingResult {
    pub url: String;
    pub title: String;
    pub content: String;
    pub document_id: String;
    pub success: bool;
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchScrapingRequest {
    pub urls: Vec<String>;
    pub collection_name: String;
    pub document_type: String;
    pub max_depth: Option<usize>;
    pub max_pages: Option<usize>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchScrapingResult {
    pub results: Vec<ScrapingResult>;
    pub success_count: usize;
    pub failure_count: usize;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationSearchRequest {
    pub query: String;
    pub collection_name: String;
    pub n_results: usize;
    pub filter: Option<HashMap<String, String>>,
}

pub struct DocumentationScraper {
    chroma_manager: Arc<Mutex<ChromaManager>>,
    http_client: reqwest::Client,
}

impl DocumentationScraper {
    pub fn new(chroma_manager: Arc<Mutex<ChromaManager>>) -> Self {
        Self {
            chroma_manager,
            http_client: reqwest::Client::builder()
                .user_agent("Auto-Coder-Companion/1.0")
                .build()
                .unwrap_or_default(),
        }
    }
    
    pub async fn scrape_documentation(&self, request: &ScrapingRequest) -> Result<ScrapingResult, Box<dyn Error>> {
        // Fetch the content from the URL
        let response = self.http_client.get(&request.url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch URL: {}", response.status()).into());
        }
        
        let content = response.text().await?;
        
        // Extract title and clean content
        let (title, cleaned_content) = self.extract_content(&content);
        
        // Generate a unique document ID
        let document_id = format!("doc_{}", uuid::Uuid::new_v4());
        
        // Create metadata
        let metadata = DocumentMetadata {
            source: "web".to_string(),
            document_type: request.document_type.clone(),
            language: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_path: None,
            url: Some(request.url.clone()),
            title: Some(title.clone()),
            additional: HashMap::new(),
        };
        
        // Store in ChromaDB
        let mut chroma = self.chroma_manager.lock().await;
        chroma.add_documents(
            &request.collection_name,
            vec![cleaned_content.clone()],
            vec![metadata],
            Some(vec![document_id.clone()]),
        ).await?;
        
        Ok(ScrapingResult {
            url: request.url.clone(),
            title,
            content: cleaned_content,
            document_id,
            success: true,
            error: None,
        })
    }
    
    pub async fn batch_scrape_documentation(&self, request: &BatchScrapingRequest) -> Result<BatchScrapingResult, Box<dyn Error>> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for url in &request.urls {
            let scraping_request = ScrapingRequest {
                url: url.clone(),
                collection_name: request.collection_name.clone(),
                document_type: request.document_type.clone(),
                max_depth: request.max_depth,
                max_pages: request.max_pages,
            };
            
            match self.scrape_documentation(&scraping_request).await {
                Ok(result) => {
                    results.push(result);
                    success_count += 1;
                }
                Err(e) => {
                    results.push(ScrapingResult {
                        url: url.clone(),
                        title: String::new(),
                        content: String::new(),
                        document_id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    failure_count += 1;
                }
            }
        }
        
        Ok(BatchScrapingResult {
            results,
            success_count,
            failure_count,
        })
    }
    
    pub async fn search_documentation(&self, request: &DocumentationSearchRequest) -> Result<Vec<QueryResult>, Box<dyn Error>> {
        let mut chroma = self.chroma_manager.lock().await;
        
        // Convert filter to serde_json::Value if provided
        let filter = if let Some(filter_map) = &request.filter {
            let mut filter_obj = serde_json::Map::new();
            for (key, value) in filter_map {
                filter_obj.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
            Some(serde_json::Value::Object(filter_obj))
        } else {
            None
        };
        
        // Query ChromaDB
        let results = chroma.query(
            &request.collection_name,
            &request.query,
            request.n_results,
            filter,
        ).await?;
        
        Ok(results)
    }
    
    pub async fn scrape_from_search(&self, search_client: &SearXNGClient, query: &str, collection_name: &str, document_type: &str, max_results: usize) -> Result<BatchScrapingResult, Box<dyn Error>> {
        // Search for documentation
        let search_results = search_client.search(
            query,
            Some(vec!["documentation".to_string(), "github".to_string(), "stackoverflow".to_string()]),
            Some(vec!["it".to_string(), "programming".to_string()]),
            Some(max_results),
        ).await?;
        
        // Extract URLs
        let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
        
        // Batch scrape the URLs
        let request = BatchScrapingRequest {
            urls,
            collection_name: collection_name.to_string(),
            document_type: document_type.to_string(),
            max_depth: Some(1),
            max_pages: Some(max_results),
        };
        
        self.batch_scrape_documentation(&request).await
    }
    
    // Helper methods
    
    fn extract_content(&self, html: &str) -> (String, String) {
        // Use html5ever or similar to parse HTML and extract content
        // For simplicity, we'll use a basic approach here
        
        // Extract title
        let title = if let Some(title_start) = html.find("<title>") {
            if let Some(title_end) = html[title_start + 7..].find("</title>") {
                html[title_start + 7..title_start + 7 + title_end].trim().to_string()
            } else {
                "Untitled Document".to_string()
            }
        } else {
            "Untitled Document".to_string()
        };
        
        // Basic HTML to text conversion
        // In a real implementation, use a proper HTML parser
        let mut content = html.to_string();
        
        // Remove script tags
        while let Some(script_start) = content.find("<script") {
            if let Some(script_end) = content[script_start..].find("</script>") {
                content.replace_range(script_start..script_start + script_end + 9, "");
            } else {
                break;
            }
        }
        
        // Remove style tags
        while let Some(style_start) = content.find("<style") {
            if let Some(style_end) = content[style_start..].find("</style>") {
                content.replace_range(style_start..style_start + style_end + 8, "");
            } else {
                break;
            }
        }
        
        // Replace HTML tags with spaces
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        let content = re.replace_all(&content, " ").to_string();
        
        // Replace multiple spaces with a single space
        let re = regex::Regex::new(r"\s+").unwrap();
        let content = re.replace_all(&content, " ").to_string();
        
        // Decode HTML entities
        let content = html_escape::decode_html_entities(&content).to_string();
        
        (title, content.trim().to_string())
    }
}

// Tauri commands for documentation scraping

#[tauri::command]
pub async fn scrape_documentation(
    request: ScrapingRequest,
    doc_scraper: State<'_, SharedDocScraper>,
) -> Result<ScrapingResult, String> {
    let scraper = doc_scraper.lock().await;
    
    scraper
        .scrape_documentation(&request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_scrape_documentation(
    request: BatchScrapingRequest,
    doc_scraper: State<'_, SharedDocScraper>,
) -> Result<BatchScrapingResult, String> {
    let scraper = doc_scraper.lock().await;
    
    scraper
        .batch_scrape_documentation(&request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_documentation(
    request: DocumentationSearchRequest,
    doc_scraper: State<'_, SharedDocScraper>,
) -> Result<Vec<QueryResult>, String> {
    let scraper = doc_scraper.lock().await;
    
    scraper
        .search_documentation(&request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scrape_from_search(
    query: String,
    collection_name: String,
    document_type: String,
    max_results: usize,
    doc_scraper: State<'_, SharedDocScraper>,
    searxng_client: State<'_, Arc<Mutex<SearXNGClient>>>,
) -> Result<BatchScrapingResult, String> {
    let scraper = doc_scraper.lock().await;
    let search_client = searxng_client.lock().await;
    
    scraper
        .scrape_from_search(&search_client, &query, &collection_name, &document_type, max_results)
        .await
        .map_err(|e| e.to_string())
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::State;
use tokio::sync::RwLock;

/// Conservative token estimation multiplier for safety margin
const TOKEN_SAFETY_MULTIPLIER: f32 = 1.2;

/// Context Manager handles token budget allocation and context building
#[derive(Debug)]
pub struct ContextManager {
    pub max_tokens: usize,
    pub reserved_tokens: usize,
    pub pinned_files: RwLock<Vec<String>>,
    pub token_cache: RwLock<HashMap<String, usize>>,
}

/// Context budget breakdown for UI visualization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextBudget {
    pub total: usize,
    pub used: usize,
    pub available: usize,
    pub breakdown: BudgetBreakdown,
}

/// Detailed breakdown of token allocation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BudgetBreakdown {
    pub conversation: usize,
    pub rag_documents: usize,
    pub pinned_files: usize,
    pub suggested_files: usize,
    pub reserved: usize,
}

/// File context information for UI display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextFile {
    pub path: String,
    pub token_count: usize,
    pub relevance_score: f32,
    pub is_pinned: bool,
    pub file_type: String,
}

/// Context building result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuiltContext {
    pub files: Vec<ContextFile>,
    pub total_tokens: usize,
    pub budget: ContextBudget,
}

impl ContextManager {
    /// Create a new ContextManager with default settings
    pub fn new(max_tokens: usize, reserved_tokens: usize) -> Self {
        Self {
            max_tokens,
            reserved_tokens,
            pinned_files: RwLock::new(Vec::new()),
            token_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Calculate current context budget allocation
    pub async fn calculate_budget(&self, conversation_tokens: usize, rag_tokens: usize) -> ContextBudget {
        let pinned_files = self.pinned_files.read().await;
        let token_cache = self.token_cache.read().await;
        
        // Calculate pinned files tokens
        let pinned_tokens: usize = pinned_files
            .iter()
            .map(|path| token_cache.get(path).copied().unwrap_or(0))
            .sum();

        // Calculate available tokens for suggestions
        let used_tokens = conversation_tokens + rag_tokens + pinned_tokens + self.reserved_tokens;
        let available = self.max_tokens.saturating_sub(used_tokens);
        let suggested_files = available.min(self.max_tokens / 8); // Max 12.5% for suggestions

        let breakdown = BudgetBreakdown {
            conversation: conversation_tokens,
            rag_documents: rag_tokens,
            pinned_files: pinned_tokens,
            suggested_files,
            reserved: self.reserved_tokens,
        };

        ContextBudget {
            total: self.max_tokens,
            used: used_tokens,
            available,
            breakdown,
        }
    }

    /// Count tokens in text using conservative estimation
    pub fn count_tokens(&self, text: &str) -> usize {
        // Simple word-based estimation with safety multiplier
        // In production, this should use the actual model's tokenizer
        let word_count = text.split_whitespace().count();
        let estimated_tokens = (word_count as f32 * 1.3) as usize; // ~1.3 tokens per word average
        (estimated_tokens as f32 * TOKEN_SAFETY_MULTIPLIER) as usize
    }

    /// Count tokens in a file and cache the result
    pub async fn count_file_tokens(&self, file_path: &str) -> Result<usize, String> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(&cached_count) = cache.get(file_path) {
                return Ok(cached_count);
            }
        }

        // Read file and count tokens
        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

        let token_count = self.count_tokens(&content);

        // Cache the result
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(file_path.to_string(), token_count);
        }

        Ok(token_count)
    }

    /// Pin a file to always include in context
    pub async fn pin_file(&self, file_path: String) -> Result<(), String> {
        let mut pinned = self.pinned_files.write().await;
        if !pinned.contains(&file_path) {
            pinned.push(file_path);
        }
        Ok(())
    }

    /// Unpin a file from context
    pub async fn unpin_file(&self, file_path: String) -> Result<(), String> {
        let mut pinned = self.pinned_files.write().await;
        pinned.retain(|path| path != &file_path);
        Ok(())
    }

    /// Check if a file is pinned
    pub async fn is_file_pinned(&self, file_path: &str) -> bool {
        let pinned = self.pinned_files.read().await;
        pinned.contains(&file_path.to_string())
    }

    /// Calculate relevance score for a file (mocked for MVP)
    pub async fn calculate_file_relevance(&self, file_path: &str, _conversation_context: &str) -> f32 {
        // Mock relevance scoring for MVP - returns random score between 60-95%
        // In production, this would use embeddings and semantic similarity
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate consistent "random" score between 0.6 and 0.95
        let normalized = (hash % 36) as f32 / 100.0; // 0.0 to 0.35
        0.60 + normalized // 0.60 to 0.95
    }

    /// Get file type from extension
    pub fn get_file_type(&self, file_path: &str) -> String {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Build context with current files and budget
    pub async fn build_context(&self, 
        conversation_context: &str,
        rag_tokens: usize,
        suggested_files: Vec<String>
    ) -> Result<BuiltContext, String> {
        let conversation_tokens = self.count_tokens(conversation_context);
        let budget = self.calculate_budget(conversation_tokens, rag_tokens).await;
        
        let mut context_files = Vec::new();
        
        // Add pinned files first
        let pinned_files = self.pinned_files.read().await.clone();
        for file_path in &pinned_files {
            if let Ok(token_count) = self.count_file_tokens(file_path).await {
                let relevance = self.calculate_file_relevance(file_path, conversation_context).await;
                context_files.push(ContextFile {
                    path: file_path.clone(),
                    token_count,
                    relevance_score: relevance,
                    is_pinned: true,
                    file_type: self.get_file_type(file_path),
                });
            }
        }
        
        // Add suggested files within budget
        let mut remaining_budget = budget.breakdown.suggested_files;
        for file_path in suggested_files {
            if remaining_budget == 0 {
                break;
            }
            
            // Skip if already pinned
            if pinned_files.contains(&file_path) {
                continue;
            }
            
            if let Ok(token_count) = self.count_file_tokens(&file_path).await {
                if token_count <= remaining_budget {
                    let relevance = self.calculate_file_relevance(&file_path, conversation_context).await;
                    context_files.push(ContextFile {
                        path: file_path.clone(),
                        token_count,
                        relevance_score: relevance,
                        is_pinned: false,
                        file_type: self.get_file_type(&file_path),
                    });
                    remaining_budget = remaining_budget.saturating_sub(token_count);
                }
            }
        }
        
        // Sort by relevance score (highest first)
        context_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let total_tokens: usize = context_files.iter().map(|f| f.token_count).sum();
        
        Ok(BuiltContext {
            files: context_files,
            total_tokens: total_tokens + conversation_tokens + rag_tokens,
            budget,
        })
    }

    /// Clear token cache (useful when files are modified)
    pub async fn clear_cache(&self) {
        let mut cache = self.token_cache.write().await;
        cache.clear();
    }

    /// Get current pinned files
    pub async fn get_pinned_files(&self) -> Vec<String> {
        self.pinned_files.read().await.clone()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        // Default to 128k tokens with 25k reserved
        Self::new(128_000, 25_600)
    }
}

/// Tauri command to get current context budget
#[tauri::command]
pub async fn get_context_budget(
    manager: State<'_, ContextManager>,
    conversation_tokens: Option<usize>,
    rag_tokens: Option<usize>,
) -> Result<ContextBudget, String> {
    let budget = manager.calculate_budget(
        conversation_tokens.unwrap_or(0),
        rag_tokens.unwrap_or(0)
    ).await;
    Ok(budget)
}

/// Tauri command to pin a file
#[tauri::command]
pub async fn pin_file(
    manager: State<'_, ContextManager>,
    file_path: String,
) -> Result<(), String> {
    manager.pin_file(file_path).await
}

/// Tauri command to unpin a file
#[tauri::command]
pub async fn unpin_file(
    manager: State<'_, ContextManager>,
    file_path: String,
) -> Result<(), String> {
    manager.unpin_file(file_path).await
}

/// Tauri command to calculate file relevance
#[tauri::command]
pub async fn calculate_file_relevance(
    manager: State<'_, ContextManager>,
    file_path: String,
    conversation_context: String,
) -> Result<f32, String> {
    let relevance = manager.calculate_file_relevance(&file_path, &conversation_context).await;
    Ok(relevance)
}

/// Tauri command to build context
#[tauri::command]
pub async fn build_context(
    manager: State<'_, ContextManager>,
    conversation_context: String,
    rag_tokens: Option<usize>,
    suggested_files: Vec<String>,
) -> Result<BuiltContext, String> {
    manager.build_context(
        &conversation_context,
        rag_tokens.unwrap_or(0),
        suggested_files
    ).await
}

/// Tauri command to get pinned files
#[tauri::command]
pub async fn get_pinned_files(
    manager: State<'_, ContextManager>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_pinned_files().await)
}

/// Tauri command to count file tokens
#[tauri::command]
pub async fn count_file_tokens(
    manager: State<'_, ContextManager>,
    file_path: String,
) -> Result<usize, String> {
    manager.count_file_tokens(&file_path).await
}

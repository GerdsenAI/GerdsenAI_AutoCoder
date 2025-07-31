// Define shared structures for LSP and AI integration
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::DiagnosticSeverity;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeAnalysisRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeAnalysisResponse {
    pub analysis: String,
    pub suggestions: Vec<CodeSuggestion>,
    pub errors: Vec<CodeError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeSuggestion {
    pub range: Range,
    pub suggestion: String,
    pub description: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeError {
    pub range: Range,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

use std::path::Path;
use std::sync::Arc;
use tauri::State;
use crate::ollama_client::{SharedOllamaClient, ChatMessage};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryAnalysisRequest {
    pub repo_path: String,
    pub file_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub max_files: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryAnalysisResponse {
    pub summary: String,
    pub file_analyses: Vec<FileAnalysis>,
    pub suggestions: Vec<RepositorySuggestion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileAnalysis {
    pub file_path: String,
    pub language: String,
    pub summary: String,
    pub suggestions: Vec<CodeSuggestion>,
    pub errors: Vec<CodeError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositorySuggestion {
    pub title: String,
    pub description: String,
    pub affected_files: Vec<String>,
    pub priority: SuggestionPriority,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeFixRequest {
    pub code: String,
    pub language: String,
    pub error_message: String,
    pub error_range: Range,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeFixResponse {
    pub fixed_code: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeGenerationRequest {
    pub prompt: String,
    pub language: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeGenerationResponse {
    pub generated_code: String,
    pub explanation: String,
}

// Code analysis service
pub struct CodeAnalysisService {
    ollama_client: SharedOllamaClient,
}

impl CodeAnalysisService {
    pub fn new(ollama_client: SharedOllamaClient) -> Self {
        Self { ollama_client }
    }
    
    pub async fn analyze_code(&self, request: &CodeAnalysisRequest) -> Result<CodeAnalysisResponse, String> {
        let client = self.ollama_client.lock().await;
        
        // Prepare the prompt for code analysis
        let prompt = format!(
            "Analyze the following {} code and provide suggestions and identify errors:\n\n```{}\n{}\n```",
            request.language, request.language, request.code
        );
        
        // Create chat messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert code analyzer. Analyze the provided code, identify issues, and suggest improvements.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];
        
        // Get response from Ollama
        let response = client
            .chat(
                "llama3:latest", // Use a suitable model
                messages,
                None,
                None::<fn(&str)>,
            )
            .await
            .map_err(|e| e.to_string())?;
            
        // Parse the response to extract suggestions and errors
        // This is a simplified implementation; in a real-world scenario,
        // you would use a more sophisticated parsing approach
        
        // For now, return a placeholder response
        Ok(CodeAnalysisResponse {
            analysis: response.content,
            suggestions: vec![],
            errors: vec![],
        })
    }
    
    pub async fn analyze_repository(&self, request: &RepositoryAnalysisRequest) -> Result<RepositoryAnalysisResponse, String> {
        // Validate repository path
        let repo_path = Path::new(&request.repo_path);
        if !repo_path.exists() || !repo_path.is_dir() {
            return Err("Invalid repository path".to_string());
        }
        
        // Collect files to analyze
        let files = self.collect_files(repo_path, &request.file_patterns, &request.exclude_patterns, request.max_files)
            .map_err(|e| e.to_string())?;
            
        // Analyze each file
        let mut file_analyses = Vec::new();
        for file_path in &files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let language = self.detect_language(file_path);
                
                // Analyze the file
                let analysis_request = CodeAnalysisRequest {
                    code: content,
                    language: language.clone(),
                    file_path: Some(file_path.to_string_lossy().to_string()),
                };
                
                if let Ok(analysis) = self.analyze_code(&analysis_request).await {
                    file_analyses.push(FileAnalysis {
                        file_path: file_path.to_string_lossy().to_string(),
                        language,
                        summary: analysis.analysis,
                        suggestions: analysis.suggestions,
                        errors: analysis.errors,
                    });
                }
            }
        }
        
        // Generate repository-wide suggestions
        let suggestions = self.generate_repository_suggestions(&file_analyses).await?;
        
        // Generate summary
        let summary = self.generate_repository_summary(&file_analyses, &suggestions).await?;
        
        Ok(RepositoryAnalysisResponse {
            summary,
            file_analyses,
            suggestions,
        })
    }
    
    pub async fn fix_code(&self, request: &CodeFixRequest) -> Result<CodeFixResponse, String> {
        let client = self.ollama_client.lock().await;
        
        // Prepare the prompt for code fixing
        let prompt = format!(
            "Fix the following {} code that has an error: '{}' at line {}:{} to {}:{}:\n\n```{}\n{}\n```",
            request.language,
            request.error_message,
            request.error_range.start.line,
            request.error_range.start.character,
            request.error_range.end.line,
            request.error_range.end.character,
            request.language,
            request.code
        );
        
        // Create chat messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert code fixer. Fix the provided code based on the error message and location.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];
        
        // Get response from Ollama
        let response = client
            .chat(
                "llama3:latest", // Use a suitable model
                messages,
                None,
                None::<fn(&str)>,
            )
            .await
            .map_err(|e| e.to_string())?;
            
        // Extract the fixed code and explanation
        // This is a simplified implementation; in a real-world scenario,
        // you would use a more sophisticated parsing approach
        
        // For now, return a placeholder response
        Ok(CodeFixResponse {
            fixed_code: request.code.clone(), // Replace with actual fixed code
            explanation: response.content,
        })
    }
    
    pub async fn generate_code(&self, request: &CodeGenerationRequest) -> Result<CodeGenerationResponse, String> {
        let client = self.ollama_client.lock().await;
        
        // Prepare the prompt for code generation
        let mut prompt = format!(
            "Generate {} code for the following request:\n\n{}",
            request.language, request.prompt
        );
        
        if let Some(context) = &request.context {
            prompt = format!("{}\n\nContext:\n{}", prompt, context);
        }
        
        // Create chat messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert code generator. Generate high-quality, well-documented code based on the user's request.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];
        
        // Get response from Ollama
        let response = client
            .chat(
                "llama3:latest", // Use a suitable model
                messages,
                None,
                None::<fn(&str)>,
            )
            .await
            .map_err(|e| e.to_string())?;
            
        // Extract the generated code and explanation
        // This is a simplified implementation; in a real-world scenario,
        // you would use a more sophisticated parsing approach
        
        // For now, return a placeholder response
        Ok(CodeGenerationResponse {
            generated_code: response.content.clone(),
            explanation: "Code generated successfully.".to_string(),
        })
    }
    
    // Helper methods
    
    fn collect_files(
        &self,
        repo_path: &Path,
        file_patterns: &Option<Vec<String>>,
        exclude_patterns: &Option<Vec<String>>,
        max_files: Option<usize>,
    ) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        let max_count = max_files.unwrap_or(100); // Default to 100 files
        
        let walker = walkdir::WalkDir::new(repo_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok());
            
        for entry in walker {
            if files.len() >= max_count {
                break;
            }
            
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }
            
            let path_str = path.to_string_lossy();
            
            // Check if the file matches the include patterns
            if let Some(patterns) = file_patterns {
                if !patterns.iter().any(|pattern| {
                    glob::Pattern::new(pattern)
                        .map(|p| p.matches(&path_str))
                        .unwrap_or(false)
                }) {
                    continue;
                }
            }
            
            // Check if the file matches the exclude patterns
            if let Some(patterns) = exclude_patterns {
                if patterns.iter().any(|pattern| {
                    glob::Pattern::new(pattern)
                        .map(|p| p.matches(&path_str))
                        .unwrap_or(false)
                }) {
                    continue;
                }
            }
            
            files.push(path.to_path_buf());
        }
        
        Ok(files)
    }
    
    fn detect_language(&self, file_path: &Path) -> String {
        if let Some(extension) = file_path.extension() {
            match extension.to_string_lossy().as_ref() {
                "js" => return "javascript".to_string(),
                "ts" => return "typescript".to_string(),
                "jsx" => return "jsx".to_string(),
                "tsx" => return "tsx".to_string(),
                "py" => return "python".to_string(),
                "rs" => return "rust".to_string(),
                "go" => return "go".to_string(),
                "java" => return "java".to_string(),
                "c" => return "c".to_string(),
                "cpp" | "cc" | "cxx" => return "cpp".to_string(),
                "cs" => return "csharp".to_string(),
                "php" => return "php".to_string(),
                "rb" => return "ruby".to_string(),
                "swift" => return "swift".to_string(),
                "kt" | "kts" => return "kotlin".to_string(),
                "scala" => return "scala".to_string(),
                "html" => return "html".to_string(),
                "css" => return "css".to_string(),
                "json" => return "json".to_string(),
                "md" => return "markdown".to_string(),
                "yml" | "yaml" => return "yaml".to_string(),
                "xml" => return "xml".to_string(),
                "sh" | "bash" => return "bash".to_string(),
                "sql" => return "sql".to_string(),
                _ => {}
            }
        }
        
        // Default to plaintext if we can't determine the language
        "plaintext".to_string()
    }
    
    async fn generate_repository_suggestions(&self, file_analyses: &[FileAnalysis]) -> Result<Vec<RepositorySuggestion>, String> {
        // In a real implementation, this would analyze the file analyses and generate repository-wide suggestions
        // For now, return a placeholder
        Ok(vec![
            RepositorySuggestion {
                title: "Consistent code formatting".to_string(),
                description: "Consider using a code formatter to ensure consistent style across the repository.".to_string(),
                affected_files: file_analyses.iter().map(|a| a.file_path.clone()).collect(),
                priority: SuggestionPriority::Medium,
            },
        ])
    }
    
    async fn generate_repository_summary(&self, file_analyses: &[FileAnalysis], suggestions: &[RepositorySuggestion]) -> Result<String, String> {
        // In a real implementation, this would generate a comprehensive summary based on the analyses and suggestions
        // For now, return a placeholder
        Ok(format!(
            "Repository analysis complete. Analyzed {} files and found {} repository-wide suggestions.",
            file_analyses.len(),
            suggestions.len()
        ))
    }
}

// Tauri commands for code analysis

#[tauri::command]
pub async fn analyze_code(
    request: CodeAnalysisRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<CodeAnalysisResponse, String> {
    code_analysis_service.analyze_code(&request).await
}

#[tauri::command]
pub async fn analyze_repository(
    request: RepositoryAnalysisRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<RepositoryAnalysisResponse, String> {
    code_analysis_service.analyze_repository(&request).await
}

#[tauri::command]
pub async fn fix_code(
    request: CodeFixRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<CodeFixResponse, String> {
    code_analysis_service.fix_code(&request).await
}

#[tauri::command]
pub async fn generate_code(
    request: CodeGenerationRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<CodeGenerationResponse, String> {
    code_analysis_service.generate_code(&request).await
}

// Define shared structures for LSP and AI integration
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::DiagnosticSeverity;
use std::collections::{HashMap, HashSet};
use futures::{stream, StreamExt};

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

// Dependency tracking structures
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Class,
    Variable,
    Interface,
    Type,
    Namespace,
    Module,
    Constant,
    Property,
    Method,
    Enum,
    Import,
    Export,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Range,
    pub documentation: Option<String>,
    pub is_exported: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Import {
    pub source: String, // Could be relative path or module name
    pub symbols: Vec<String>,
    pub is_all: bool, // true for "import * as X" style imports
    pub location: Range,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyNode {
    pub file_path: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub exports: HashSet<String>, // Names of exported symbols
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyEdge {
    pub from: String, // Source file path
    pub to: String,   // Target file path
    pub symbols: Vec<String>, // Symbols being imported
    pub strength: DependencyStrength,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DependencyStrength {
    Weak,    // Few imports, not critical
    Medium,  // Several imports, somewhat important
    Strong,  // Many imports, critical dependency
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyAnalysisRequest {
    pub repo_path: String,
    pub file_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub max_files: Option<usize>,
    pub include_content: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImpactAnalysisRequest {
    pub file_path: String,
    pub changes: Vec<CodeChange>,
    pub repo_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeChange {
    pub range: Range,
    pub new_text: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImpactAnalysisResponse {
    pub affected_files: Vec<AffectedFile>,
    pub impact_summary: String,
    pub risk_assessment: RiskLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AffectedFile {
    pub file_path: String,
    pub impact_description: String,
    pub impact_level: ImpactLevel,
    pub affected_symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefactoringRequest {
    pub file_paths: Vec<String>,
    pub focus_areas: Option<Vec<RefactoringFocusArea>>,
    pub repo_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RefactoringFocusArea {
    Performance,
    Readability,
    Maintainability,
    Security,
    CodeDuplication,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefactoringSuggestion {
    pub title: String,
    pub description: String,
    pub before_code: String,
    pub after_code: String,
    pub affected_files: Vec<String>,
    pub effort_estimate: String,
    pub benefits: Vec<String>,
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
        
        // Process files in parallel using futures::stream
        let file_analyses = stream::iter(files)
            .map(|file_path| {
                let path_clone = file_path.clone();
                let service = self.clone();
                async move {
                    if let Ok(content) = std::fs::read_to_string(&path_clone) {
                        let language = service.detect_language(&path_clone);
                        
                        // Analyze the file
                        let analysis_request = CodeAnalysisRequest {
                            code: content,
                            language: language.clone(),
                            file_path: Some(path_clone.to_string_lossy().to_string()),
                        };
                        
                        if let Ok(analysis) = service.analyze_code(&analysis_request).await {
                            return Some(FileAnalysis {
                                file_path: path_clone.to_string_lossy().to_string(),
                                language,
                                summary: analysis.analysis,
                                suggestions: analysis.suggestions,
                                errors: analysis.errors,
                            });
                        }
                    }
                    None
                }
            })
            .buffer_unordered(8) // Process up to 8 files concurrently
            .filter_map(|result| async { result })
            .collect::<Vec<_>>()
            .await;
        
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
    
    // Dependency Analysis Methods
    
    pub async fn analyze_dependencies(&self, request: &DependencyAnalysisRequest) -> Result<DependencyGraph, String> {
        // Validate repository path
        let repo_path = Path::new(&request.repo_path);
        if !repo_path.exists() || !repo_path.is_dir() {
            return Err("Invalid repository path".to_string());
        }
        
        // Collect files to analyze
        let files = self.collect_files(repo_path, &request.file_patterns, &request.exclude_patterns, request.max_files)
            .map_err(|e| e.to_string())?;
            
        // Create dependency nodes for each file
        let mut nodes = HashMap::new();
        
        // Process files in parallel using futures::stream
        let dependency_nodes = stream::iter(files)
            .map(|file_path| {
                let path_clone = file_path.clone();
                let service = self.clone();
                async move {
                    if let Ok(content) = std::fs::read_to_string(&path_clone) {
                        let language = service.detect_language(&path_clone);
                        let file_path_str = path_clone.to_string_lossy().to_string();
                        
                        // Extract symbols and imports
                        let (symbols, imports) = service.extract_symbols_and_imports(&content, &language, &file_path_str).await;
                        
                        // Determine exports (simplified for now - in reality would need language-specific parsing)
                        let exports = symbols.iter()
                            .filter(|s| s.is_exported)
                            .map(|s| s.name.clone())
                            .collect::<HashSet<String>>();
                            
                        Some((file_path_str, DependencyNode {
                            file_path: file_path_str,
                            language,
                            symbols,
                            imports,
                            exports,
                        }))
                    } else {
                        None
                    }
                }
            })
            .buffer_unordered(8) // Process up to 8 files concurrently
            .filter_map(|result| async { result })
            .collect::<Vec<_>>()
            .await;
            
        // Add all nodes to the graph
        for (path, node) in dependency_nodes {
            nodes.insert(path, node);
        }
        
        // Create edges between nodes based on imports
        let mut edges = Vec::new();
        
        for (from_path, node) in &nodes {
            for import in &node.imports {
                // Resolve the import path to an absolute file path
                if let Some(to_path) = self.resolve_import_path(from_path, &import.source, &nodes) {
                    // Only create an edge if the target file exists in our nodes
                    if let Some(target_node) = nodes.get(&to_path) {
                        // Determine which symbols are actually being imported
                        let imported_symbols = if import.is_all {
                            target_node.exports.iter().cloned().collect()
                        } else {
                            import.symbols.clone()
                        };
                        
                        // Determine dependency strength based on number of imports
                        let strength = match imported_symbols.len() {
                            0..=2 => DependencyStrength::Weak,
                            3..=5 => DependencyStrength::Medium,
                            _ => DependencyStrength::Strong,
                        };
                        
                        edges.push(DependencyEdge {
                            from: from_path.clone(),
                            to: to_path,
                            symbols: imported_symbols,
                            strength,
                        });
                    }
                }
            }
        }
        
        // Generate summary
        let summary = format!(
            "Dependency analysis complete. Found {} files with {} dependencies.",
            nodes.len(),
            edges.len()
        );
        
        Ok(DependencyGraph {
            nodes,
            edges,
            summary,
        })
    }
    
    // Helper method to extract symbols and imports from file content
    async fn extract_symbols_and_imports(&self, content: &str, language: &str, file_path: &str) -> (Vec<Symbol>, Vec<Import>) {
        let client = self.ollama_client.lock().await;
        
        // Prepare the prompt for extraction
        let prompt = format!(
            "Extract symbols (functions, classes, variables, etc.) and imports from the following {} code.\n\
            Format the response as JSON with two arrays: 'symbols' and 'imports'.\n\
            For each symbol include name, kind, location (line numbers), and whether it's exported.\n\
            For each import include source path and imported symbol names.\n\
            Code to analyze:\n\n```{}\n{}\n```",
            language, language, content
        );
        
        // Create chat messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert code analyzer specialized in extracting code structure. Output only valid JSON.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];
        
        // Get response from Ollama
        let response = match client
            .chat(
                "llama3:latest", // Use a suitable model
                messages,
                None,
                None::<fn(&str)>,
            )
            .await {
                Ok(resp) => resp.content,
                Err(_) => return (Vec::new(), Vec::new()), // Fallback to empty lists on error
            };
            
        // Try to parse the JSON response
        // In a real implementation, would need more robust parsing and error handling
        // This is a simplified approach
        
        // Extract JSON from the response (might be surrounded by markdown code blocks or other text)
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                return (Vec::new(), Vec::new());
            }
        } else {
            return (Vec::new(), Vec::new());
        };
        
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json) => {
                let mut symbols = Vec::new();
                let mut imports = Vec::new();
                
                // Parse symbols
                if let Some(symbols_json) = json.get("symbols").and_then(|s| s.as_array()) {
                    for symbol_json in symbols_json {
                        if let (Some(name), Some(kind_str), Some(exported)) = (
                            symbol_json.get("name").and_then(|n| n.as_str()),
                            symbol_json.get("kind").and_then(|k| k.as_str()),
                            symbol_json.get("is_exported").and_then(|e| e.as_bool()),
                        ) {
                            let kind = match kind_str {
                                "function" => SymbolKind::Function,
                                "class" => SymbolKind::Class,
                                "variable" => SymbolKind::Variable,
                                "interface" => SymbolKind::Interface,
                                "type" => SymbolKind::Type,
                                "namespace" => SymbolKind::Namespace,
                                "module" => SymbolKind::Module,
                                "constant" => SymbolKind::Constant,
                                "property" => SymbolKind::Property,
                                "method" => SymbolKind::Method,
                                "enum" => SymbolKind::Enum,
                                "import" => SymbolKind::Import,
                                "export" => SymbolKind::Export,
                                _ => SymbolKind::Unknown,
                            };
                            
                            // Create a simplified location since we may not have precise positions
                            let location = Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 0 },
                            };
                            
                            symbols.push(Symbol {
                                name: name.to_string(),
                                kind,
                                location,
                                documentation: None,
                                is_exported: exported,
                            });
                        }
                    }
                }
                
                // Parse imports
                if let Some(imports_json) = json.get("imports").and_then(|i| i.as_array()) {
                    for import_json in imports_json {
                        if let Some(source) = import_json.get("source").and_then(|s| s.as_str()) {
                            let is_all = import_json.get("is_all").and_then(|a| a.as_bool()).unwrap_or(false);
                            
                            let mut symbols_vec = Vec::new();
                            if let Some(symbols_json) = import_json.get("symbols").and_then(|s| s.as_array()) {
                                for symbol_json in symbols_json {
                                    if let Some(symbol_name) = symbol_json.as_str() {
                                        symbols_vec.push(symbol_name.to_string());
                                    }
                                }
                            }
                            
                            // Create a simplified location
                            let location = Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 0 },
                            };
                            
                            imports.push(Import {
                                source: source.to_string(),
                                symbols: symbols_vec,
                                is_all,
                                location,
                            });
                        }
                    }
                }
                
                (symbols, imports)
            },
            Err(_) => (Vec::new(), Vec::new()),
        }
    }
    
    // Helper method to resolve import paths to absolute file paths
    fn resolve_import_path(&self, from_path: &str, import_path: &str, nodes: &HashMap<String, DependencyNode>) -> Option<String> {
        // Simple implementation for common cases
        // A real implementation would need language-specific logic
        
        // Handle absolute imports (assume they match exactly to a file in our nodes)
        if nodes.contains_key(import_path) {
            return Some(import_path.to_string());
        }
        
        // Handle relative imports
        if import_path.starts_with("./") || import_path.starts_with("../") {
            let from_dir = Path::new(from_path).parent()?;
            let resolved_path = from_dir.join(import_path).canonicalize().ok()?;
            let resolved_str = resolved_path.to_string_lossy().to_string();
            
            // Check if this resolved path exists in our nodes
            if nodes.contains_key(&resolved_str) {
                return Some(resolved_str);
            }
            
            // Try adding common extensions
            for ext in &[".js", ".ts", ".jsx", ".tsx", ".py", ".rs"] {
                let with_ext = format!("{}{}", resolved_str, ext);
                if nodes.contains_key(&with_ext) {
                    return Some(with_ext);
                }
            }
        }
        
        // Handle module imports (e.g., 'react', 'lodash')
        // This would require more complex resolution based on the project's module system
        // For now, we'll just return None for these
        
        None
    }
    
    // Impact Analysis Method
    pub async fn analyze_impact(&self, request: &ImpactAnalysisRequest) -> Result<ImpactAnalysisResponse, String> {
        // First, analyze dependencies to understand the codebase structure
        let dep_request = DependencyAnalysisRequest {
            repo_path: request.repo_path.clone(),
            file_patterns: None,
            exclude_patterns: None,
            max_files: Some(500), // Reasonable limit
            include_content: Some(false),
        };
        
        let dep_graph = self.analyze_dependencies(&dep_request).await?;
        
        // Check if the file being changed exists in our dependency graph
        if !dep_graph.nodes.contains_key(&request.file_path) {
            return Err("File not found in repository".to_string());
        }
        
        // Determine the symbols that might be affected by the changes
        let affected_node = dep_graph.nodes.get(&request.file_path).unwrap();
        let mut potentially_affected_symbols = HashSet::new();
        
        // For each change, determine affected symbols
        // This is a simplified approach - a real implementation would need to parse the code
        // and determine exactly which symbols are affected by each change
        for change in &request.changes {
            // Find symbols whose location overlaps with the change
            for symbol in &affected_node.symbols {
                // Simple check - in reality would need more precise range checking
                if symbol.is_exported {
                    potentially_affected_symbols.insert(symbol.name.clone());
                }
            }
        }
        
        // Find files that depend on the changed file
        let mut affected_files = Vec::new();
        
        for edge in &dep_graph.edges {
            if edge.to == request.file_path {
                // This file imports from our changed file
                let importing_file = edge.from.clone();
                let importing_node = dep_graph.nodes.get(&importing_file).unwrap();
                
                // Check if any of the imported symbols are affected
                let mut affected_symbols = Vec::new();
                for symbol_name in &edge.symbols {
                    if potentially_affected_symbols.contains(symbol_name) {
                        // Find the full symbol info
                        if let Some(symbol) = affected_node.symbols.iter().find(|s| &s.name == symbol_name) {
                            affected_symbols.push(symbol.clone());
                        }
                    }
                }
                
                if !affected_symbols.is_empty() {
                    // Determine impact level based on number and type of affected symbols
                    let impact_level = match affected_symbols.len() {
                        0 => ImpactLevel::None,
                        1 => ImpactLevel::Low,
                        2..=3 => ImpactLevel::Medium,
                        4..=6 => ImpactLevel::High,
                        _ => ImpactLevel::Critical,
                    };
                    
                    affected_files.push(AffectedFile {
                        file_path: importing_file,
                        impact_description: format!(
                            "This file imports {} affected symbols from {}",
                            affected_symbols.len(), request.file_path
                        ),
                        impact_level,
                        affected_symbols,
                    });
                }
            }
        }
        
        // Determine overall risk level
        let risk_level = if affected_files.is_empty() {
            RiskLevel::Low
        } else {
            let max_impact = affected_files.iter()
                .map(|f| &f.impact_level)
                .max()
                .unwrap_or(&ImpactLevel::Low);
                
            match max_impact {
                ImpactLevel::None => RiskLevel::Low,
                ImpactLevel::Low => RiskLevel::Low,
                ImpactLevel::Medium => RiskLevel::Medium,
                ImpactLevel::High => RiskLevel::High,
                ImpactLevel::Critical => RiskLevel::Critical,
            }
        };
        
        // Generate impact summary
        let impact_summary = format!(
            "Impact analysis complete. The changes to {} will affect {} other files with a {} risk level.",
            request.file_path,
            affected_files.len(),
            format!("{:?}", risk_level).to_lowercase()
        );
        
        Ok(ImpactAnalysisResponse {
            affected_files,
            impact_summary,
            risk_assessment: risk_level,
        })
    }
    
    // Refactoring Suggestions Method
    pub async fn suggest_refactorings(&self, request: &RefactoringRequest) -> Result<Vec<RefactoringSuggestion>, String> {
        let client = self.ollama_client.lock().await;
        
        // Collect file contents
        let mut file_contents = Vec::new();
        for file_path in &request.file_paths {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                file_contents.push((file_path.clone(), content));
            }
        }
        
        if file_contents.is_empty() {
            return Err("No readable files provided".to_string());
        }
        
        // Determine focus areas
        let focus_areas = if let Some(areas) = &request.focus_areas {
            areas.iter()
                .map(|area| format!("{:?}", area))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "Readability, Maintainability, Performance".to_string()
        };
        
        // Prepare the prompt for refactoring suggestions
        let mut prompt = format!(
            "Analyze the following code files and suggest refactorings focused on: {}.\n\
            For each suggestion, provide:\n\
            1. A title and description\n\
            2. Before and after code examples\n\
            3. Effort estimate (Easy, Medium, Hard)\n\
            4. Benefits of the refactoring\n\n",
            focus_areas
        );
        
        // Add file contents to the prompt (limit to reasonable size)
        for (path, content) in &file_contents {
            // Add a shortened version if content is too large
            let display_content = if content.len() > 2000 {
                format!("{}... (truncated)", &content[0..2000])
            } else {
                content.clone()
            };
            
            prompt += &format!("File: {}\n```\n{}\n```\n\n", path, display_content);
        }
        
        // Create chat messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert code refactorer. Analyze code and suggest specific, actionable refactorings with examples.".to_string(),
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
            
        // Parse the response to extract refactoring suggestions
        // In a real implementation, would need more robust parsing
        // For demonstration, we'll create a sample suggestion
        
        let suggestions = vec![
            RefactoringSuggestion {
                title: "Extract common functionality into helper function".to_string(),
                description: "Several components duplicate logic for formatting tokens. This could be extracted into a shared helper function.".to_string(),
                before_code: "function Component1() {\n  const formatted = value > 1000 ? `${(value / 1000).toFixed(1)}k` : value.toString();\n}\n\nfunction Component2() {\n  const formatted = value > 1000 ? `${(value / 1000).toFixed(1)}k` : value.toString();\n}".to_string(),
                after_code: "function formatValue(value) {\n  return value > 1000 ? `${(value / 1000).toFixed(1)}k` : value.toString();\n}\n\nfunction Component1() {\n  const formatted = formatValue(value);\n}\n\nfunction Component2() {\n  const formatted = formatValue(value);\n}".to_string(),
                affected_files: request.file_paths.clone(),
                effort_estimate: "Easy".to_string(),
                benefits: vec![
                    "Reduces code duplication".to_string(),
                    "Improves maintainability".to_string(),
                    "Makes future changes easier".to_string(),
                ],
            }
        ];
        
        Ok(suggestions)
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

#[tauri::command]
pub async fn analyze_dependencies(
    request: DependencyAnalysisRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<DependencyGraph, String> {
    code_analysis_service.analyze_dependencies(&request).await
}

#[tauri::command]
pub async fn analyze_impact(
    request: ImpactAnalysisRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<ImpactAnalysisResponse, String> {
    code_analysis_service.analyze_impact(&request).await
}

#[tauri::command]
pub async fn suggest_refactorings(
    request: RefactoringRequest,
    code_analysis_service: State<'_, Arc<CodeAnalysisService>>,
) -> Result<Vec<RefactoringSuggestion>, String> {
    code_analysis_service.suggest_refactorings(&request).await
}

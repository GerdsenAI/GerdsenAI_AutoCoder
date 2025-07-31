use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use std::time::{Duration, Instant};
use crate::code_analysis::{CodeAnalysisService, CodeAnalysisResponse};

// Performance optimization structures
#[derive(Debug)]
pub struct DebouncedAnalyzer {
    delay: Duration,
    pending_tasks: Arc<Mutex<HashMap<Url, tokio::task::JoinHandle<()>>>>,
}

impl DebouncedAnalyzer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            pending_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn schedule_analysis<F, Fut>(&self, uri: Url, analysis_fn: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut tasks = self.pending_tasks.lock().await;
        
        // Cancel existing task for this URI
        if let Some(handle) = tasks.remove(&uri) {
            handle.abort();
        }

        let delay = self.delay;
        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            analysis_fn().await;
        });

        tasks.insert(uri, task);
    }
}

#[derive(Debug, Clone)]
pub struct CachedAnalysisResult {
    pub result: CodeAnalysisResponse,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct AIResponseCache {
    cache: Arc<Mutex<HashMap<String, CachedAnalysisResult>>>,
    ttl: Duration,
}

impl AIResponseCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CodeAnalysisResponse> {
        let mut cache = self.cache.lock().await;
        
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.result.clone());
            } else {
                cache.remove(key);
            }
        }
        
        None
    }

    pub async fn insert(&self, key: String, result: CodeAnalysisResponse) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, CachedAnalysisResult {
            result,
            timestamp: Instant::now(),
        });
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().await;
        cache.retain(|_, cached| cached.timestamp.elapsed() < self.ttl);
    }
}

struct Backend {
    client: Client,
    document_map: Arc<Mutex<HashMap<Url, String>>>,
    ai_service: Arc<CodeAnalysisService>,
    debounced_analyzer: DebouncedAnalyzer,
    response_cache: AIResponseCache,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "auto-coder.analyzeCode".to_string(),
                        "auto-coder.fixError".to_string(),
                        "auto-coder.explainCode".to_string(),
                        "auto-coder.generateCode".to_string(),
                    ],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "Auto-Coder LSP".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Auto-Coder LSP server initialized!")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        
        let mut document_map = self.document_map.lock().await;
        document_map.insert(uri.clone(), text.clone());
        drop(document_map);
        
        self.client
            .log_message(MessageType::INFO, "Document opened!")
            .await;
            
        // Analyze the document and provide diagnostics
        self.analyze_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut document_map = self.document_map.lock().await;
        
        if let Some(change) = params.content_changes.last() {
            document_map.insert(params.text_document.uri.clone(), change.text.clone());
            
            // Analyze the document and provide diagnostics
            self.analyze_document(&params.text_document.uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut document_map = self.document_map.lock().await;
        document_map.remove(&params.text_document.uri);
        
        // Clear diagnostics for closed document
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let document_map = self.document_map.lock().await;
        let uri = &params.text_document_position_params.text_document.uri;
        
        if let Some(document) = document_map.get(uri) {
            let position = params.text_document_position_params.position;
            
            // Extract the word and surrounding context
            if let Some(word) = Self::get_word_at_position(document, position) {
                let context = Self::get_context_around_position(document, position, 3);
                let language = Self::detect_language_from_uri(uri);
                
                // Try to get AI-powered hover information
                let hover_content = match self.get_ai_hover_info(&word, &context, &language).await {
                    Ok(ai_info) => ai_info,
                    Err(_) => {
                        // Fallback to basic hover info
                        format!("**{}**\n\nAuto-Coder can provide more information about this code. Use the command palette to analyze or explain this code.", word)
                    }
                };
                
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_content,
                    }),
                    range: None,
                }));
            }
        }
        
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let document_map = self.document_map.lock().await;
        let uri = &params.text_document_position.text_document.uri;
        
        if let Some(document) = document_map.get(uri) {
            let position = params.text_document_position.position;
            let language = Self::detect_language_from_uri(uri);
            
            // Get extended context for better AI completions
            let context = Self::get_context_around_position(document, position, 5);
            
            // Determine trigger character
            let trigger_char = params.context
                .as_ref()
                .and_then(|ctx| ctx.trigger_character.as_deref());
            
            // Get AI-powered completions
            let mut items = match self.get_ai_completions(&context, &language, trigger_char).await {
                Ok(ai_items) => ai_items,
                Err(_) => {
                    // Fallback to basic completions
                    self.get_fallback_completions(&language).await
                }
            };
            
            // Add some basic code snippets for common patterns
            let mut basic_items = self.get_fallback_completions(&language).await;
            items.append(&mut basic_items);
            
            // Limit the number of completions to prevent overwhelming the user
            items.truncate(20);
            
            return Ok(Some(CompletionResponse::Array(items)));
        }
        
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let document_map = self.document_map.lock().await;
        let uri = &params.text_document.uri;
        
        if let Some(document) = document_map.get(uri) {
            let mut actions = vec![];
            
            // Add code actions for each diagnostic
            for diagnostic in &params.context.diagnostics {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Fix: {}", diagnostic.message),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: None,
                    command: Some(Command {
                        title: "Fix Error".to_string(),
                        command: "auto-coder.fixError".to_string(),
                        arguments: Some(vec![
                            serde_json::to_value(uri.to_string()).unwrap_or_default(),
                            serde_json::to_value(&diagnostic.range).unwrap_or_default(),
                            serde_json::to_value(&diagnostic.message).unwrap_or_default(),
                        ]),
                    }),
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                }));
            }
            
            // Add general code actions
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Analyze Code".to_string(),
                kind: Some(CodeActionKind::SOURCE),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: "Analyze Code".to_string(),
                    command: "auto-coder.analyzeCode".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap_or_default(),
                    ]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
            
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Explain Code".to_string(),
                kind: Some(CodeActionKind::SOURCE),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: "Explain Code".to_string(),
                    command: "auto-coder.explainCode".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                    ]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
            
            return Ok(Some(actions));
        }
        
        Ok(None)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> LspResult<Option<serde_json::Value>> {
        match params.command.as_str() {
            "auto-coder.analyzeCode" => {
                let args = &params.arguments;
                if !args.is_empty() {
                    if let Some(uri_value) = args.get(0) {
                        if let Ok(uri_str) = serde_json::from_value::<String>(uri_value.clone()) {
                            let uri = Url::parse(&uri_str).unwrap();
                            let document_map = self.document_map.lock().await;
                            
                            if let Some(document) = document_map.get(&uri) {
                                // Analyze the code
                                self.client
                                    .show_message(MessageType::INFO, "Analyzing code...")
                                    .await;
                                    
                                // In a real implementation, this would call the AI model
                                // For now, we'll just return a placeholder message
                                return Ok(Some(serde_json::json!({
                                    "message": "Code analysis completed"
                                })));
                            }
                        }
                    }
                }
            }
            "auto-coder.fixError" => {
                // Implementation for fixing errors
                self.client
                    .show_message(MessageType::INFO, "Fixing error...")
                    .await;
            }
            "auto-coder.explainCode" => {
                // Implementation for explaining code
                self.client
                    .show_message(MessageType::INFO, "Explaining code...")
                    .await;
            }
            "auto-coder.generateCode" => {
                // Implementation for generating code
                self.client
                    .show_message(MessageType::INFO, "Generating code...")
                    .await;
            }
            _ => {
                self.client
                    .log_message(MessageType::ERROR, &format!("Unknown command: {}", params.command))
                    .await;
            }
        }
        
        Ok(None)
    }
}

impl Backend {
    async fn analyze_document(&self, uri: &Url, text: &str) -> () {
        let uri_clone = uri.clone();
        let text_clone = text.to_string();
        let client = self.client.clone();
        let ai_service = self.ai_service.clone();
        let response_cache = self.response_cache.clone();
        
        // Use debounced analysis to prevent excessive AI calls
        self.debounced_analyzer.schedule_analysis(uri.clone(), move || {
            let uri = uri_clone;
            let text = text_clone;
            let client = client;
            let ai_service = ai_service;
            let response_cache = response_cache;
            
            async move {
                // Generate cache key based on content hash
                let cache_key = format!("{}_{}", uri.to_string(), 
                    format!("{:x}", md5::compute(text.as_bytes())));
                
                // Check cache first
                if let Some(cached_result) = response_cache.get(&cache_key).await {
                    Self::publish_ai_diagnostics(&client, &uri, &cached_result).await;
                    return;
                }
                
                // Detect language from URI
                let language = Self::detect_language_from_uri(&uri);
                
                // Create analysis request
                let analysis_request = crate::code_analysis::CodeAnalysisRequest {
                    code: text.clone(),
                    language: language.clone(),
                    file_path: Some(uri.to_string()),
                };
                
                // Perform AI analysis
                match ai_service.analyze_code(&analysis_request).await {
                    Ok(analysis_result) => {
                        // Cache the result
                        response_cache.insert(cache_key, analysis_result.clone()).await;
                        
                        // Publish AI-powered diagnostics
                        Self::publish_ai_diagnostics(&client, &uri, &analysis_result).await;
                    }
                    Err(e) => {
                        // Fall back to basic analysis on AI failure
                        client.log_message(
                            tower_lsp::lsp_types::MessageType::WARNING,
                            &format!("AI analysis failed for {}: {}. Using fallback analysis.", uri, e)
                        ).await;
                        
                        Self::publish_fallback_diagnostics(&client, &uri, &text).await;
                    }
                }
            }
        }).await;
    }
    
    async fn publish_ai_diagnostics(
        client: &Client,
        uri: &Url,
        analysis_result: &crate::code_analysis::CodeAnalysisResponse,
    ) {
        let mut diagnostics = vec![];
        
        // Convert AI suggestions to LSP diagnostics
        for suggestion in &analysis_result.suggestions {
            diagnostics.push(Diagnostic {
                range: tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: suggestion.range.start.line,
                        character: suggestion.range.start.character,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: suggestion.range.end.line,
                        character: suggestion.range.end.character,
                    },
                },
                severity: Some(suggestion.severity),
                code: None,
                code_description: None,
                source: Some("Auto-Coder AI".to_string()),
                message: suggestion.description.clone(),
                related_information: None,
                tags: None,
                data: None,
            });
        }
        
        // Convert AI errors to LSP diagnostics
        for error in &analysis_result.errors {
            diagnostics.push(Diagnostic {
                range: tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: error.range.start.line,
                        character: error.range.start.character,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: error.range.end.line,
                        character: error.range.end.character,
                    },
                },
                severity: Some(error.severity),
                code: None,
                code_description: None,
                source: Some("Auto-Coder AI".to_string()),
                message: error.message.clone(),
                related_information: None,
                tags: None,
                data: None,
            });
        }
        
        // Publish diagnostics
        client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }
    
    async fn publish_fallback_diagnostics(client: &Client, uri: &Url, text: &str) {
        let mut diagnostics = vec![];
        
        // Basic fallback analysis: flag TODO comments and simple issues
        for (i, line) in text.lines().enumerate() {
            if line.contains("TODO") {
                diagnostics.push(Diagnostic {
                    range: tower_lsp::lsp_types::Range {
                        start: tower_lsp::lsp_types::Position {
                            line: i as u32,
                            character: line.find("TODO").unwrap_or(0) as u32,
                        },
                        end: tower_lsp::lsp_types::Position {
                            line: i as u32,
                            character: (line.find("TODO").unwrap_or(0) + 4) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: None,
                    code_description: None,
                    source: Some("Auto-Coder Fallback".to_string()),
                    message: "TODO comment found".to_string(),
                    related_information: None,
                    tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                    data: None,
                });
            }
        }
        
        client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }
    
    fn detect_language_from_uri(uri: &Url) -> String {
        let path = uri.path();
        if let Some(extension) = std::path::Path::new(path).extension() {
            match extension.to_string_lossy().as_ref() {
                "js" => "javascript".to_string(),
                "ts" => "typescript".to_string(),
                "jsx" => "jsx".to_string(),
                "tsx" => "tsx".to_string(),
                "py" => "python".to_string(),
                "rs" => "rust".to_string(),
                "go" => "go".to_string(),
                "java" => "java".to_string(),
                "c" => "c".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "cs" => "csharp".to_string(),
                "php" => "php".to_string(),
                "rb" => "ruby".to_string(),
                "swift" => "swift".to_string(),
                "kt" | "kts" => "kotlin".to_string(),
                "scala" => "scala".to_string(),
                "html" => "html".to_string(),
                "css" => "css".to_string(),
                "json" => "json".to_string(),
                "md" => "markdown".to_string(),
                "yml" | "yaml" => "yaml".to_string(),
                "xml" => "xml".to_string(),
                "sh" | "bash" => "bash".to_string(),
                "sql" => "sql".to_string(),
                _ => "plaintext".to_string(),
            }
        } else {
            "plaintext".to_string()
        }
    }
    
    fn get_word_at_position(document: &str, position: tower_lsp::lsp_types::Position) -> Option<String> {
        let lines: Vec<&str> = document.lines().collect();
        
        if position.line as usize >= lines.len() {
            return None;
        }
        
        let line = lines[position.line as usize];
        
        if position.character as usize > line.len() {
            return None;
        }
        
        // Find word boundaries
        let mut start = position.character as usize;
        let mut end = position.character as usize;
        
        // Move start to the beginning of the word
        while start > 0 && line.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric() || c == '_') {
            start -= 1;
        }
        
        // Move end to the end of the word
        while end < line.len() && line.chars().nth(end).map_or(false, |c| c.is_alphanumeric() || c == '_') {
            end += 1;
        }
        
        if start == end {
            return None;
        }
        
        Some(line[start..end].to_string())
    }
    
    fn get_context_before_position(document: &str, position: tower_lsp::lsp_types::Position) -> String {
        let lines: Vec<&str> = document.lines().collect();
        
        if position.line as usize >= lines.len() {
            return String::new();
        }
        
        let line = lines[position.line as usize];
        
        if position.character as usize > line.len() {
            return line.to_string();
        }
        
        line[..position.character as usize].to_string()
    }
    
    fn get_context_around_position(document: &str, position: tower_lsp::lsp_types::Position, context_lines: u32) -> String {
        let lines: Vec<&str> = document.lines().collect();
        let line_num = position.line as usize;
        
        let start = line_num.saturating_sub(context_lines as usize);
        let end = std::cmp::min(line_num + context_lines as usize + 1, lines.len());
        
        lines[start..end].join("\n")
    }
    
    async fn get_ai_hover_info(&self, word: &str, context: &str, language: &str) -> std::result::Result<String, String> {
        // Create a specialized prompt for hover information
        let prompt = format!(
            "Explain the {} identifier '{}' in the following context. Provide a concise explanation suitable for an IDE hover tooltip:\n\n```{}\n{}\n```\n\nFocus on what '{}' is, its purpose, and relevant details.",
            language, word, language, context, word
        );
        
        let request = crate::code_analysis::CodeGenerationRequest {
            prompt,
            language: language.to_string(),
            context: Some(context.to_string()),
        };
        
        match self.ai_service.generate_code(&request).await {
            Ok(response) => Ok(format!("**{}**\n\n{}", word, response.explanation)),
            Err(e) => Err(e),
        }
    }
    
    async fn get_ai_completions(&self, context: &str, language: &str, trigger_char: Option<&str>) -> Result<Vec<CompletionItem>, String> {
        let prompt = match trigger_char {
            Some(".") => format!(
                "Given this {} code context, suggest appropriate method/property completions after the dot:\n\n```{}\n{}\n```\n\nReturn a list of completions with descriptions.",
                language, language, context
            ),
            Some(":") => format!(
                "Given this {} code context, suggest appropriate type or namespace completions after the colon:\n\n```{}\n{}\n```\n\nReturn a list of completions with descriptions.",
                language, language, context
            ),
            _ => format!(
                "Given this {} code context, suggest appropriate code completions:\n\n```{}\n{}\n```\n\nReturn a list of relevant completions with descriptions.",
                language, language, context
            ),
        };
        
        let request = crate::code_analysis::CodeGenerationRequest {
            prompt,
            language: language.to_string(),
            context: Some(context.to_string()),
        };
        
        match self.ai_service.generate_code(&request).await {
            Ok(response) => {
                // Parse AI response into completion items
                // This is a simplified implementation - in practice, you'd want more sophisticated parsing
                let mut items = vec![];
                
                // Extract suggestions from AI response (simplified)
                for (i, line) in response.generated_code.lines().take(10).enumerate() {
                    if !line.trim().is_empty() {
                        items.push(CompletionItem {
                            label: line.trim().to_string(),
                            kind: Some(CompletionItemKind::TEXT),
                            detail: Some("AI Suggestion".to_string()),
                            documentation: Some(Documentation::String(format!("AI-generated suggestion: {}", line.trim()))),
                            insert_text: Some(line.trim().to_string()),
                            sort_text: Some(format!("{:02}", i)),
                            ..CompletionItem::default()
                        });
                    }
                }
                
                Ok(items)
            }
            Err(e) => Err(e),
        }
    }
    
    async fn get_fallback_completions(&self, language: &str) -> Vec<CompletionItem> {
        match language {
            "javascript" | "typescript" | "jsx" | "tsx" => vec![
                CompletionItem {
                    label: "function".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new function".to_string()),
                    insert_text: Some("function ${1:name}(${2:params}) {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "class".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new class".to_string()),
                    insert_text: Some("class ${1:Name} {\n\tconstructor(${2:params}) {\n\t\t${0}\n\t}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "if".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create an if statement".to_string()),
                    insert_text: Some("if (${1:condition}) {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
            ],
            "rust" => vec![
                CompletionItem {
                    label: "fn".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new function".to_string()),
                    insert_text: Some("fn ${1:name}(${2:params}) ${3:-> ReturnType} {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "struct".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new struct".to_string()),
                    insert_text: Some("struct ${1:Name} {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "impl".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create an impl block".to_string()),
                    insert_text: Some("impl ${1:Name} {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
            ],
            "python" => vec![
                CompletionItem {
                    label: "def".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new function".to_string()),
                    insert_text: Some("def ${1:name}(${2:params}):\n\t${0}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "class".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create a new class".to_string()),
                    insert_text: Some("class ${1:Name}:\n\tdef __init__(self${2:, params}):\n\t\t${0}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
                CompletionItem {
                    label: "if".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create an if statement".to_string()),
                    insert_text: Some("if ${1:condition}:\n\t${0}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
            ],
            _ => vec![
                CompletionItem {
                    label: "if".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some("Create an if statement".to_string()),
                    insert_text: Some("if ${1:condition} {\n\t${0}\n}".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
            ],
        }
    }
}

pub async fn start_lsp_server_with_ai(ai_service: Arc<CodeAnalysisService>) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_map: Arc::new(Mutex::new(HashMap::new())),
        ai_service: ai_service.clone(),
        debounced_analyzer: DebouncedAnalyzer::new(500), // 500ms delay
        response_cache: AIResponseCache::new(300), // 5 minute TTL
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}

pub async fn start_lsp_server() {
    // Legacy function for backward compatibility - uses a dummy AI service
    use crate::ollama_client::OllamaClient;
    use tokio::sync::Mutex;
    
    let dummy_ollama = Arc::new(Mutex::new(OllamaClient::new(None)));
    let dummy_ai_service = Arc::new(CodeAnalysisService::new(dummy_ollama));
    
    start_lsp_server_with_ai(dummy_ai_service).await;
}

// Tauri Commands for LSP functionality

#[tauri::command]
pub async fn initialize_lsp_server(
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<String, String> {
    // In a real implementation, you would start the LSP server in a background task
    // For now, we'll just return a success message
    Ok("LSP server initialized successfully".to_string())
}

#[tauri::command]
pub async fn shutdown_lsp_server() -> std::result::Result<String, String> {
    // Placeholder for LSP server shutdown
    Ok("LSP server shutdown successfully".to_string())
}

#[tauri::command]
pub async fn lsp_open_document(
    uri: String,
    content: String,
    language: String,
) -> std::result::Result<String, String> {
    // Placeholder for document open handling
    Ok(format!("Document {} opened successfully", uri))
}

#[tauri::command]
pub async fn lsp_close_document(uri: String) -> std::result::Result<String, String> {
    // Placeholder for document close handling
    Ok(format!("Document {} closed successfully", uri))
}

#[tauri::command]
pub async fn lsp_update_document(
    uri: String,
    content: String,
) -> std::result::Result<String, String> {
    // Placeholder for document update handling
    Ok(format!("Document {} updated successfully", uri))
}

#[tauri::command]
pub async fn lsp_get_diagnostics(
    uri: String,
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<Vec<String>, String> {
    // Placeholder for getting diagnostics
    Ok(vec!["Example diagnostic".to_string()])
}

#[tauri::command]
pub async fn lsp_get_completions(
    uri: String,
    position_line: u32,
    position_character: u32,
    context: Option<String>,
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<Vec<String>, String> {
    // Placeholder for getting completions
    Ok(vec!["example_completion".to_string(), "another_completion".to_string()])
}

#[tauri::command]
pub async fn lsp_get_hover(
    uri: String,
    position_line: u32,
    position_character: u32,
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<Option<String>, String> {
    // Placeholder for getting hover information
    Ok(Some("Hover information for the symbol".to_string()))
}

#[tauri::command]
pub async fn lsp_get_code_actions(
    uri: String,
    range_start_line: u32,
    range_start_character: u32,
    range_end_line: u32,
    range_end_character: u32,
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<Vec<String>, String> {
    // Placeholder for getting code actions
    Ok(vec!["Fix with AI".to_string(), "Analyze code".to_string()])
}

#[tauri::command]
pub async fn lsp_execute_command(
    command: String,
    args: Option<Vec<String>>,
    ai_service: State<'_, Arc<CodeAnalysisService>>,
) -> std::result::Result<String, String> {
    // Placeholder for executing commands
    Ok(format!("Command {} executed successfully", command))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_word_at_position() {
        let document = "function test() {\n    console.log('hello');\n}";

        // Test getting "function"
        let word = Backend::get_word_at_position(document, tower_lsp::lsp_types::Position {
            line: 0,
            character: 3,
        });
        assert_eq!(word, Some("function".to_string()));

        // Test getting "test"
        let word = Backend::get_word_at_position(document, tower_lsp::lsp_types::Position {
            line: 0,
            character: 12,
        });
        assert_eq!(word, Some("test".to_string()));

        // Test getting "console"
        let word = Backend::get_word_at_position(document, tower_lsp::lsp_types::Position {
            line: 1,
            character: 6,
        });
        assert_eq!(word, Some("console".to_string()));
    }
}

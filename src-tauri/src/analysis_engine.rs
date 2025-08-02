use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use crate::ollama_client::{OllamaClient, GenerateOptions};
use crate::chroma_manager::ChromaManager;
use uuid::Uuid;

/// Analysis mode types for Deep Analysis Mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisMode {
    Standard,
    Socratic,
    Systematic,
}

/// Question-Answer chain for tracking reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswerChain {
    pub question: String,
    pub answer: String,
    pub round: usize,
    pub timestamp: String,
    pub confidence: f32,
}

/// Deep analysis result containing solution and reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAnalysisResult {
    pub solution: String,
    pub reasoning: Vec<QuestionAnswerChain>,
    pub confidence: f32,
    pub saved_to_rag: bool,
    pub mode_used: AnalysisMode,
}

/// Configuration for deep analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub mode: AnalysisMode,
    pub max_rounds: usize,
    pub time_limit: Duration,
    pub save_to_rag: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            mode: AnalysisMode::Standard,
            max_rounds: 5,
            time_limit: Duration::from_secs(300), // 5 minutes
            save_to_rag: true,
        }
    }
}

/// Deep Analysis Engine for Socratic questioning and systematic problem-solving
pub struct AnalysisEngine {
    ollama_client: OllamaClient,
    chroma_manager: Option<ChromaManager>,
}

impl AnalysisEngine {
    pub fn new(ollama_client: OllamaClient, chroma_manager: Option<ChromaManager>) -> Self {
        Self {
            ollama_client,
            chroma_manager,
        }
    }

    /// Main entry point for deep analysis
    pub async fn analyze(
        &mut self,
        prompt: &str,
        model: &str,
        config: AnalysisConfig,
    ) -> Result<DeepAnalysisResult, String> {
        match config.mode {
            AnalysisMode::Standard => self.standard_analysis(prompt, model).await,
            AnalysisMode::Socratic => self.socratic_analysis(prompt, model, &config).await,
            AnalysisMode::Systematic => self.systematic_analysis(prompt, model, &config).await,
        }
    }

    /// Standard analysis mode - direct answer
    async fn standard_analysis(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<DeepAnalysisResult, String> {
        let options = GenerateOptions {
            temperature: Some(0.3),
            max_tokens: None,
            top_p: None,
            top_k: None,
        };

        let response = self
            .ollama_client
            .generate_completion(model, prompt, Some(options))
            .await
            .map_err(|e| e.to_string())?;

        Ok(DeepAnalysisResult {
            solution: response,
            reasoning: vec![],
            confidence: 0.8, // Standard mode has good confidence
            saved_to_rag: false,
            mode_used: AnalysisMode::Standard,
        })
    }

    /// Socratic analysis mode - guided questioning
    async fn socratic_analysis(
        &mut self,
        prompt: &str,
        model: &str,
        config: &AnalysisConfig,
    ) -> Result<DeepAnalysisResult, String> {
        let analysis_future = self.run_socratic_questioning(prompt, model, config);
        
        match timeout(config.time_limit, analysis_future).await {
            Ok(result) => result,
            Err(_) => Err("Analysis timed out. Try reducing max rounds or increasing time limit.".to_string()),
        }
    }

    /// Run the Socratic questioning process
    async fn run_socratic_questioning(
        &mut self,
        original_prompt: &str,
        model: &str,
        config: &AnalysisConfig,
    ) -> Result<DeepAnalysisResult, String> {
        let mut reasoning_chain = Vec::new();
        let mut current_context = original_prompt.to_string();

        // Query similar patterns for enhanced analysis
        let similar_patterns = self.query_similar_patterns(original_prompt, 3).await;
        if !similar_patterns.is_empty() {
            current_context = format!(
                "{}\n\nSimilar Successful Patterns:\n{}",
                current_context,
                similar_patterns.join("\n---\n")
            );
        }

        // Define the 4-stage Socratic questioning framework
        let questioning_stages = [
            "What assumptions are we making about this problem? What evidence supports these assumptions?",
            "What alternative approaches or perspectives could we consider? What are we not seeing?",
            "What are the implications and consequences of different solutions? What could go wrong?",
            "How can we validate our understanding? What would convince us this is the right solution?",
        ];

        for (round, stage_question) in questioning_stages.iter().enumerate() {
            if round >= config.max_rounds {
                break;
            }

            // Generate a contextual question based on the stage and current understanding
            let question_prompt = format!(
                "Given this problem context: {}\n\nAnd considering this stage of analysis: {}\n\nAsk one specific, insightful question that will deepen understanding. Be concise and focused:",
                current_context, stage_question
            );

            let question = self.ask_focused_question(&question_prompt, model).await?;
            
            // Get the answer to the question
            let answer_prompt = format!(
                "Context: {}\n\nQuestion: {}\n\nProvide a thoughtful, detailed answer:",
                current_context, question
            );

            let answer = self.get_detailed_answer(&answer_prompt, model).await?;
            
            // Calculate confidence based on answer quality and stage
            let confidence = self.calculate_confidence(&answer, round);

            reasoning_chain.push(QuestionAnswerChain {
                question: question.clone(),
                answer: answer.clone(),
                round: round + 1,
                timestamp: chrono::Utc::now().to_rfc3339(),
                confidence,
            });

            // Update context with new insights
            current_context = format!(
                "{}\n\nInsight from Round {}: Q: {} A: {}",
                current_context, round + 1, question, answer
            );
        }

        // Generate final solution based on all reasoning
        let final_solution = self.synthesize_solution(&current_context, model).await?;
        
        // Calculate overall confidence
        let overall_confidence = reasoning_chain
            .iter()
            .map(|qa| qa.confidence)
            .fold(0.0, |acc, conf| acc + conf) / reasoning_chain.len() as f32;

        // Save to RAG if configured
        let saved_to_rag = if config.save_to_rag {
            self.save_reasoning_to_rag(original_prompt, &reasoning_chain, &final_solution).await
        } else {
            false
        };

        Ok(DeepAnalysisResult {
            solution: final_solution,
            reasoning: reasoning_chain,
            confidence: overall_confidence,
            saved_to_rag,
            mode_used: AnalysisMode::Socratic,
        })
    }

    /// Systematic analysis mode - PDCA/OODA loop approach  
    async fn systematic_analysis(
        &mut self,
        prompt: &str,
        model: &str,
        config: &AnalysisConfig,
    ) -> Result<DeepAnalysisResult, String> {
        let mut reasoning_chain = Vec::new();

        // Query similar patterns for enhanced systematic analysis
        let similar_patterns = self.query_similar_patterns(prompt, 2).await;
        let mut context = if !similar_patterns.is_empty() {
            format!(
                "{}\n\nLearning from Similar Cases:\n{}",
                prompt,
                similar_patterns.join("\n---\n")
            )
        } else {
            prompt.to_string()
        };

        // PDCA (Plan-Do-Check-Act) cycle adapted for problem-solving
        let systematic_stages = [
            ("Plan", "What is the core problem? What are our objectives and constraints?"),
            ("Do", "What is our proposed solution approach? What are the key steps?"),
            ("Check", "What are the potential issues with this approach? How do we validate it?"),
            ("Act", "How do we refine and implement this solution? What are the next steps?"),
        ];

        for (round, (stage_name, stage_question)) in systematic_stages.iter().enumerate() {
            if round >= config.max_rounds {
                break;
            }

            let systematic_prompt = format!(
                "Problem Context: {}\n\n{} Stage: {}\n\nProvide a structured analysis for this stage:",
                context, stage_name, stage_question
            );

            let analysis = self.get_detailed_answer(&systematic_prompt, model).await?;
            let confidence = self.calculate_confidence(&analysis, round);

            reasoning_chain.push(QuestionAnswerChain {
                question: format!("{} Stage: {}", stage_name, stage_question),
                answer: analysis.clone(),
                round: round + 1,
                timestamp: chrono::Utc::now().to_rfc3339(),
                confidence,
            });

            context = format!("{}\n\n{} Analysis: {}", context, stage_name, analysis);
        }

        let final_solution = self.synthesize_solution(&context, model).await?;
        
        let overall_confidence = reasoning_chain
            .iter()
            .map(|qa| qa.confidence)
            .fold(0.0, |acc, conf| acc + conf) / reasoning_chain.len() as f32;

        let saved_to_rag = if config.save_to_rag {
            self.save_reasoning_to_rag(prompt, &reasoning_chain, &final_solution).await
        } else {
            false
        };

        Ok(DeepAnalysisResult {
            solution: final_solution,
            reasoning: reasoning_chain,
            confidence: overall_confidence,
            saved_to_rag,
            mode_used: AnalysisMode::Systematic,
        })
    }

    /// Ask a focused question for Socratic method
    async fn ask_focused_question(&self, prompt: &str, model: &str) -> Result<String, String> {
        let options = GenerateOptions {
            temperature: Some(0.7), // Higher temperature for creative questioning
            max_tokens: Some(100),  // Keep questions concise
            top_p: None,
            top_k: None,
        };

        self.ollama_client
            .generate_completion(model, prompt, Some(options))
            .await
            .map_err(|e| e.to_string())
    }

    /// Get a detailed answer to a specific question
    async fn get_detailed_answer(&self, prompt: &str, model: &str) -> Result<String, String> {
        let options = GenerateOptions {
            temperature: Some(0.3), // Lower temperature for focused answers
            max_tokens: Some(500),  // Allow detailed responses
            top_p: None,
            top_k: None,
        };

        self.ollama_client
            .generate_completion(model, prompt, Some(options))
            .await
            .map_err(|e| e.to_string())
    }

    /// Synthesize final solution from all reasoning
    async fn synthesize_solution(&self, context: &str, model: &str) -> Result<String, String> {
        let synthesis_prompt = format!(
            "Based on all the analysis and reasoning below, provide a clear, actionable solution:\n\n{}\n\nFinal Solution:",
            context
        );

        let options = GenerateOptions {
            temperature: Some(0.2), // Low temperature for precise synthesis
            max_tokens: Some(800),  // Allow comprehensive solution
            top_p: None,
            top_k: None,
        };

        self.ollama_client
            .generate_completion(model, &synthesis_prompt, Some(options))
            .await
            .map_err(|e| e.to_string())
    }

    /// Calculate confidence based on answer quality and stage
    fn calculate_confidence(&self, answer: &str, round: usize) -> f32 {
        let base_confidence = match round {
            0 => 0.6, // First round - identifying the problem
            1 => 0.7, // Second round - exploring solutions
            2 => 0.8, // Third round - validating approach
            3 => 0.9, // Fourth round - final validation
            _ => 0.85, // Additional rounds
        };

        // Adjust confidence based on answer length and completeness
        let length_factor = (answer.len() as f32 / 200.0).min(1.2).max(0.5);
        
        // Check for key indicators of thoughtful analysis
        let quality_indicators = [
            "because", "however", "therefore", "consider", "alternative",
            "implication", "consequence", "assumption", "evidence"
        ];
        
        let quality_score = quality_indicators
            .iter()
            .map(|&indicator| if answer.to_lowercase().contains(indicator) { 0.05 } else { 0.0 })
            .sum::<f32>();

        (base_confidence * length_factor + quality_score).min(0.95).max(0.3)
    }

    /// Save reasoning patterns to RAG for future learning
    async fn save_reasoning_to_rag(
        &mut self,
        original_prompt: &str,
        reasoning_chain: &[QuestionAnswerChain],
        solution: &str,
    ) -> bool {
        // Get problem type before mutable borrow
        let problem_type = self.classify_problem_type(original_prompt);
        
        if let Some(manager) = &mut self.chroma_manager {
            // Create a comprehensive document that captures the reasoning pattern
            let reasoning_document = format!(
                "Deep Analysis Pattern\n\nOriginal Problem: {}\n\nReasoning Process:\n{}\n\nFinal Solution: {}\n\nPattern Summary: This is a successful {} analysis with {} reasoning rounds and average confidence of {:.2}.",
                original_prompt,
                reasoning_chain
                    .iter()
                    .enumerate()
                    .map(|(i, qa)| format!(
                        "Round {}: {}\nAnswer: {} (Confidence: {:.2})",
                        i + 1, qa.question, qa.answer, qa.confidence
                    ))
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                solution,
                if reasoning_chain.len() > 0 {
                    match reasoning_chain[0].question.to_lowercase().contains("assumption") {
                        true => "Socratic",
                        false => "Systematic",
                    }
                } else {
                    "Unknown"
                },
                reasoning_chain.len(),
                reasoning_chain.iter().map(|qa| qa.confidence).sum::<f32>() / reasoning_chain.len() as f32
            );

            // Generate unique ID for this reasoning pattern
            let pattern_id = format!("deep_analysis_{}", uuid::Uuid::new_v4());

            // Create metadata for pattern matching
            let metadata = crate::chroma_manager::DocumentMetadata {
                source: "deep_analysis_engine".to_string(),
                document_type: "reasoning_pattern".to_string(),
                language: Some("analysis".to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
                file_path: None,
                url: None,
                title: None,
                additional: HashMap::from([
                    ("problem_type".to_string(), serde_json::Value::String(problem_type.clone())),
                    ("analysis_mode".to_string(), serde_json::Value::String(if reasoning_chain.len() > 0 {
                        match reasoning_chain[0].question.to_lowercase().contains("assumption") {
                            true => "socratic".to_string(),
                            false => "systematic".to_string(),
                        }
                    } else {
                        "unknown".to_string()
                    })),
                    ("rounds".to_string(), serde_json::Value::String(reasoning_chain.len().to_string())),
                    ("avg_confidence".to_string(), serde_json::Value::String(format!("{:.2}", 
                        reasoning_chain.iter().map(|qa| qa.confidence).sum::<f32>() / reasoning_chain.len() as f32)
                    )),
                    ("solution_length".to_string(), serde_json::Value::String(solution.len().to_string())),
                ]),
            };

            // Try to add to ChromaDB reasoning patterns collection
            match manager.add_documents(
                "reasoning_patterns",
                vec![reasoning_document],
                vec![metadata],
                Some(vec![pattern_id]),
            ) {
                Ok(_) => {
                    println!("Successfully saved reasoning pattern to RAG");
                    true
                }
                Err(e) => {
                    eprintln!("Failed to save reasoning pattern to RAG: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Classify the type of problem for better pattern matching
    fn classify_problem_type(&self, prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("debug") || prompt_lower.contains("error") || prompt_lower.contains("fix") {
            "debugging".to_string()
        } else if prompt_lower.contains("design") || prompt_lower.contains("architecture") {
            "design".to_string()
        } else if prompt_lower.contains("optimize") || prompt_lower.contains("performance") {
            "optimization".to_string()
        } else if prompt_lower.contains("refactor") || prompt_lower.contains("improve") {
            "refactoring".to_string()
        } else if prompt_lower.contains("implement") || prompt_lower.contains("create") {
            "implementation".to_string()
        } else if prompt_lower.contains("explain") || prompt_lower.contains("understand") {
            "explanation".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Query similar reasoning patterns from RAG for enhanced analysis
    pub async fn query_similar_patterns(&mut self, prompt: &str, limit: usize) -> Vec<String> {
        // Get problem type before mutable borrow
        let problem_type = self.classify_problem_type(prompt);
        
        if let Some(manager) = &mut self.chroma_manager {
            
            // Create a query that looks for similar problem patterns
            let query = format!(
                "Similar problem to analyze: {} Find reasoning patterns for {} problems",
                prompt,
                problem_type
            );

            match manager.query("reasoning_patterns", &query, limit, None) {
                Ok(results) => {
                    results.into_iter()
                        .map(|result| result.document)
                        .collect()
                }
                Err(e) => {
                    eprintln!("Failed to query similar patterns: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// Get statistics about saved reasoning patterns
    pub async fn get_pattern_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        if let Some(manager) = &self.chroma_manager {
            // This would require ChromaDB to support collection statistics
            // For now, return empty stats
            stats.insert("total_patterns".to_string(), 0);
            stats.insert("socratic_patterns".to_string(), 0);
            stats.insert("systematic_patterns".to_string(), 0);
        }
        
        stats
    }
}

/// Helper function to determine if a prompt suggests complex analysis is needed
pub fn should_suggest_deep_analysis(prompt: &str) -> bool {
    let complexity_indicators = [
        "why", "how", "debug", "troubleshoot", "issue", "problem", "error",
        "not working", "failing", "broken", "complex", "architecture",
        "design", "refactor", "optimize", "performance"
    ];

    let prompt_lower = prompt.to_lowercase();
    let indicator_count = complexity_indicators
        .iter()
        .filter(|&indicator| prompt_lower.contains(indicator))
        .count();

    // Suggest deep analysis if multiple complexity indicators or prompt is long
    indicator_count >= 2 || prompt.len() > 200
}
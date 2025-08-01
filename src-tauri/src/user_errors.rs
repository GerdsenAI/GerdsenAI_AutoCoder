use serde::{Deserialize, Serialize};
use std::fmt;

/// User-friendly error types with actionable guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserError {
    pub title: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub help_link: Option<String>,
    pub error_code: String,
    pub technical_details: Option<String>,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UserError {}

/// Convert common error scenarios to user-friendly messages
pub trait ToUserError {
    fn to_user_error(&self) -> UserError;
}

impl ToUserError for Box<dyn std::error::Error + Send> {
    fn to_user_error(&self) -> UserError {
        let error_str = self.to_string().to_lowercase();
        
        // Connection-related errors
        if error_str.contains("connection refused") || error_str.contains("could not connect") {
            if error_str.contains("11434") || error_str.contains("ollama") {
                return UserError {
                    title: "Ollama Not Running".to_string(),
                    message: "Cannot connect to Ollama. The AI service appears to be offline.".to_string(),
                    suggestion: Some("Please start Ollama by running 'ollama serve' in your terminal, or restart the Ollama application.".to_string()),
                    help_link: Some("https://ollama.ai/download".to_string()),
                    error_code: "OLLAMA_OFFLINE".to_string(),
                    technical_details: Some(self.to_string()),
                };
            } else if error_str.contains("8080") || error_str.contains("searxng") {
                return UserError {
                    title: "Search Service Unavailable".to_string(),
                    message: "Cannot connect to SearXNG web search service.".to_string(),
                    suggestion: Some("Search functionality will be limited. You can continue using other features normally.".to_string()),
                    help_link: Some("https://docs.searxng.org/admin/installation.html".to_string()),
                    error_code: "SEARXNG_OFFLINE".to_string(),
                    technical_details: Some(self.to_string()),
                };
            } else if error_str.contains("8000") || error_str.contains("chroma") {
                return UserError {
                    title: "Document Storage Unavailable".to_string(),
                    message: "Cannot connect to ChromaDB document storage service.".to_string(),
                    suggestion: Some("Document management features will be unavailable. Please check if ChromaDB is running.".to_string()),
                    help_link: Some("https://docs.trychroma.com/getting-started".to_string()),
                    error_code: "CHROMADB_OFFLINE".to_string(),
                    technical_details: Some(self.to_string()),
                };
            } else {
                return UserError {
                    title: "Connection Failed".to_string(),
                    message: "Unable to connect to a required service.".to_string(),
                    suggestion: Some("Please check your network connection and ensure all services are running.".to_string()),
                    help_link: None,
                    error_code: "CONNECTION_FAILED".to_string(),
                    technical_details: Some(self.to_string()),
                };
            }
        }
        
        // Timeout errors
        if error_str.contains("timeout") || error_str.contains("timed out") {
            return UserError {
                title: "Operation Timed Out".to_string(),
                message: "The operation took too long to complete.".to_string(),
                suggestion: Some("This might be due to network issues or high server load. Please try again in a moment.".to_string()),
                help_link: None,
                error_code: "TIMEOUT".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // Model-related errors
        if error_str.contains("model") && (error_str.contains("not found") || error_str.contains("does not exist")) {
            return UserError {
                title: "AI Model Not Available".to_string(),
                message: "The requested AI model is not installed or available.".to_string(),
                suggestion: Some("Please install the model using 'ollama pull <model-name>' or select a different model from the dropdown.".to_string()),
                help_link: Some("https://ollama.ai/library".to_string()),
                error_code: "MODEL_NOT_FOUND".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // Authentication/Permission errors
        if error_str.contains("unauthorized") || error_str.contains("forbidden") || error_str.contains("permission denied") {
            return UserError {
                title: "Access Denied".to_string(),
                message: "You don't have permission to perform this action.".to_string(),
                suggestion: Some("Please check your credentials or contact your administrator.".to_string()),
                help_link: None,
                error_code: "ACCESS_DENIED".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // File system errors
        if error_str.contains("no such file") || error_str.contains("file not found") {
            return UserError {
                title: "File Not Found".to_string(),
                message: "The requested file or directory could not be found.".to_string(),
                suggestion: Some("Please check the file path and ensure the file exists.".to_string()),
                help_link: None,
                error_code: "FILE_NOT_FOUND".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        if error_str.contains("permission denied") && (error_str.contains("write") || error_str.contains("create")) {
            return UserError {
                title: "Cannot Write File".to_string(),
                message: "Unable to save changes due to file permission issues.".to_string(),
                suggestion: Some("Please check file permissions or try running as administrator.".to_string()),
                help_link: None,
                error_code: "WRITE_PERMISSION".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // Network errors
        if error_str.contains("dns") || error_str.contains("name resolution") {
            return UserError {
                title: "Network Error".to_string(),
                message: "Unable to resolve the server address.".to_string(),
                suggestion: Some("Please check your internet connection and DNS settings.".to_string()),
                help_link: None,
                error_code: "DNS_ERROR".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // Memory/Resource errors
        if error_str.contains("out of memory") || error_str.contains("memory allocation") {
            return UserError {
                title: "Memory Error".to_string(),
                message: "The system has run out of available memory.".to_string(),
                suggestion: Some("Please close other applications or try working with smaller files.".to_string(),
                help_link: None,
                error_code: "OUT_OF_MEMORY".to_string(),
                technical_details: Some(self.to_string()),
            };
        }
        
        // Generic fallback
        UserError {
            title: "Unexpected Error".to_string(),
            message: "An unexpected error occurred.".to_string(),
            suggestion: Some("Please try again. If the problem persists, you can report this issue for assistance.".to_string()),
            help_link: None,
            error_code: "GENERIC_ERROR".to_string(),
            technical_details: Some(self.to_string()),
        }
    }
}

impl ToUserError for String {
    fn to_user_error(&self) -> UserError {
        let error_str = self.to_lowercase();
        
        // Use the same logic as Box<dyn Error + Send>
        if error_str.contains("connection refused") || error_str.contains("could not connect") {
            if error_str.contains("11434") || error_str.contains("ollama") {
                return UserError {
                    title: "Ollama Not Running".to_string(),
                    message: "Cannot connect to Ollama. The AI service appears to be offline.".to_string(),
                    suggestion: Some("Please start Ollama by running 'ollama serve' in your terminal, or restart the Ollama application.".to_string()),
                    help_link: Some("https://ollama.ai/download".to_string()),
                    error_code: "OLLAMA_OFFLINE".to_string(),
                    technical_details: Some(self.clone()),
                };
            }
        }
        
        // Add other patterns as needed...
        UserError {
            title: "Error".to_string(),
            message: self.clone(),
            suggestion: Some("Please try again or contact support if the issue persists.".to_string()),
            help_link: None,
            error_code: "GENERIC_ERROR".to_string(),
            technical_details: Some(self.clone()),
        }
    }
}

/// Helper function to convert any error to a user-friendly format
pub fn to_user_friendly_error<T: ToUserError>(error: T) -> String {
    let user_error = error.to_user_error();
    serde_json::to_string(&user_error).unwrap_or_else(|_| user_error.message)
}

/// Common user-friendly error scenarios
pub mod common {
    use super::UserError;
    
    pub fn ollama_not_running() -> UserError {
        UserError {
            title: "Ollama Service Required".to_string(),
            message: "GerdsenAI Socrates requires Ollama to be running for AI functionality.".to_string(),
            suggestion: Some("Please download and start Ollama from ollama.ai, then restart this application.".to_string()),
            help_link: Some("https://ollama.ai/download".to_string()),
            error_code: "OLLAMA_REQUIRED".to_string(),
            technical_details: None,
        }
    }
    
    pub fn model_not_available(model_name: &str) -> UserError {
        UserError {
            title: "AI Model Not Installed".to_string(),
            message: format!("The '{}' model is not available on your system.", model_name),
            suggestion: Some(format!("Install it by running: ollama pull {}", model_name)),
            help_link: Some("https://ollama.ai/library".to_string()),
            error_code: "MODEL_REQUIRED".to_string(),
            technical_details: None,
        }
    }
    
    pub fn service_temporarily_unavailable(service: &str) -> UserError {
        UserError {
            title: format!("{} Temporarily Unavailable", service),
            message: format!("The {} service is temporarily unavailable.", service),
            suggestion: Some("Some features may be limited. The application will automatically reconnect when the service is available.".to_string()),
            help_link: None,
            error_code: "SERVICE_UNAVAILABLE".to_string(),
            technical_details: None,
        }
    }
    
    pub fn installation_incomplete() -> UserError {
        UserError {
            title: "Setup Incomplete".to_string(),
            message: "Some required services are not properly configured.".to_string(),
            suggestion: Some("Please run the setup wizard or check the installation guide for complete setup instructions.".to_string()),
            help_link: Some("docs/WINDOWS_SETUP.md".to_string()),
            error_code: "SETUP_INCOMPLETE".to_string(),
            technical_details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ollama_connection_error() {
        let error = "connection refused to localhost:11434";
        let user_error = error.to_user_error();
        
        assert_eq!(user_error.error_code, "OLLAMA_OFFLINE");
        assert!(user_error.message.contains("Ollama"));
        assert!(user_error.suggestion.is_some());
    }
    
    #[test]
    fn test_model_not_found_error() {
        let error = "model 'llama2' does not exist";
        let user_error = error.to_user_error();
        
        assert_eq!(user_error.error_code, "MODEL_NOT_FOUND");
        assert!(user_error.message.contains("model"));
    }
    
    #[test]
    fn test_timeout_error() {
        let error = "request timed out after 30 seconds";
        let user_error = error.to_user_error();
        
        assert_eq!(user_error.error_code, "TIMEOUT");
        assert!(user_error.message.contains("too long"));
    }
}
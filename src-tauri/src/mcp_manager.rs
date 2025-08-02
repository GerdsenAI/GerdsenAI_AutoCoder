use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tokio::sync::{Mutex, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as TokioCommand;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// MCP Server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub connected: bool,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_connected: Option<DateTime<Utc>>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

/// MCP Connection handle
#[derive(Debug)]
struct MCPConnection {
    server_id: String,
    process: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
    stdout_reader: Option<BufReader<tokio::process::ChildStdout>>,
    tools: Vec<MCPTool>,
    connected: bool,
}

/// Main MCP Manager for handling multiple MCP servers
pub struct MCPManager {
    servers: RwLock<HashMap<String, MCPServer>>,
    connections: Mutex<HashMap<String, MCPConnection>>,
}

impl MCPManager {
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Add a new MCP server configuration
    pub async fn add_server(&self, config: MCPServerConfig) -> Result<MCPServer, String> {
        let server_id = Uuid::new_v4().to_string();
        let server = MCPServer {
            id: server_id.clone(),
            name: config.name,
            command: config.command,
            args: config.args,
            env: config.env,
            enabled: false, // Start disabled by default
            connected: false,
            last_error: None,
            created_at: Utc::now(),
            last_connected: None,
        };

        let mut servers = self.servers.write().await;
        servers.insert(server_id, server.clone());
        
        Ok(server)
    }

    /// Remove an MCP server
    pub async fn remove_server(&self, server_id: &str) -> Result<(), String> {
        // Disconnect if connected
        self.disconnect_server(server_id).await?;
        
        // Remove from servers list
        let mut servers = self.servers.write().await;
        servers.remove(server_id);
        
        Ok(())
    }

    /// Enable or disable an MCP server
    pub async fn toggle_server(&self, server_id: &str, enabled: bool) -> Result<(), String> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(server_id) {
            server.enabled = enabled;
            
            // If disabling, also disconnect
            if !enabled {
                drop(servers); // Release the write lock
                self.disconnect_server(server_id).await?;
            }
        } else {
            return Err("Server not found".to_string());
        }
        
        Ok(())
    }

    /// Connect to an MCP server
    pub async fn connect_server(&self, server_id: &str) -> Result<(), String> {
        let server = {
            let servers = self.servers.read().await;
            servers.get(server_id)
                .ok_or_else(|| "Server not found".to_string())?
                .clone()
        };

        if !server.enabled {
            return Err("Server is not enabled".to_string());
        }

        // Start the MCP server process
        let mut cmd = TokioCommand::new(&server.command);
        cmd.args(&server.args)
            .envs(&server.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to start MCP server: {}", e))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| "Failed to get stdin handle".to_string())?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| "Failed to get stdout handle".to_string())?;

        let stdout_reader = BufReader::new(stdout);

        // Create connection
        let connection = MCPConnection {
            server_id: server_id.to_string(),
            process: Some(child),
            stdin: Some(stdin),
            stdout_reader: Some(stdout_reader),
            tools: Vec::new(),
            connected: false,
        };

        // Store connection
        let mut connections = self.connections.lock().await;
        connections.insert(server_id.to_string(), connection);

        // Perform MCP handshake
        match self.perform_handshake(server_id).await {
            Ok(_) => {
                // Update server status
                let mut servers = self.servers.write().await;
                if let Some(server) = servers.get_mut(server_id) {
                    server.connected = true;
                    server.last_connected = Some(Utc::now());
                    server.last_error = None;
                }
                Ok(())
            }
            Err(e) => {
                // Clean up failed connection
                self.disconnect_server(server_id).await?;
                
                // Update server status
                let mut servers = self.servers.write().await;
                if let Some(server) = servers.get_mut(server_id) {
                    server.connected = false;
                    server.last_error = Some(e.clone());
                }
                
                Err(e)
            }
        }
    }

    /// Disconnect from an MCP server
    pub async fn disconnect_server(&self, server_id: &str) -> Result<(), String> {
        let mut connections = self.connections.lock().await;
        if let Some(mut connection) = connections.remove(server_id) {
            // Close stdin to signal shutdown
            if let Some(mut stdin) = connection.stdin.take() {
                let _ = stdin.shutdown().await;
            }
            
            // Terminate the process
            if let Some(mut process) = connection.process.take() {
                let _ = process.kill();
                let _ = process.wait().await;
            }
        }

        // Update server status
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(server_id) {
            server.connected = false;
        }

        Ok(())
    }

    /// Perform MCP protocol handshake
    async fn perform_handshake(&self, server_id: &str) -> Result<(), String> {
        // Send initialize request
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "GerdsenAI Socrates",
                    "version": "1.0.0"
                }
            }
        });

        self.send_request(server_id, &init_request).await?;
        
        // TODO: Read and parse response
        // For now, assume success if we can send the request
        
        // Send initialized notification
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        self.send_request(server_id, &initialized_notification).await?;

        // Discover available tools
        self.discover_tools(server_id).await?;

        Ok(())
    }

    /// Send a JSON-RPC request to an MCP server
    async fn send_request(&self, server_id: &str, request: &serde_json::Value) -> Result<(), String> {
        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.get_mut(server_id) {
            if let Some(stdin) = &mut connection.stdin {
                let request_str = serde_json::to_string(request)
                    .map_err(|e| format!("Failed to serialize request: {}", e))?;
                
                stdin.write_all(request_str.as_bytes()).await
                    .map_err(|e| format!("Failed to write to MCP server: {}", e))?;
                
                stdin.write_all(b"\n").await
                    .map_err(|e| format!("Failed to write newline to MCP server: {}", e))?;
                
                stdin.flush().await
                    .map_err(|e| format!("Failed to flush MCP server stdin: {}", e))?;
                
                Ok(())
            } else {
                Err("No stdin available for MCP server".to_string())
            }
        } else {
            Err("MCP server not connected".to_string())
        }
    }

    /// Discover tools available from an MCP server
    async fn discover_tools(&self, server_id: &str) -> Result<(), String> {
        let tools_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        self.send_request(server_id, &tools_request).await?;
        
        // TODO: Read response and parse tools
        // For now, create some mock tools for demonstration
        let mock_tools = vec![
            MCPTool {
                name: "example_tool".to_string(),
                description: Some("An example tool from this MCP server".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Input parameter"
                        }
                    },
                    "required": ["input"]
                }),
            }
        ];

        // Store discovered tools
        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.get_mut(server_id) {
            connection.tools = mock_tools;
            connection.connected = true;
        }

        Ok(())
    }

    /// Call a tool on an MCP server
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let tool_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": Uuid::new_v4().to_string(),
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        self.send_request(server_id, &tool_request).await?;
        
        // TODO: Read and parse response
        // For now, return a mock response
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Mock response from tool '{}' on server '{}'", tool_name, server_id)
                }
            ]
        }))
    }

    /// List all available tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<(String, MCPTool)> {
        let connections = self.connections.lock().await;
        let mut all_tools = Vec::new();

        for (server_id, connection) in connections.iter() {
            if connection.connected {
                for tool in &connection.tools {
                    all_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }

        all_tools
    }

    /// Get all configured servers
    pub async fn list_servers(&self) -> Vec<MCPServer> {
        let servers = self.servers.read().await;
        servers.values().cloned().collect()
    }

    /// Test connection to an MCP server
    pub async fn test_connection(&self, server_id: &str) -> Result<bool, String> {
        // First check if already connected
        {
            let connections = self.connections.lock().await;
            if let Some(connection) = connections.get(server_id) {
                if connection.connected {
                    return Ok(true);
                }
            }
        }

        // Try to connect
        match self.connect_server(server_id).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Update error status
                let mut servers = self.servers.write().await;
                if let Some(server) = servers.get_mut(server_id) {
                    server.last_error = Some(e.clone());
                    server.connected = false;
                }
                Err(e)
            }
        }
    }

    /// Auto-connect all enabled servers on startup
    pub async fn auto_connect_enabled_servers(&self) -> Vec<(String, Result<(), String>)> {
        let servers = {
            let servers_guard = self.servers.read().await;
            servers_guard.values()
                .filter(|server| server.enabled)
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut results = Vec::new();
        for server in servers {
            let result = self.connect_server(&server.id).await;
            results.push((server.id, result));
        }

        results
    }
}
use crate::mcp_manager::{MCPManager, MCPServer, MCPServerConfig, MCPTool};
use serde_json::Value;
use tauri::State;

/// List all configured MCP servers
#[tauri::command]
pub async fn list_mcp_servers(
    mcp_manager: State<'_, MCPManager>,
) -> Result<Vec<MCPServer>, String> {
    Ok(mcp_manager.list_servers().await)
}

/// Add a new MCP server
#[tauri::command]
pub async fn add_mcp_server(
    config: MCPServerConfig,
    mcp_manager: State<'_, MCPManager>,
) -> Result<MCPServer, String> {
    mcp_manager.add_server(config).await
}

/// Remove an MCP server
#[tauri::command]
pub async fn remove_mcp_server(
    server_id: String,
    mcp_manager: State<'_, MCPManager>,
) -> Result<(), String> {
    mcp_manager.remove_server(&server_id).await
}

/// Toggle an MCP server on/off
#[tauri::command]
pub async fn toggle_mcp_server(
    server_id: String,
    enabled: bool,
    mcp_manager: State<'_, MCPManager>,
) -> Result<(), String> {
    mcp_manager.toggle_server(&server_id, enabled).await
}

/// Test connection to an MCP server
#[tauri::command]
pub async fn test_mcp_connection(
    server_id: String,
    mcp_manager: State<'_, MCPManager>,
) -> Result<bool, String> {
    mcp_manager.test_connection(&server_id).await
}

/// Connect to an MCP server
#[tauri::command]
pub async fn connect_mcp_server(
    server_id: String,
    mcp_manager: State<'_, MCPManager>,
) -> Result<(), String> {
    mcp_manager.connect_server(&server_id).await
}

/// Disconnect from an MCP server
#[tauri::command]
pub async fn disconnect_mcp_server(
    server_id: String,
    mcp_manager: State<'_, MCPManager>,
) -> Result<(), String> {
    mcp_manager.disconnect_server(&server_id).await
}

/// List all available tools from all connected MCP servers
#[tauri::command]
pub async fn list_mcp_tools(
    mcp_manager: State<'_, MCPManager>,
) -> Result<Vec<(String, MCPTool)>, String> {
    Ok(mcp_manager.list_all_tools().await)
}

/// Call a tool on an MCP server
#[tauri::command]
pub async fn call_mcp_tool(
    server_id: String,
    tool_name: String,
    arguments: Value,
    mcp_manager: State<'_, MCPManager>,
) -> Result<Value, String> {
    mcp_manager.call_tool(&server_id, &tool_name, arguments).await
}

/// Auto-connect all enabled MCP servers (useful on app startup)
#[tauri::command]
pub async fn auto_connect_mcp_servers(
    mcp_manager: State<'_, MCPManager>,
) -> Result<Vec<(String, String)>, String> {
    let results = mcp_manager.auto_connect_enabled_servers().await;
    
    // Convert Result<(), String> to (String, String) for serialization
    let formatted_results = results
        .into_iter()
        .map(|(server_id, result)| {
            let status = match result {
                Ok(_) => "connected".to_string(),
                Err(e) => format!("error: {}", e),
            };
            (server_id, status)
        })
        .collect();
    
    Ok(formatted_results)
}
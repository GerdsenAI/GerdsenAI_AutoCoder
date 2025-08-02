# MCP Server Integration

## Overview

MCP (Model Context Protocol) integration allows users to extend AutoCoder's capabilities with additional tools while keeping core functionality built-in and reliable.

## Architecture

### Core Services (Built-in)
These remain as first-class integrated services:
- **Ollama** - AI model integration
- **SearXNG** - Web search capabilities  
- **ChromaDB** - Vector database for RAG
- **LSP Server** - Code intelligence

### MCP Extensions (User-configurable)
Users can add MCP servers for additional capabilities:
- File system operations
- GitHub integration
- Database connections
- Custom tools

## Implementation Design

### Frontend Configuration UI

Located in Settings ? Integrations panel (see `mockups/mcp-config-ui.html`):

```typescript
interface MCPServerConfig {
  id: string;
  name: string;
  enabled: boolean;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  autoStart: boolean;
}

interface MCPServerState {
  config: MCPServerConfig;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  availableTools?: Tool[];
  lastError?: string;
}
```

### Backend MCP Client

```rust
// src-tauri/src/mcp_manager.rs
use serde_json::Value;
use tokio::process::Command;

pub struct MCPManager {
    servers: HashMap<String, MCPServerConfig>,
    connections: HashMap<String, MCPConnection>,
}

pub struct MCPConnection {
    process: Child,
    rpc_client: JsonRpcClient,
    tools: Vec<Tool>,
}

impl MCPManager {
    pub async fn add_server(&mut self, config: MCPServerConfig) -> Result<()> {
        // Validate config
        // Save to user settings
        // Start if autoStart enabled
    }
    
    pub async fn connect(&mut self, server_id: &str) -> Result<()> {
        let config = self.servers.get(server_id)?;
        
        // Spawn MCP server process
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        
        let process = cmd.spawn()?;
        
        // Establish JSON-RPC connection
        let rpc_client = JsonRpcClient::connect(&process).await?;
        
        // Discover available tools
        let tools = rpc_client.list_tools().await?;
        
        self.connections.insert(server_id.to_string(), MCPConnection {
            process,
            rpc_client,
            tools,
        });
        
        Ok(())
    }
    
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value
    ) -> Result<Value> {
        let connection = self.connections.get(server_id)?;
        connection.rpc_client.call_tool(tool_name, args).await
    }
}
```

### Tauri Commands

```rust
#[tauri::command]
async fn mcp_add_server(
    state: State<'_, Arc<Mutex<MCPManager>>>,
    config: MCPServerConfig
) -> Result<()> {
    let mut manager = state.lock().await;
    manager.add_server(config).await
}

#[tauri::command]
async fn mcp_list_servers(
    state: State<'_, Arc<Mutex<MCPManager>>>
) -> Result<Vec<MCPServerState>> {
    let manager = state.lock().await;
    manager.list_servers()
}

#[tauri::command]
async fn mcp_connect(
    state: State<'_, Arc<Mutex<MCPManager>>>,
    server_id: String
) -> Result<()> {
    let mut manager = state.lock().await;
    manager.connect(&server_id).await
}

#[tauri::command]
async fn mcp_call_tool(
    state: State<'_, Arc<Mutex<MCPManager>>>,
    server_id: String,
    tool: String,
    args: Value
) -> Result<Value> {
    let manager = state.lock().await;
    manager.call_tool(&server_id, &tool, args).await
}
```

### Tool Integration in Chat

```typescript
// In ChatInterface.tsx
const [mcpTools, setMcpTools] = useState<MCPTool[]>([]);

// Discover available tools
useEffect(() => {
  const loadTools = async () => {
    const servers = await invoke('mcp_list_servers');
    const tools = servers
      .filter(s => s.status === 'connected')
      .flatMap(s => s.availableTools || []);
    setMcpTools(tools);
  };
  loadTools();
}, []);

// Show tools in UI
{mcpTools.length > 0 && (
  <div className="available-tools">
    <h4>Available MCP Tools:</h4>
    {mcpTools.map(tool => (
      <ToolCard 
        key={tool.id}
        tool={tool}
        onUse={(args) => handleToolUse(tool, args)}
      />
    ))}
  </div>
)}
```

## User Experience

### Adding a Server

1. User opens Settings ? Integrations
2. Clicks "Add Server"
3. Fills in:
   - Name: "GitHub Integration"
   - Command: `npx @modelcontextprotocol/server-github`
   - Environment: `GITHUB_TOKEN=xxx`
4. Clicks "Test Connection"
5. If successful, server appears in list with available tools

### Using MCP Tools

1. Tools appear in chat interface when connected
2. User can invoke tools directly: `@github search repositories query="rust mcp"`
3. Or let AI decide when to use tools based on context
4. Results integrate seamlessly into conversation

## Security Considerations

### Sandboxing
- MCP servers run in separate processes
- Limited file system access based on configuration
- No direct access to AutoCoder internals

### API Key Management
- Encrypted storage for sensitive environment variables
- Never exposed in UI after initial entry
- User can update/rotate keys in settings

### Permission Model
```typescript
interface MCPPermissions {
  allowFileSystem: boolean;
  allowNetwork: boolean;
  allowedPaths?: string[];
  maxMemory?: number;
  timeout?: number;
}
```

## Popular MCP Servers

### Quick-Add Gallery

```typescript
const POPULAR_SERVERS = [
  {
    id: 'filesystem',
    name: 'Filesystem Access',
    description: 'Read/write files with proper sandboxing',
    command: 'npx @modelcontextprotocol/server-filesystem',
    args: ['$PROJECT_DIR'],
    permissions: { allowFileSystem: true }
  },
  {
    id: 'github',  
    name: 'GitHub Integration',
    description: 'Access repos, issues, PRs',
    command: 'npx @modelcontextprotocol/server-github',
    env: { GITHUB_TOKEN: '' },
    permissions: { allowNetwork: true }
  },
  {
    id: 'sequential-thinking',
    name: 'Sequential Thinking',
    description: 'Complex reasoning and planning',
    command: 'npx @modelcontextprotocol/server-sequential-thinking',
    permissions: {}
  }
];
```

## Error Handling

### Connection Failures
- Clear error messages in UI
- Retry with exponential backoff
- Fallback to core functionality

### Tool Execution Errors
- Graceful degradation
- Error context in chat
- Suggest alternatives

## Performance Optimization

### Lazy Loading
- Only connect to servers when needed
- Cache tool definitions
- Reuse connections across sessions

### Resource Management
```rust
impl MCPManager {
    pub async fn cleanup_idle_connections(&mut self) {
        // Disconnect servers idle > 5 minutes
        // Free memory from unused tools
        // Kill zombie processes
    }
}
```

## Future Enhancements

1. **MCP Server Marketplace**
   - Community-contributed servers
   - Ratings and reviews
   - One-click installation

2. **Custom MCP Server Builder**
   - Visual tool creator
   - JavaScript/TypeScript templates
   - Built-in testing

3. **Tool Chaining**
   - Combine multiple MCP tools
   - Create workflows
   - Save as presets

## Migration Path

For users upgrading from current version:
1. Core functionality unchanged
2. MCP features appear in Settings
3. Gradual adoption - use when needed
4. Import community configurations

---

*MCP Integration: Extend your AutoCoder with the tools you need, when you need them.*
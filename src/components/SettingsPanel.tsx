import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './SettingsPanel.css';

interface MCPServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  connected: boolean;
  lastError?: string;
}

interface MCPServerConfig {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
}

const POPULAR_MCP_SERVERS = [
  {
    name: 'GitHub',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-github'],
    env: { GITHUB_PERSONAL_ACCESS_TOKEN: '' },
    description: 'Repository integration and issue management'
  },
  {
    name: 'Filesystem',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-filesystem', '--', '.'],
    env: {},
    description: 'Enhanced file operations and project navigation'
  },
  {
    name: 'Sequential Thinking',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-sequential-thinking'],
    env: {},
    description: 'Complex reasoning and step-by-step problem solving'
  },
  {
    name: 'Brave Search',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-brave-search'],
    env: { BRAVE_API_KEY: '' },
    description: 'Alternative web search capabilities'
  }
];

export const SettingsPanel: React.FC = () => {
  const [activeSection, setActiveSection] = useState<'general' | 'mcp' | 'advanced'>('general');
  const [mcpServers, setMcpServers] = useState<MCPServer[]>([]);
  const [showAddServer, setShowAddServer] = useState(false);
  const [newServerConfig, setNewServerConfig] = useState<MCPServerConfig>({
    name: '',
    command: '',
    args: [],
    env: {}
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load MCP servers on component mount
  useEffect(() => {
    loadMcpServers();
  }, []);

  const loadMcpServers = async () => {
    try {
      setLoading(true);
      // This would call a Tauri command to list configured MCP servers
      // For now, we'll use a placeholder
      const servers = await invoke<MCPServer[]>('list_mcp_servers').catch(() => []);
      setMcpServers(servers);
    } catch (err) {
      setError('Failed to load MCP servers');
      console.error('Error loading MCP servers:', err);
    } finally {
      setLoading(false);
    }
  };

  const addMcpServer = async (config: MCPServerConfig) => {
    try {
      setLoading(true);
      setError(null);
      
      // Call Tauri command to add MCP server
      const newServer = await invoke<MCPServer>('add_mcp_server', { config });
      setMcpServers(prev => [...prev, newServer]);
      setShowAddServer(false);
      setNewServerConfig({ name: '', command: '', args: [], env: {} });
    } catch (err: any) {
      setError(err.toString());
      console.error('Error adding MCP server:', err);
    } finally {
      setLoading(false);
    }
  };

  const removeMcpServer = async (serverId: string) => {
    try {
      setLoading(true);
      await invoke('remove_mcp_server', { serverId });
      setMcpServers(prev => prev.filter(s => s.id !== serverId));
    } catch (err: any) {
      setError(err.toString());
      console.error('Error removing MCP server:', err);
    } finally {
      setLoading(false);
    }
  };

  const toggleMcpServer = async (serverId: string, enabled: boolean) => {
    try {
      await invoke('toggle_mcp_server', { serverId, enabled });
      setMcpServers(prev => 
        prev.map(s => s.id === serverId ? { ...s, enabled } : s)
      );
    } catch (err: any) {
      setError(err.toString());
      console.error('Error toggling MCP server:', err);
    }
  };

  const testMcpConnection = async (serverId: string) => {
    try {
      const result = await invoke<boolean>('test_mcp_connection', { serverId });
      setMcpServers(prev => 
        prev.map(s => s.id === serverId ? 
          { ...s, connected: result, lastError: result ? undefined : 'Connection failed' } : s
        )
      );
    } catch (err: any) {
      setMcpServers(prev => 
        prev.map(s => s.id === serverId ? 
          { ...s, connected: false, lastError: err.toString() } : s
        )
      );
    }
  };

  const addPopularServer = (template: typeof POPULAR_MCP_SERVERS[0]) => {
    setNewServerConfig({
      name: template.name,
      command: template.command,
      args: template.args,
      env: template.env
    });
    setShowAddServer(true);
  };

  const handleArgsChange = (value: string) => {
    const args = value.split(' ').filter(arg => arg.trim() !== '');
    setNewServerConfig(prev => ({ ...prev, args }));
  };

  const handleEnvChange = (key: string, value: string) => {
    setNewServerConfig(prev => ({
      ...prev,
      env: { ...prev.env, [key]: value }
    }));
  };

  const addEnvVar = () => {
    const key = prompt('Environment variable name:');
    if (key && key.trim()) {
      setNewServerConfig(prev => ({
        ...prev,
        env: { ...prev.env, [key.trim()]: '' }
      }));
    }
  };

  const removeEnvVar = (key: string) => {
    setNewServerConfig(prev => {
      const { [key]: removed, ...rest } = prev.env;
      return { ...prev, env: rest };
    });
  };

  return (
    <div className="settings-panel">
      <div className="settings-header">
        <h2>Settings</h2>
        <div className="settings-sections">
          <button
            className={`section-tab ${activeSection === 'general' ? 'active' : ''}`}
            onClick={() => setActiveSection('general')}
          >
            General
          </button>
          <button
            className={`section-tab ${activeSection === 'mcp' ? 'active' : ''}`}
            onClick={() => setActiveSection('mcp')}
          >
            MCP Extensions
          </button>
          <button
            className={`section-tab ${activeSection === 'advanced' ? 'active' : ''}`}
            onClick={() => setActiveSection('advanced')}
          >
            Advanced
          </button>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="settings-content">
        {activeSection === 'general' && (
          <div className="general-settings">
            <h3>General Settings</h3>
            <div className="setting-group">
              <label>Theme</label>
              <p className="setting-description">Appearance settings are managed globally via the theme toggle in the header.</p>
            </div>
            <div className="setting-group">
              <label>Default Model</label>
              <p className="setting-description">Default model selection is managed via the model selector in the header.</p>
            </div>
            <div className="setting-group">
              <label>Analysis Preferences</label>
              <p className="setting-description">Deep Analysis Mode settings are configured per-conversation via the analysis mode selector.</p>
            </div>
          </div>
        )}

        {activeSection === 'mcp' && (
          <div className="mcp-settings">
            <div className="mcp-header">
              <h3>Model Context Protocol (MCP) Extensions</h3>
              <p className="section-description">
                Extend GerdsenAI Socrates with additional capabilities through MCP servers. 
                Core services (Ollama, SearXNG, ChromaDB, LSP) remain built-in.
              </p>
              <button 
                className="add-server-button"
                onClick={() => setShowAddServer(true)}
                disabled={loading}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M12 5v14"/>
                  <path d="M5 12h14"/>
                </svg>
                Add MCP Server
              </button>
            </div>

            {/* Popular Servers Quick Add */}
            <div className="popular-servers">
              <h4>Popular MCP Servers</h4>
              <div className="server-gallery">
                {POPULAR_MCP_SERVERS.map((server) => (
                  <div key={server.name} className="server-card">
                    <div className="server-info">
                      <h5>{server.name}</h5>
                      <p>{server.description}</p>
                    </div>
                    <button
                      className="quick-add-button"
                      onClick={() => addPopularServer(server)}
                      disabled={loading}
                    >
                      Quick Add
                    </button>
                  </div>
                ))}
              </div>
            </div>

            {/* Configured Servers */}
            <div className="configured-servers">
              <h4>Configured Servers</h4>
              {mcpServers.length === 0 ? (
                <div className="no-servers">
                  <p>No MCP servers configured. Add one to extend functionality.</p>
                </div>
              ) : (
                <div className="servers-list">
                  {mcpServers.map((server) => (
                    <div key={server.id} className="server-item">
                      <div className="server-details">
                        <div className="server-name-status">
                          <h5>{server.name}</h5>
                          <div className="server-status">
                            <div className={`status-indicator ${server.connected ? 'connected' : 'disconnected'}`}></div>
                            <span>{server.connected ? 'Connected' : 'Disconnected'}</span>
                          </div>
                        </div>
                        <div className="server-command">
                          <code>{server.command} {server.args.join(' ')}</code>
                        </div>
                        {server.lastError && (
                          <div className="server-error">
                            <span>Error: {server.lastError}</span>
                          </div>
                        )}
                      </div>
                      <div className="server-actions">
                        <label className="toggle-switch">
                          <input
                            type="checkbox"
                            checked={server.enabled}
                            onChange={(e) => toggleMcpServer(server.id, e.target.checked)}
                            disabled={loading}
                          />
                          <span className="slider"></span>
                        </label>
                        <button
                          className="test-button"
                          onClick={() => testMcpConnection(server.id)}
                          disabled={loading}
                        >
                          Test
                        </button>
                        <button
                          className="remove-button"
                          onClick={() => removeMcpServer(server.id)}
                          disabled={loading}
                        >
                          Remove
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Add Server Modal */}
            {showAddServer && (
              <div className="modal-overlay">
                <div className="add-server-modal">
                  <div className="modal-header">
                    <h4>Add MCP Server</h4>
                    <button 
                      className="close-button"
                      onClick={() => setShowAddServer(false)}
                    >
                      ×
                    </button>
                  </div>
                  <div className="modal-content">
                    <div className="form-group">
                      <label>Server Name</label>
                      <input
                        type="text"
                        value={newServerConfig.name}
                        onChange={(e) => setNewServerConfig(prev => ({ ...prev, name: e.target.value }))}
                        placeholder="e.g., GitHub Integration"
                      />
                    </div>
                    <div className="form-group">
                      <label>Command</label>
                      <input
                        type="text"
                        value={newServerConfig.command}
                        onChange={(e) => setNewServerConfig(prev => ({ ...prev, command: e.target.value }))}
                        placeholder="e.g., npx"
                      />
                    </div>
                    <div className="form-group">
                      <label>Arguments</label>
                      <input
                        type="text"
                        value={newServerConfig.args.join(' ')}
                        onChange={(e) => handleArgsChange(e.target.value)}
                        placeholder="e.g., -y @modelcontextprotocol/server-github"
                      />
                    </div>
                    <div className="form-group">
                      <label>Environment Variables</label>
                      <div className="env-vars">
                        {Object.entries(newServerConfig.env).map(([key, value]) => (
                          <div key={key} className="env-var">
                            <input
                              type="text"
                              value={key}
                              disabled
                              className="env-key"
                            />
                            <input
                              type="text"
                              value={value}
                              onChange={(e) => handleEnvChange(key, e.target.value)}
                              placeholder="Value"
                              className="env-value"
                            />
                            <button
                              type="button"
                              onClick={() => removeEnvVar(key)}
                              className="remove-env-button"
                            >
                              ×
                            </button>
                          </div>
                        ))}
                        <button
                          type="button"
                          onClick={addEnvVar}
                          className="add-env-button"
                        >
                          Add Environment Variable
                        </button>
                      </div>
                    </div>
                  </div>
                  <div className="modal-actions">
                    <button
                      onClick={() => setShowAddServer(false)}
                      className="cancel-button"
                      disabled={loading}
                    >
                      Cancel
                    </button>
                    <button
                      onClick={() => addMcpServer(newServerConfig)}
                      className="add-button"
                      disabled={loading || !newServerConfig.name || !newServerConfig.command}
                    >
                      {loading ? 'Adding...' : 'Add Server'}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {activeSection === 'advanced' && (
          <div className="advanced-settings">
            <h3>Advanced Settings</h3>
            <div className="setting-group">
              <label>Performance</label>
              <p className="setting-description">
                Advanced performance settings are automatically managed by the application.
              </p>
            </div>
            <div className="setting-group">
              <label>Debugging</label>
              <p className="setting-description">
                Debug information is available in the developer console (F12).
              </p>
            </div>
            <div className="setting-group">
              <label>Reset</label>
              <p className="setting-description">
                To reset all settings, clear browser data or reinstall the application.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default SettingsPanel;
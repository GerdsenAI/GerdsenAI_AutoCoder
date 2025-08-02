import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './MultiAIModelSelector.css';

// Types
interface AIModel {
  id: string;
  name: string;
  provider: 'Ollama' | 'OpenAI' | 'Anthropic';
  capabilities: string[];
  context_length: number;
  cost_per_token?: number;
  speed_tokens_per_second?: number;
  is_available: boolean;
  description: string;
}

interface ModelListResponse {
  models: AIModel[];
  provider_health: Record<string, boolean>;
}

interface ProviderStatus {
  healthy: boolean;
  model_count: number;
  last_checked: string;
}

interface ProviderHealthResponse {
  providers: Record<string, ProviderStatus>;
}

interface MultiAIConfig {
  default_provider: 'Ollama' | 'OpenAI' | 'Anthropic';
  openai_api_key?: string;
  anthropic_api_key?: string;
  task_routing: Record<string, string>;
  enabled_providers: string[];
}

interface Props {
  onModelSelect: (modelId: string) => void;
  selectedModel?: string;
  onConfigChange?: (config: MultiAIConfig) => void;
  showConfiguration?: boolean;
}

export const MultiAIModelSelector: React.FC<Props> = ({
  onModelSelect,
  selectedModel,
  onConfigChange,
  showConfiguration = false,
}) => {
  const [models, setModels] = useState<AIModel[]>([]);
  const [providerHealth, setProviderHealth] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showConfig, setShowConfig] = useState(showConfiguration);
  const [config, setConfig] = useState<MultiAIConfig>({
    default_provider: 'Ollama',
    openai_api_key: '',
    anthropic_api_key: '',
    task_routing: {},
    enabled_providers: ['Ollama'],
  });
  
  const [filterProvider, setFilterProvider] = useState<string>('All');
  const [filterCapability, setFilterCapability] = useState<string>('All');
  const [sortBy, setSortBy] = useState<'name' | 'provider' | 'speed' | 'cost'>('name');

  useEffect(() => {
    loadModels();
    loadConfig();
    loadProviderHealth();
  }, []);

  const loadModels = async () => {
    try {
      setLoading(true);
      const response: ModelListResponse = await invoke('get_all_ai_models');
      setModels(response.models);
      setProviderHealth(response.provider_health);
      setError(null);
    } catch (err) {
      setError(err as string);
      console.error('Failed to load AI models:', err);
    } finally {
      setLoading(false);
    }
  };

  const loadConfig = async () => {
    try {
      const currentConfig: MultiAIConfig = await invoke('get_multi_ai_config');
      setConfig(currentConfig);
    } catch (err) {
      console.error('Failed to load config:', err);
    }
  };

  const loadProviderHealth = async () => {
    try {
      const healthResponse: ProviderHealthResponse = await invoke('get_provider_health');
      const healthMap: Record<string, boolean> = {};
      Object.entries(healthResponse.providers).forEach(([provider, status]) => {
        healthMap[provider] = status.healthy;
      });
      setProviderHealth(healthMap);
    } catch (err) {
      console.error('Failed to load provider health:', err);
    }
  };

  const saveConfig = async (newConfig: MultiAIConfig) => {
    try {
      await invoke('update_multi_ai_config', { config: newConfig });
      setConfig(newConfig);
      onConfigChange?.(newConfig);
      await loadModels(); // Reload models after config change
    } catch (err) {
      setError(err as string);
      console.error('Failed to save config:', err);
    }
  };

  const handleProviderToggle = async (provider: string, enabled: boolean) => {
    const newConfig = {
      ...config,
      enabled_providers: enabled
        ? [...config.enabled_providers, provider]
        : config.enabled_providers.filter(p => p !== provider),
    };
    await saveConfig(newConfig);
  };

  const handleApiKeyChange = async (provider: 'openai' | 'anthropic', apiKey: string) => {
    const newConfig = {
      ...config,
      ...(provider === 'openai' 
        ? { openai_api_key: apiKey }
        : { anthropic_api_key: apiKey }
      ),
    };
    await saveConfig(newConfig);
  };

  const getProviderIcon = (provider: string) => {
    switch (provider) {
      case 'Ollama': return '🦙';
      case 'OpenAI': return '🤖';
      case 'Anthropic': return '🧠';
      default: return '❓';
    }
  };

  const getProviderStatusIcon = (provider: string) => {
    const isHealthy = providerHealth[provider];
    return isHealthy ? '🟢' : '🔴';
  };

  const getCostDisplay = (model: AIModel) => {
    if (!model.cost_per_token) return 'Free';
    return `$${(model.cost_per_token * 1000).toFixed(3)}/1K tokens`;
  };

  const getSpeedDisplay = (model: AIModel) => {
    if (!model.speed_tokens_per_second) return 'Unknown';
    return `${model.speed_tokens_per_second.toFixed(0)} tok/s`;
  };

  const filteredAndSortedModels = models
    .filter(model => {
      if (filterProvider !== 'All' && model.provider !== filterProvider) return false;
      if (filterCapability !== 'All' && !model.capabilities.includes(filterCapability)) return false;
      return model.is_available;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'provider':
          return a.provider.localeCompare(b.provider);
        case 'speed':
          return (b.speed_tokens_per_second || 0) - (a.speed_tokens_per_second || 0);
        case 'cost':
          return (a.cost_per_token || 0) - (b.cost_per_token || 0);
        default:
          return a.name.localeCompare(b.name);
      }
    });

  const capabilities = Array.from(
    new Set(models.flatMap(model => model.capabilities))
  ).sort();

  const providers = Array.from(new Set(models.map(model => model.provider))).sort();

  if (loading) {
    return (
      <div className="multi-ai-selector loading">
        <div className="loading-spinner">🔄</div>
        <p>Loading AI models...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="multi-ai-selector error">
        <div className="error-icon">❌</div>
        <p>Error loading models: {error}</p>
        <button onClick={loadModels} className="retry-button">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="multi-ai-selector">
      <div className="selector-header">
        <h3>AI Model Selection</h3>
        <div className="header-actions">
          <button
            onClick={() => setShowConfig(!showConfig)}
            className="config-toggle"
            title="Configure providers"
          >
            ⚙️
          </button>
          <button
            onClick={loadModels}
            className="refresh-button"
            title="Refresh models"
          >
            🔄
          </button>
        </div>
      </div>

      {showConfig && (
        <div className="configuration-panel">
          <h4>Provider Configuration</h4>
          
          <div className="provider-configs">
            <div className="provider-config">
              <div className="provider-header">
                <span className="provider-name">
                  {getProviderIcon('Ollama')} Ollama {getProviderStatusIcon('Ollama')}
                </span>
                <label className="toggle-switch">
                  <input
                    type="checkbox"
                    checked={config.enabled_providers.includes('Ollama')}
                    onChange={(e) => handleProviderToggle('Ollama', e.target.checked)}
                  />
                  <span className="slider"></span>
                </label>
              </div>
              <p className="provider-description">
                Local AI models via Ollama. Free and private.
              </p>
            </div>

            <div className="provider-config">
              <div className="provider-header">
                <span className="provider-name">
                  {getProviderIcon('OpenAI')} OpenAI {getProviderStatusIcon('OpenAI')}
                </span>
                <label className="toggle-switch">
                  <input
                    type="checkbox"
                    checked={config.enabled_providers.includes('OpenAI')}
                    onChange={(e) => handleProviderToggle('OpenAI', e.target.checked)}
                  />
                  <span className="slider"></span>
                </label>
              </div>
              <div className="provider-details">
                <input
                  type="password"
                  placeholder="OpenAI API Key"
                  value={config.openai_api_key || ''}
                  onChange={(e) => handleApiKeyChange('openai', e.target.value)}
                  className="api-key-input"
                />
                <p className="provider-description">
                  GPT models from OpenAI. Requires API key and has usage costs.
                </p>
              </div>
            </div>

            <div className="provider-config">
              <div className="provider-header">
                <span className="provider-name">
                  {getProviderIcon('Anthropic')} Anthropic {getProviderStatusIcon('Anthropic')}
                </span>
                <label className="toggle-switch">
                  <input
                    type="checkbox"
                    checked={config.enabled_providers.includes('Anthropic')}
                    onChange={(e) => handleProviderToggle('Anthropic', e.target.checked)}
                  />
                  <span className="slider"></span>
                </label>
              </div>
              <div className="provider-details">
                <input
                  type="password"
                  placeholder="Anthropic API Key"
                  value={config.anthropic_api_key || ''}
                  onChange={(e) => handleApiKeyChange('anthropic', e.target.value)}
                  className="api-key-input"
                />
                <p className="provider-description">
                  Claude models from Anthropic. Requires API key and has usage costs.
                </p>
              </div>
            </div>
          </div>

          <div className="default-provider">
            <label>Default Provider:</label>
            <select
              value={config.default_provider}
              onChange={(e) => saveConfig({
                ...config,
                default_provider: e.target.value as any,
              })}
            >
              <option value="Ollama">Ollama</option>
              <option value="OpenAI">OpenAI</option>
              <option value="Anthropic">Anthropic</option>
            </select>
          </div>
        </div>
      )}

      <div className="model-filters">
        <div className="filter-group">
          <label>Provider:</label>
          <select
            value={filterProvider}
            onChange={(e) => setFilterProvider(e.target.value)}
          >
            <option value="All">All Providers</option>
            {providers.map(provider => (
              <option key={provider} value={provider}>
                {getProviderIcon(provider)} {provider}
              </option>
            ))}
          </select>
        </div>

        <div className="filter-group">
          <label>Capability:</label>
          <select
            value={filterCapability}
            onChange={(e) => setFilterCapability(e.target.value)}
          >
            <option value="All">All Capabilities</option>
            {capabilities.map(capability => (
              <option key={capability} value={capability}>
                {capability.replace(/([A-Z])/g, ' $1').trim()}
              </option>
            ))}
          </select>
        </div>

        <div className="filter-group">
          <label>Sort by:</label>
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
          >
            <option value="name">Name</option>
            <option value="provider">Provider</option>
            <option value="speed">Speed</option>
            <option value="cost">Cost</option>
          </select>
        </div>
      </div>

      <div className="model-list">
        {filteredAndSortedModels.length === 0 ? (
          <div className="no-models">
            <p>No models available with current filters.</p>
            <p>Try adjusting your filters or configuring additional providers.</p>
          </div>
        ) : (
          filteredAndSortedModels.map(model => (
            <div
              key={model.id}
              className={`model-item ${selectedModel === model.id ? 'selected' : ''}`}
              onClick={() => onModelSelect(model.id)}
            >
              <div className="model-header">
                <div className="model-name">
                  <span className="provider-icon">
                    {getProviderIcon(model.provider)}
                  </span>
                  <span className="name">{model.name}</span>
                  <span className="provider-badge">{model.provider}</span>
                </div>
                <div className="model-stats">
                  <span className="cost" title="Cost per 1K tokens">
                    {getCostDisplay(model)}
                  </span>
                  <span className="speed" title="Tokens per second">
                    {getSpeedDisplay(model)}
                  </span>
                </div>
              </div>
              
              <div className="model-details">
                <p className="description">{model.description}</p>
                <div className="capabilities">
                  {model.capabilities.slice(0, 4).map(capability => (
                    <span key={capability} className="capability-tag">
                      {capability.replace(/([A-Z])/g, ' $1').trim()}
                    </span>
                  ))}
                  {model.capabilities.length > 4 && (
                    <span className="capability-tag more">
                      +{model.capabilities.length - 4} more
                    </span>
                  )}
                </div>
                <div className="model-info">
                  <span className="context-length" title="Context length">
                    📄 {model.context_length.toLocaleString()} tokens
                  </span>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="provider-status-bar">
        <div className="status-title">Provider Status:</div>
        {Object.entries(providerHealth).map(([provider, healthy]) => (
          <div key={provider} className={`provider-status ${healthy ? 'healthy' : 'unhealthy'}`}>
            <span className="provider-icon">{getProviderIcon(provider)}</span>
            <span className="provider-name">{provider}</span>
            <span className="status-indicator">{healthy ? '🟢' : '🔴'}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
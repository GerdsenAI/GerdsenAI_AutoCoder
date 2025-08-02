import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Types
export interface AIModel {
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

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;  
  total_tokens: number;
  estimated_cost?: number;
}

export interface AIGenerationRequest {
  prompt: string;
  model_id?: string;
  capability?: string;
  options?: {
    temperature?: number;
    max_tokens?: number;
    top_p?: number;
    top_k?: number;
    stop_sequences?: string[];
    stream?: boolean;
  };
  stream?: boolean;
}

export interface AIGenerationResponse {
  content: string;
  model: string;
  provider: string;
  usage?: TokenUsage;
  finish_reason?: string;
  metadata: Record<string, any>;
}

export interface MultiAIConfig {
  default_provider: 'Ollama' | 'OpenAI' | 'Anthropic';
  openai_api_key?: string;
  anthropic_api_key?: string;
  task_routing: Record<string, string>;
  enabled_providers: string[];
}

export interface ProviderStatus {
  healthy: boolean;
  model_count: number;
  last_checked: string;
}

export interface ModelListResponse {
  models: AIModel[];
  provider_health: Record<string, boolean>;
}

export interface ProviderHealthResponse {
  providers: Record<string, ProviderStatus>;
}

// Custom hook for multi-AI functionality
export const useMultiAI = () => {
  const [models, setModels] = useState<AIModel[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [config, setConfig] = useState<MultiAIConfig>({
    default_provider: 'Ollama',
    openai_api_key: '',
    anthropic_api_key: '',
    task_routing: {},
    enabled_providers: ['Ollama'],
  });
  const [providerHealth, setProviderHealth] = useState<Record<string, ProviderStatus>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [initialized, setInitialized] = useState(false);

  // Initialize the multi-AI system
  const initialize = useCallback(async (initialConfig?: MultiAIConfig) => {
    try {
      setLoading(true);
      setError(null);
      
      const configToUse = initialConfig || config;
      await invoke('initialize_multi_ai', { config: configToUse });
      
      // Load models and provider health
      await Promise.all([
        loadModels(),
        loadProviderHealth(),
        loadConfig(),
      ]);
      
      setInitialized(true);
    } catch (err) {
      setError(err as string);
      console.error('Failed to initialize multi-AI:', err);
    } finally {
      setLoading(false);
    }
  }, [config]);

  // Load available models
  const loadModels = useCallback(async () => {
    try {
      const response: ModelListResponse = await invoke('get_all_ai_models');
      setModels(response.models);
      
      // Auto-select first available model if none selected
      if (!selectedModel && response.models.length > 0) {
        const firstAvailable = response.models.find(m => m.is_available);
        if (firstAvailable) {
          setSelectedModel(firstAvailable.id);
        }
      }
      
      return response;
    } catch (err) {
      setError(err as string);
      throw err;
    }
  }, [selectedModel]);

  // Load provider health status
  const loadProviderHealth = useCallback(async () => {
    try {
      const response: ProviderHealthResponse = await invoke('get_provider_health');
      setProviderHealth(response.providers);
      return response;
    } catch (err) {
      console.error('Failed to load provider health:', err);
      throw err;
    }
  }, []);

  // Load current configuration
  const loadConfig = useCallback(async () => {
    try {
      const currentConfig: MultiAIConfig = await invoke('get_multi_ai_config');
      setConfig(currentConfig);
      return currentConfig;
    } catch (err) {
      console.error('Failed to load config:', err);
      throw err;
    }
  }, []);

  // Update configuration
  const updateConfig = useCallback(async (newConfig: Partial<MultiAIConfig>) => {
    try {
      setLoading(true);
      const updatedConfig = { ...config, ...newConfig };
      
      await invoke('update_multi_ai_config', { config: updatedConfig });
      setConfig(updatedConfig);
      
      // Reload models after config change
      await loadModels();
      await loadProviderHealth();
      
      return updatedConfig;
    } catch (err) {
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [config, loadModels, loadProviderHealth]);

  // Generate AI completion
  const generate = useCallback(async (request: AIGenerationRequest): Promise<AIGenerationResponse> => {
    try {
      setLoading(true);
      setError(null);
      
      const response: AIGenerationResponse = await invoke('generate_ai_smart', { request });
      return response;
    } catch (err) {
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // Generate with specific model
  const generateWithModel = useCallback(async (
    modelId: string,
    prompt: string,
    options?: AIGenerationRequest['options']
  ): Promise<AIGenerationResponse> => {
    return generate({
      prompt,
      model_id: modelId,
      options,
    });
  }, [generate]);

  // Generate with smart capability-based routing
  const generateSmart = useCallback(async (
    prompt: string,
    capability?: string,
    options?: AIGenerationRequest['options']
  ): Promise<AIGenerationResponse> => {
    return generate({
      prompt,
      capability,
      options,
    });
  }, [generate]);

  // Get model information
  const getModelInfo = useCallback(async (modelId: string): Promise<AIModel> => {
    try {
      const model: AIModel = await invoke('get_ai_model_info', { modelId });
      return model;
    } catch (err) {
      throw err;
    }
  }, []);

  // Classify prompt to determine capability
  const classifyPrompt = useCallback(async (prompt: string): Promise<string> => {
    try {
      const capability: string = await invoke('classify_prompt', { prompt });
      return capability;
    } catch (err) {
      throw err;
    }
  }, []);

  // Get available capabilities
  const getCapabilities = useCallback(async (): Promise<string[]> => {
    try {
      const capabilities: string[] = await invoke('get_model_capabilities');
      return capabilities;
    } catch (err) {
      throw err;
    }
  }, []);

  // Get supported providers
  const getSupportedProviders = useCallback(async (): Promise<string[]> => {
    try {
      const providers: string[] = await invoke('get_supported_providers');
      return providers;
    } catch (err) {
      throw err;
    }
  }, []);

  // Get models by capability
  const getModelsByCapability = useCallback((capability: string): AIModel[] => {
    return models.filter(model => 
      model.is_available && model.capabilities.includes(capability)
    );
  }, [models]);

  // Get models by provider
  const getModelsByProvider = useCallback((provider: string): AIModel[] => {
    return models.filter(model => 
      model.is_available && model.provider === provider
    );
  }, [models]);

  // Get best model for capability
  const getBestModelForCapability = useCallback((capability: string): AIModel | null => {
    const capableModels = getModelsByCapability(capability);
    if (capableModels.length === 0) return null;
    
    // Sort by speed and cost to find the best balance
    return capableModels.sort((a, b) => {
      const aScore = (a.speed_tokens_per_second || 0) - (a.cost_per_token || 0) * 1000;
      const bScore = (b.speed_tokens_per_second || 0) - (b.cost_per_token || 0) * 1000;
      return bScore - aScore;
    })[0];
  }, [getModelsByCapability]);

  // Refresh all data
  const refresh = useCallback(async () => {
    await Promise.all([
      loadModels(),
      loadProviderHealth(),
    ]);
  }, [loadModels, loadProviderHealth]);

  // Auto-initialize on mount
  useEffect(() => {
    if (!initialized) {
      initialize();
    }
  }, [initialize, initialized]);

  // Computed values
  const availableModels = models.filter(model => model.is_available);
  const healthyProviders = Object.entries(providerHealth)
    .filter(([_, status]) => status.healthy)
    .map(([provider]) => provider);
  
  const selectedModelInfo = models.find(model => model.id === selectedModel);
  
  const isProviderHealthy = (provider: string): boolean => {
    return providerHealth[provider]?.healthy || false;
  };

  const getProviderModelCount = (provider: string): number => {
    return providerHealth[provider]?.model_count || 0;
  };

  return {
    // State
    models,
    availableModels,
    selectedModel,
    selectedModelInfo,
    config,
    providerHealth,
    healthyProviders,
    loading,
    error,
    initialized,

    // Actions
    initialize,
    loadModels,
    loadProviderHealth,
    loadConfig,
    updateConfig,
    generate,
    generateWithModel,
    generateSmart,
    getModelInfo,
    classifyPrompt,
    getCapabilities,
    getSupportedProviders,
    refresh,

    // Model selection
    setSelectedModel,
    getModelsByCapability,
    getModelsByProvider,
    getBestModelForCapability,

    // Provider utilities
    isProviderHealthy,
    getProviderModelCount,

    // Error handling
    clearError: () => setError(null),
  };
};
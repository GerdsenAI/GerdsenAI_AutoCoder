import { useState, useCallback } from 'react';
import './ContextControls.css';

export interface ContextSettings {
  model: string;
  reservedTokens: number;
  contextStrategy: 'balanced' | 'conversation' | 'documentation' | 'code';
  autoIncludeDependencies: boolean;
  maxTokens: number;
}

interface ModelInfo {
  name: string;
  maxTokens: number;
  available: boolean;
}

interface ContextControlsProps {
  settings: ContextSettings;
  onSettingsChange: (settings: ContextSettings) => void;
  availableModels?: ModelInfo[];
  className?: string;
}

const DEFAULT_MODELS: ModelInfo[] = [
  { name: 'llama3.2:3b', maxTokens: 128000, available: true },
  { name: 'llama3.1:8b', maxTokens: 128000, available: true },
  { name: 'llama3.1:70b', maxTokens: 128000, available: true },
  { name: 'codellama:13b', maxTokens: 16384, available: true },
  { name: 'mistral:7b', maxTokens: 32768, available: true },
];

const CONTEXT_STRATEGIES = [
  {
    value: 'balanced' as const,
    label: 'Balanced',
    description: 'Equal priority for conversation, files, and documentation'
  },
  {
    value: 'conversation' as const,
    label: 'Conversation Priority',
    description: 'Prioritize chat history and recent context'
  },
  {
    value: 'documentation' as const,
    label: 'Documentation Heavy',
    description: 'Include more documentation and README files'
  },
  {
    value: 'code' as const,
    label: 'Code Analysis',
    description: 'Focus on source code and dependencies'
  }
];

export const ContextControls = ({
  settings,
  onSettingsChange,
  availableModels = DEFAULT_MODELS,
  className = ''
}: ContextControlsProps) => {
  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleModelChange = useCallback(async (modelName: string) => {
    const selectedModel = availableModels.find(m => m.name === modelName);
    if (!selectedModel) return;

    setIsLoading(true);
    try {
      const newSettings = {
        ...settings,
        model: modelName,
        maxTokens: selectedModel.maxTokens,
        // Adjust reserved tokens if they exceed new model's capacity
        reservedTokens: Math.min(settings.reservedTokens, selectedModel.maxTokens * 0.3)
      };
      onSettingsChange(newSettings);
      setErrors(prev => ({ ...prev, model: '' }));
    } catch (error) {
      setErrors(prev => ({ ...prev, model: 'Failed to switch model' }));
    } finally {
      setIsLoading(false);
    }
  }, [settings, onSettingsChange, availableModels]);

  const handleReservedTokensChange = useCallback((value: string) => {
    const numValue = parseInt(value, 10);
    if (isNaN(numValue) || numValue < 0) {
      setErrors(prev => ({ ...prev, reservedTokens: 'Must be a positive number' }));
      return;
    }

    if (numValue > settings.maxTokens * 0.5) {
      setErrors(prev => ({ ...prev, reservedTokens: 'Cannot exceed 50% of total tokens' }));
      return;
    }

    setErrors(prev => ({ ...prev, reservedTokens: '' }));
    onSettingsChange({
      ...settings,
      reservedTokens: numValue
    });
  }, [settings, onSettingsChange]);

  const handleStrategyChange = useCallback((strategy: ContextSettings['contextStrategy']) => {
    onSettingsChange({
      ...settings,
      contextStrategy: strategy
    });
  }, [settings, onSettingsChange]);

  const handleAutoIncludeToggle = useCallback(() => {
    onSettingsChange({
      ...settings,
      autoIncludeDependencies: !settings.autoIncludeDependencies
    });
  }, [settings, onSettingsChange]);

  const formatTokens = useCallback((tokens: number): string => {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(0)}k`;
    }
    return tokens.toString();
  }, []);

  const getSelectedStrategy = useCallback(() => {
    return CONTEXT_STRATEGIES.find(s => s.value === settings.contextStrategy) || CONTEXT_STRATEGIES[0];
  }, [settings.contextStrategy]);

  const selectedModel = availableModels.find(m => m.name === settings.model);

  return (
    <div className={`context-controls ${className}`}>
      <div className="control-panel">
        <div className="section-header">
          <div className="section-title">
            <div className="section-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M12 1v6m0 6v6"/>
                <path d="m21 12-6-3 6-3v6"/>
                <path d="m3 12 6 3-6 3v-6"/>
              </svg>
            </div>
            <span>Context Controls</span>
          </div>
        </div>
        
        <div className="controls-grid">
          {/* Model Selection */}
          <div className="control-group">
            <label className="control-label">
              Model
              {selectedModel && (
                <span className="model-info">
                  ({formatTokens(selectedModel.maxTokens)} tokens)
                </span>
              )}
            </label>
            <div className="select-wrapper">
              <select
                className={`control-input ${errors.model ? 'error' : ''}`}
                value={settings.model}
                onChange={(e) => handleModelChange(e.target.value)}
                disabled={isLoading}
              >
                {availableModels.map((model) => (
                  <option key={model.name} value={model.name} disabled={!model.available}>
                    {model.name} ({formatTokens(model.maxTokens)})
                    {!model.available && ' - Unavailable'}
                  </option>
                ))}
              </select>
              <div className="select-arrow">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M6 9l6 6 6-6"/>
                </svg>
              </div>
            </div>
            {errors.model && <div className="error-message">{errors.model}</div>}
          </div>
          
          {/* Reserved Tokens */}
          <div className="control-group">
            <label className="control-label">
              Reserved Tokens
              <span className="control-hint">
                ({Math.round((settings.reservedTokens / settings.maxTokens) * 100)}% of total)
              </span>
            </label>
            <input
              type="number"
              className={`control-input ${errors.reservedTokens ? 'error' : ''}`}
              value={settings.reservedTokens}
              onChange={(e) => handleReservedTokensChange(e.target.value)}
              min="0"
              max={Math.floor(settings.maxTokens * 0.5)}
              step="1024"
            />
            {errors.reservedTokens && <div className="error-message">{errors.reservedTokens}</div>}
          </div>
          
          {/* Context Strategy */}
          <div className="control-group">
            <label className="control-label">
              Context Strategy
              <span className="control-hint">{getSelectedStrategy().description}</span>
            </label>
            <div className="select-wrapper">
              <select
                className="control-input"
                value={settings.contextStrategy}
                onChange={(e) => handleStrategyChange(e.target.value as ContextSettings['contextStrategy'])}
              >
                {CONTEXT_STRATEGIES.map((strategy) => (
                  <option key={strategy.value} value={strategy.value}>
                    {strategy.label}
                  </option>
                ))}
              </select>
              <div className="select-arrow">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M6 9l6 6 6-6"/>
                </svg>
              </div>
            </div>
          </div>
          
          {/* Auto-Include Dependencies */}
          <div className="control-group">
            <label className="control-label">
              Auto-Include Dependencies
              <span className="control-hint">
                Automatically include related files and imports
              </span>
            </label>
            <div className="toggle-switch" onClick={handleAutoIncludeToggle}>
              <div className={`toggle ${settings.autoIncludeDependencies ? 'active' : ''}`}>
                <div className="toggle-ball" />
              </div>
              <span className="toggle-label">
                {settings.autoIncludeDependencies ? 'Enabled' : 'Disabled'}
              </span>
            </div>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="control-stats">
          <div className="stat-item">
            <span className="stat-label">Total Capacity</span>
            <span className="stat-value">{formatTokens(settings.maxTokens)}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Reserved</span>
            <span className="stat-value">{formatTokens(settings.reservedTokens)}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Available</span>
            <span className="stat-value">
              {formatTokens(settings.maxTokens - settings.reservedTokens)}
            </span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Strategy</span>
            <span className="stat-value">{getSelectedStrategy().label}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ContextControls;

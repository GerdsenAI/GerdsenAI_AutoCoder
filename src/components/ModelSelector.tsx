import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { validateModelName } from '../utils/validation';
import './ModelSelector.css';

export interface ModelInfo {
  name: string;
  parameter_size?: string;
  size_mb: number;
  quantization?: string;
}

interface ModelSelectorProps {
  onModelSelect: (model: string) => void;
  selectedModel: string;
}

export const ModelSelector: React.FC<ModelSelectorProps> = ({ onModelSelect, selectedModel }) => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');
  const [customUrl, setCustomUrl] = useState('');
  const [showCustomUrlInput, setShowCustomUrlInput] = useState(false);

  // Fetch available models on component mount
  useEffect(() => {
    fetchModels();
  }, []);

  // Check Ollama connection status
  useEffect(() => {
    checkConnection();
  }, []);

  const checkConnection = async () => {
    try {
      setConnectionStatus('checking');
      const isConnected = await invoke<boolean>('check_ollama_connection');
      setConnectionStatus(isConnected ? 'connected' : 'disconnected');
    } catch (error) {
      console.error('Failed to check Ollama connection:', error);
      setConnectionStatus('disconnected');
    }
  };

  const fetchModels = async () => {
    try {
      setLoading(true);
      setError(null);
      const modelList = await invoke<ModelInfo[]>('list_models');
      setModels(modelList);
    } catch (error) {
      console.error('Failed to fetch models:', error);
      setError('Failed to fetch models. Please check your Ollama connection.');
    } finally {
      setLoading(false);
    }
  };

  const handleModelSelect = (model: string) => {
    // Validate model name before selection
    const validation = validateModelName(model);
    if (!validation.isValid) {
      console.error('Invalid model name:', validation.error);
      alert(`Cannot select model: ${validation.error}`);
      return;
    }
    
    onModelSelect(validation.sanitized!);
    setIsOpen(false);
  };

  const toggleDropdown = () => {
    setIsOpen(!isOpen);
  };

  const handleCustomUrlSubmit = async () => {
    try {
      setConnectionStatus('checking');
      const isConnected = await invoke<boolean>('check_ollama_connection', { baseUrl: customUrl });
      
      if (isConnected) {
        setConnectionStatus('connected');
        await fetchModels();
        setShowCustomUrlInput(false);
      } else {
        setConnectionStatus('disconnected');
        setError('Could not connect to Ollama at the specified URL');
      }
    } catch (error) {
      console.error('Failed to connect to custom Ollama URL:', error);
      setConnectionStatus('disconnected');
      setError('Failed to connect to custom Ollama URL');
    }
  };

  const formatSize = (sizeInMb: number): string => {
    if (sizeInMb >= 1000) {
      return `${(sizeInMb / 1000).toFixed(1)} GB`;
    }
    return `${sizeInMb.toFixed(0)} MB`;
  };

  return (
    <div className="model-selector-container">
      <div 
        className={`model-selector ${isOpen ? 'open' : ''}`} 
        onClick={toggleDropdown}
      >
        <div className="selected-model">
          <div className="model-icon">
            <span className="icon">🦙</span>
          </div>
          <div className="model-info">
            <span className="model-name">{selectedModel || 'Select a model'}</span>
            {(() => {
              const selectedModelInfo = selectedModel && models && Array.isArray(models) 
                ? models.find(m => m.name === selectedModel) 
                : null;
              
              return selectedModelInfo?.parameter_size ? (
                <span className="model-details">
                  {selectedModelInfo.parameter_size} • 
                  {selectedModelInfo.quantization && ` ${selectedModelInfo.quantization} •`} 
                  {formatSize(selectedModelInfo.size_mb)}
                </span>
              ) : null;
            })()}
          </div>
        </div>
        <div className="connection-status">
          <span className={`status-indicator ${connectionStatus}`}></span>
          <span className="status-text">
            {connectionStatus === 'connected' ? 'Connected' : 
             connectionStatus === 'checking' ? 'Checking...' : 'Disconnected'}
          </span>
        </div>
        <div className="dropdown-arrow">
          <span className={`arrow ${isOpen ? 'up' : 'down'}`}>▼</span>
        </div>
      </div>

      {isOpen && (
        <div className="model-dropdown">
          {loading ? (
            <div className="loading-indicator">Loading models...</div>
          ) : error ? (
            <div className="error-message">{error}</div>
          ) : models.length === 0 ? (
            <div className="no-models">No models available</div>
          ) : (
            <div className="model-list">
              {models.map((model) => (
                <div 
                  key={model.name} 
                  className={`model-item ${selectedModel === model.name ? 'selected' : ''}`}
                  onClick={() => handleModelSelect(model.name)}
                >
                  <div className="model-item-name">{model.name}</div>
                  <div className="model-item-details">
                    {model.parameter_size && <span>{model.parameter_size}</span>}
                    {model.quantization && <span>{model.quantization}</span>}
                    <span>{formatSize(model.size_mb)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="model-dropdown-footer">
            {showCustomUrlInput ? (
              <div className="custom-url-input">
                <input
                  type="text"
                  value={customUrl}
                  onChange={(e) => setCustomUrl(e.target.value)}
                  placeholder="http://localhost:11434"
                />
                <button onClick={handleCustomUrlSubmit}>Connect</button>
                <button className="cancel" onClick={() => setShowCustomUrlInput(false)}>Cancel</button>
              </div>
            ) : (
              <>
                <button className="refresh-button" onClick={fetchModels}>
                  Refresh Models
                </button>
                <button className="custom-url-button" onClick={() => setShowCustomUrlInput(true)}>
                  Custom Ollama URL
                </button>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ModelSelector;

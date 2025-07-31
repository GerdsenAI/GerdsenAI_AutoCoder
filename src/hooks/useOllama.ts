import { useState, useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface OllamaMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface OllamaResponse {
  model: string;
  created_at: string;
  message: OllamaMessage;
  done: boolean;
  total_duration?: number;
  load_duration?: number;
  prompt_eval_count?: number;
  prompt_eval_duration?: number;
  eval_count?: number;
  eval_duration?: number;
  context?: number[];
}

export interface OllamaStreamResponse {
  model: string;
  created_at: string;
  message: OllamaMessage;
  done: boolean;
  context?: number[];
}

export interface StreamEvent {
  type: 'token' | 'done' | 'error';
  payload: {
    token?: string;
    response?: OllamaResponse;
    context?: number[];
    error?: string;
  };
}

export interface OllamaModelInfo {
  name: string;
  parameter_size?: string;
  size_mb: number;
  quantization?: string;
  modified_at?: string;
  family?: string;
}

export interface UseOllamaOptions {
  baseUrl?: string;
  onError?: (error: Error) => void;
}

export interface OllamaCallOptions {
  temperature?: number;
  topP?: number;
  topK?: number;
  maxTokens?: number;
  context?: number[];
}

/**
 * Custom hook for interacting with Ollama API
 * This implementation uses the Tauri IPC bridge to communicate with the Rust backend
 * which handles the actual API calls to Ollama
 */
export function useOllama(options: UseOllamaOptions = {}) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [models, setModels] = useState<OllamaModelInfo[]>([]);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');

  // Clean up any event listeners when the component unmounts
  useEffect(() => {
    const cleanupListeners: (() => void)[] = [];
    
    return () => {
      cleanupListeners.forEach(cleanup => cleanup());
    };
  }, []);

  const checkConnection = useCallback(async (baseUrl?: string) => {
    try {
      setConnectionStatus('checking');
      const isConnected = await invoke<boolean>('check_ollama_connection', { baseUrl });
      setConnectionStatus(isConnected ? 'connected' : 'disconnected');
      return isConnected;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setConnectionStatus('disconnected');
      setError(error);
      options.onError?.(error);
      return false;
    }
  }, [options.onError]);

  const listModels = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const modelList = await invoke<OllamaModelInfo[]>('list_models', { baseUrl: options.baseUrl });
      setModels(modelList);
      return modelList;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options.baseUrl, options.onError]);

  const chat = useCallback(async (
    model: string,
    messages: OllamaMessage[],
    callOptions?: OllamaCallOptions
  ): Promise<OllamaResponse> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const response = await invoke<OllamaResponse>('chat_with_ollama', {
        model,
        messages,
        temperature: callOptions?.temperature,
        topP: callOptions?.topP,
        topK: callOptions?.topK,
        maxTokens: callOptions?.maxTokens,
        context: callOptions?.context,
        baseUrl: options.baseUrl,
      });
      
      return response;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options.baseUrl, options.onError]);

  const chatStream = useCallback(async (
    model: string,
    messages: OllamaMessage[],
    onToken: (token: string, context?: number[]) => void,
    onComplete?: (response: OllamaResponse) => void,
    callOptions?: OllamaCallOptions
  ): Promise<void> => {
    let unlistenCallback: (() => void) | undefined;
    
    try {
      setIsLoading(true);
      setError(null);
      
      // Set up event listener for streaming tokens
      const unlisten = await listen<StreamEvent>('ollama-stream', (event) => {
        const { type, payload } = event.payload;
        
        if (type === 'token' && payload.token) {
          onToken(payload.token, payload.context);
        } else if (type === 'done' && payload.response) {
          onComplete?.(payload.response);
          unlistenCallback?.();
        } else if (type === 'error' && payload.error) {
          throw new Error(payload.error);
        }
      });
      
      unlistenCallback = unlisten;
      
      // Start the streaming process
      await invoke('chat_stream_with_ollama', {
        model,
        messages,
        temperature: callOptions?.temperature,
        topP: callOptions?.topP,
        topK: callOptions?.topK,
        maxTokens: callOptions?.maxTokens,
        context: callOptions?.context,
        baseUrl: options.baseUrl,
      });
      
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options.baseUrl, options.onError]);

  const generate = useCallback(async (
    model: string,
    prompt: string,
    callOptions?: OllamaCallOptions
  ): Promise<OllamaResponse> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const response = await invoke<OllamaResponse>('generate_with_ollama', {
        model,
        prompt,
        temperature: callOptions?.temperature,
        topP: callOptions?.topP,
        topK: callOptions?.topK,
        maxTokens: callOptions?.maxTokens,
        context: callOptions?.context,
        baseUrl: options.baseUrl,
      });
      
      return response;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options.baseUrl, options.onError]);

  const generateStream = useCallback(async (
    model: string,
    prompt: string,
    onToken: (token: string, context?: number[]) => void,
    onComplete?: (response: OllamaResponse) => void,
    callOptions?: OllamaCallOptions
  ): Promise<void> => {
    let unlistenCallback: (() => void) | undefined;
    
    try {
      setIsLoading(true);
      setError(null);
      
      // Set up event listener for streaming tokens
      const unlisten = await listen<StreamEvent>('ollama-stream', (event) => {
        const { type, payload } = event.payload;
        
        if (type === 'token' && payload.token) {
          onToken(payload.token, payload.context);
        } else if (type === 'done' && payload.response) {
          onComplete?.(payload.response);
          unlistenCallback?.();
        } else if (type === 'error' && payload.error) {
          throw new Error(payload.error);
        }
      });
      
      unlistenCallback = unlisten;
      
      // Start the streaming process
      await invoke('generate_stream_with_ollama', {
        model,
        prompt,
        temperature: callOptions?.temperature,
        topP: callOptions?.topP,
        topK: callOptions?.topK,
        maxTokens: callOptions?.maxTokens,
        context: callOptions?.context,
        baseUrl: options.baseUrl,
      });
      
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options.baseUrl, options.onError]);

  return {
    isLoading,
    error,
    models,
    connectionStatus,
    checkConnection,
    listModels,
    chat,
    chatStream,
    generate,
    generateStream,
  };
}

export default useOllama;

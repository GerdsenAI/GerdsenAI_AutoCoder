import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SearchResult {
  title: string;
  url: string;
  content: string;
  source: string;
  score: number;
  date?: string;
}

export interface SearchOptions {
  engines?: string[];
  categories?: string[];
  maxResults?: number;
  timeRange?: 'all' | 'day' | 'week' | 'month' | 'year';
}

export interface UseSearchOptions {
  baseUrl?: string;
  onError?: (error: Error) => void;
}

export function useSearch(options: UseSearchOptions = {}) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');

  const checkConnection = useCallback(async (baseUrl?: string) => {
    try {
      setConnectionStatus('checking');
      const isConnected = await invoke<boolean>('check_searxng_connection', { baseUrl });
      setConnectionStatus(isConnected ? 'connected' : 'disconnected');
      return isConnected;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setConnectionStatus('disconnected');
      setError(error);
      options.onError?.(error);
      return false;
    }
  }, [options]);

  const search = useCallback(async (
    query: string,
    searchOptions?: SearchOptions
  ) => {
    try {
      setIsLoading(true);
      setError(null);
      
      const searchResults = await invoke<SearchResult[]>('search_web', {
        query,
        engines: searchOptions?.engines,
        categories: searchOptions?.categories,
        limit: searchOptions?.maxResults,
      });
      
      setResults(searchResults);
      return searchResults;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const getAvailableEngines = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const engines = await invoke<string[]>('get_available_engines');
      
      return engines;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const getAvailableCategories = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const categories = await invoke<string[]>('get_available_categories');
      
      return categories;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  return {
    isLoading,
    error,
    results,
    connectionStatus,
    checkConnection,
    search,
    getAvailableEngines,
    getAvailableCategories,
  };
}

export default useSearch;

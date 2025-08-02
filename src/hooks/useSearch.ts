import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SearchResult {
  title: string;
  url: string;
  content: string;
  engine: string;
  score?: number;
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
  autoLoadEngines?: boolean;
  defaultActiveEngines?: string[];
}

export function useSearch(options: UseSearchOptions = {}) {
  const { 
    onError, 
    autoLoadEngines = true,
    defaultActiveEngines = ['github', 'stackoverflow', 'google']
  } = options;

  // Core state
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');
  
  // Engine management state
  const [availableEngines, setAvailableEngines] = useState<string[]>([
    'github',
    'stackoverflow',
    'google',
    'duckduckgo',
    'bing',
    'brave',
    'documentation',
    'forums'
  ]);
  const [activeEngines, setActiveEngines] = useState<string[]>(defaultActiveEngines);
  const [isHealthy, setIsHealthy] = useState<boolean>(true);

  // Load available engines on mount if autoLoad is enabled
  useEffect(() => {
    if (autoLoadEngines) {
      loadAvailableEngines();
    }
  }, [autoLoadEngines]);

  const handleError = useCallback((err: unknown, defaultMessage: string) => {
    const errorMessage = err instanceof Error ? err.message : defaultMessage;
    setError(errorMessage);
    if (onError) {
      onError(err instanceof Error ? err : new Error(defaultMessage));
    }
  }, [onError]);

  const checkConnection = useCallback(async (baseUrl?: string) => {
    try {
      setConnectionStatus('checking');
      const isConnected = await invoke<boolean>('check_searxng_connection', { baseUrl });
      setConnectionStatus(isConnected ? 'connected' : 'disconnected');
      setIsHealthy(isConnected);
      return isConnected;
    } catch (err) {
      setConnectionStatus('disconnected');
      setIsHealthy(false);
      handleError(err, 'Failed to check search service connection');
      return false;
    }
  }, [handleError]);

  const loadAvailableEngines = useCallback(async () => {
    try {
      setError(null);
      const engines = await invoke<string[]>('get_available_engines');
      
      if (engines && engines.length > 0) {
        setAvailableEngines(engines);
        // Update active engines if none are selected or they don't exist in available engines
        setActiveEngines(prev => {
          const validEngines = prev.filter(engine => engines.includes(engine));
          return validEngines.length > 0 ? validEngines : engines.slice(0, 3);
        });
      }
      
      return engines;
    } catch (err) {
      handleError(err, 'Failed to load available search engines');
      throw err;
    }
  }, [handleError]);

  const performSearch = useCallback(async (
    searchQuery?: string,
    searchOptions?: SearchOptions
  ) => {
    const finalQuery = searchQuery || query;
    const finalEngines = searchOptions?.engines || activeEngines;
    
    if (!finalQuery.trim()) {
      return [];
    }

    // Check if search service is healthy before attempting search
    if (!isHealthy) {
      console.warn('Search service is not healthy, attempting search anyway...');
    }

    try {
      setIsLoading(true);
      setError(null);
      setResults([]); // Clear previous results
      
      const searchResults = await invoke<SearchResult[]>('search_web', {
        query: finalQuery.trim(),
        engines: finalEngines,
        limit: searchOptions?.maxResults || 10
      });
      
      setResults(searchResults);
      return searchResults;
    } catch (err) {
      handleError(err, 'Search failed');
      
      // Show user-friendly error message when service is unavailable
      if (!isHealthy) {
        handleError(
          new Error('Search service is currently unavailable'),
          'Search service is currently unavailable. Please check the service status or try again later.'
        );
      }
      
      return [];
    } finally {
      setIsLoading(false);
    }
  }, [query, activeEngines, isHealthy, handleError]);

  const search = useCallback(async () => {
    return await performSearch();
  }, [performSearch]);

  const toggleEngine = useCallback((engine: string) => {
    setActiveEngines(prev => 
      prev.includes(engine)
        ? prev.filter(e => e !== engine)
        : [...prev, engine]
    );
  }, []);

  const setSearchQuery = useCallback((newQuery: string) => {
    setQuery(newQuery);
    setError(null); // Clear error when query changes
  }, []);

  const updateHealthStatus = useCallback((healthy: boolean) => {
    setIsHealthy(healthy);
    setConnectionStatus(healthy ? 'connected' : 'disconnected');
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const clearResults = useCallback(() => {
    setResults([]);
  }, []);

  const reset = useCallback(() => {
    setQuery('');
    setResults([]);
    setError(null);
    setIsLoading(false);
  }, []);

  const getAvailableCategories = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const categories = await invoke<string[]>('get_available_categories');
      
      return categories;
    } catch (err) {
      handleError(err, 'Failed to load available categories');
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [handleError]);

  return {
    // State
    query,
    results,
    isLoading,
    error,
    connectionStatus,
    availableEngines,
    activeEngines,
    isHealthy,
    
    // Actions
    setQuery: setSearchQuery,
    search,
    performSearch,
    toggleEngine,
    loadAvailableEngines,
    updateHealthStatus,
    checkConnection,
    clearError,
    clearResults,
    reset,
    getAvailableCategories,
    
    // Computed
    hasResults: (results || []).length > 0,
    canSearch: query.trim().length > 0 && !isLoading,
    hasActiveEngines: (activeEngines || []).length > 0,
  };
}

export default useSearch;

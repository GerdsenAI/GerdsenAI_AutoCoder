import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ContextBudget } from '../components/TokenBudgetBar';

export interface ContextSettings {
  model: string;
  reservedTokens: number;
  contextStrategy: 'balanced' | 'conversation' | 'documentation' | 'code';
  autoIncludeDependencies: boolean;
  maxTokens: number;
}

export interface FileInfo {
  path: string;
  token_count: number;
  relevance_score: number;
  is_pinned: boolean;
  file_type: string;
}

export interface ContextManagerState {
  budget: ContextBudget | null;
  files: FileInfo[];
  settings: ContextSettings;
  loading: boolean;
  error: string | null;
}

export interface ContextManagerActions {
  refreshBudget: () => Promise<void>;
  pinFile: (path: string) => Promise<void>;
  unpinFile: (path: string) => Promise<void>;
  updateSettings: (settings: Partial<ContextSettings>) => Promise<void>;
  buildContext: () => Promise<string>;
  calculateFileRelevance: (path: string) => Promise<number>;
}

export const useContextManager = (): ContextManagerState & ContextManagerActions => {
  const [budget, setBudget] = useState<ContextBudget | null>(null);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [settings, setSettings] = useState<ContextSettings>({
    model: 'llama3.1:8b',
    reservedTokens: 2048,
    contextStrategy: 'balanced',
    autoIncludeDependencies: true,
    maxTokens: 128000
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Refresh context budget from backend
  const refreshBudget = useCallback(async () => {
    try {
      setError(null);
      const budgetData = await invoke<ContextBudget>('get_context_budget');
      setBudget(budgetData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to get context budget';
      setError(errorMessage);
      console.error('Error getting context budget:', err);
    }
  }, []);

  // Pin a file to context with optimistic updates
  const pinFile = useCallback(async (path: string) => {
    try {
      setError(null);
      
      // Optimistic update - mark file as pinned immediately
      setFiles(prevFiles => 
        prevFiles.map(file => 
          file.path === path 
            ? { ...file, is_pinned: true }
            : file
        )
      );

      // Call backend
      await invoke('pin_file', { path });
      
      // Refresh budget to show updated allocations
      await refreshBudget();
    } catch (err) {
      // Revert optimistic update on error
      setFiles(prevFiles => 
        prevFiles.map(file => 
          file.path === path 
            ? { ...file, is_pinned: false }
            : file
        )
      );
      
      const errorMessage = err instanceof Error ? err.message : 'Failed to pin file';
      setError(errorMessage);
      console.error('Error pinning file:', err);
    }
  }, [refreshBudget]);

  // Unpin a file from context with optimistic updates
  const unpinFile = useCallback(async (path: string) => {
    try {
      setError(null);
      
      // Optimistic update - mark file as unpinned immediately
      setFiles(prevFiles => 
        prevFiles.map(file => 
          file.path === path 
            ? { ...file, is_pinned: false }
            : file
        )
      );

      // Call backend
      await invoke('unpin_file', { path });
      
      // Refresh budget to show updated allocations
      await refreshBudget();
    } catch (err) {
      // Revert optimistic update on error
      setFiles(prevFiles => 
        prevFiles.map(file => 
          file.path === path 
            ? { ...file, is_pinned: true }
            : file
        )
      );
      
      const errorMessage = err instanceof Error ? err.message : 'Failed to unpin file';
      setError(errorMessage);
      console.error('Error unpinning file:', err);
    }
  }, [refreshBudget]);

  // Update context settings
  const updateSettings = useCallback(async (newSettings: Partial<ContextSettings>) => {
    try {
      setError(null);
      setLoading(true);
      
      const updatedSettings = { ...settings, ...newSettings };
      setSettings(updatedSettings);
      
      // If model changed, refresh budget to get new token limits
      if (newSettings.model && newSettings.model !== settings.model) {
        await refreshBudget();
      }
      
      // If strategy changed, recalculate file relevance scores
      if (newSettings.contextStrategy && newSettings.contextStrategy !== settings.contextStrategy) {
        // Refresh file list with new relevance scores
        const updatedFiles = await Promise.all(
          files.map(async (file) => {
            const relevance_score = await calculateFileRelevance(file.path);
            return { ...file, relevance_score };
          })
        );
        setFiles(updatedFiles);
      }
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update settings';
      setError(errorMessage);
      console.error('Error updating settings:', err);
    } finally {
      setLoading(false);
    }
  }, [settings, files, refreshBudget]);

  // Calculate file relevance score
  const calculateFileRelevance = useCallback(async (path: string): Promise<number> => {
    try {
      const relevanceScore = await invoke<number>('calculate_file_relevance', { 
        path,
        strategy: settings.contextStrategy 
      });
      return relevanceScore;
    } catch (err) {
      console.error('Error calculating file relevance:', err);
      return 0;
    }
  }, [settings.contextStrategy]);

  // Build complete context string for AI
  const buildContext = useCallback(async (): Promise<string> => {
    try {
      setError(null);
      setLoading(true);
      
      const contextString = await invoke<string>('build_context', {
        settings: {
          model: settings.model,
          reserved_tokens: settings.reservedTokens,
          strategy: settings.contextStrategy,
          include_auto_files: settings.autoIncludeDependencies
        }
      });
      
      return contextString;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to build context';
      setError(errorMessage);
      console.error('Error building context:', err);
      return '';
    } finally {
      setLoading(false);
    }
  }, [settings]);

  // Load initial data on mount
  useEffect(() => {
    const initializeContextManager = async () => {
      setLoading(true);
      try {
        await refreshBudget();
        
        // Load initial file list (this would typically come from a file explorer or project analysis)
        // For now, we'll start with an empty list and files will be added as they're discovered
        setFiles([]);
        
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to initialize context manager';
        setError(errorMessage);
        console.error('Error initializing context manager:', err);
      } finally {
        setLoading(false);
      }
    };

    initializeContextManager();
  }, [refreshBudget]);

  // Memoized sorted files for better performance
  const sortedFiles = useMemo(() => {
    return [...files].sort((a, b) => {
      // Pinned files first
      if (a.is_pinned !== b.is_pinned) {
        return a.is_pinned ? -1 : 1;
      }
      // Then by relevance score (higher first)
      return b.relevance_score - a.relevance_score;
    });
  }, [files]);

  return {
    // State
    budget,
    files: sortedFiles,
    settings,
    loading,
    error,
    
    // Actions
    refreshBudget,
    pinFile,
    unpinFile,
    updateSettings,
    buildContext,
    calculateFileRelevance
  };
};

export default useContextManager;

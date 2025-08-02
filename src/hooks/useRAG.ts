import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface RAGDocument {
  id: string;
  document: string;
  metadata: {
    source: string;
    document_type: string;
    language?: string;
    timestamp: string;
    title?: string;
    file_path?: string;
    url?: string;
    [key: string]: any;
  };
  distance: number;
}

export interface RAGCollection {
  name: string;
  count: number;
}

export interface UseRAGOptions {
  onError?: (error: Error) => void;
  autoLoadCollections?: boolean;
}

export function useRAG(options: UseRAGOptions = {}) {
  const { onError, autoLoadCollections = true } = options;

  // Core state
  const [collections, setCollections] = useState<RAGCollection[]>([]);
  const [selectedCollection, setSelectedCollection] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load collections on mount if autoLoad is enabled
  useEffect(() => {
    if (autoLoadCollections) {
      loadCollections();
    }
  }, [autoLoadCollections]);

  // Clear error when selected collection changes
  useEffect(() => {
    setError(null);
  }, [selectedCollection]);

  const handleError = useCallback((err: unknown, defaultMessage: string) => {
    const errorMessage = err instanceof Error ? err.message : defaultMessage;
    setError(errorMessage);
    if (onError) {
      onError(err instanceof Error ? err : new Error(defaultMessage));
    }
  }, [onError]);

  const loadCollections = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const collectionList = await invoke<string[]>('list_chroma_collections');
      
      // Get counts for each collection
      const collectionsWithCounts = await Promise.all(
        collectionList.map(async (name) => {
          try {
            const count = await invoke<number>('get_collection_count', { 
              collectionName: name 
            });
            return { name, count };
          } catch (err) {
            console.warn(`Failed to get count for collection ${name}:`, err);
            return { name, count: 0 };
          }
        })
      );

      setCollections(collectionsWithCounts);
      
      // Auto-select first collection if none selected
      if (!selectedCollection && collectionsWithCounts.length > 0) {
        setSelectedCollection(collectionsWithCounts[0].name);
      }

    } catch (err) {
      handleError(err, 'Failed to load collections');
    } finally {
      setLoading(false);
    }
  }, [selectedCollection, handleError]);

  const createCollection = useCallback(async (name: string) => {
    if (!name.trim()) {
      throw new Error('Collection name cannot be empty');
    }

    try {
      setLoading(true);
      setError(null);

      await invoke('create_chroma_collection', { 
        collectionName: name.trim() 
      });
      
      // Reload collections to get updated list
      await loadCollections();
      
      // Select the newly created collection
      setSelectedCollection(name.trim());

    } catch (err) {
      handleError(err, 'Failed to create collection');
      throw err; // Re-throw for component to handle
    } finally {
      setLoading(false);
    }
  }, [loadCollections, handleError]);

  const searchDocuments = useCallback(async (
    query: string, 
    collectionName?: string,
    options: { nResults?: number; filter?: Record<string, any> } = {}
  ): Promise<RAGDocument[]> => {
    const collection = collectionName || selectedCollection;
    if (!collection || !query.trim()) {
      return [];
    }

    try {
      setLoading(true);
      setError(null);

      const results = await invoke<RAGDocument[]>('query_chroma', {
        request: {
          collectionName: collection,
          queryText: query.trim(),
          nResults: options.nResults || 10,
          filter: options.filter || null
        }
      });

      return results;

    } catch (err) {
      handleError(err, 'Search failed');
      return [];
    } finally {
      setLoading(false);
    }
  }, [selectedCollection, handleError]);

  const uploadDocument = useCallback(async (
    text: string,
    metadata: Partial<RAGDocument['metadata']> = {},
    collectionName?: string
  ) => {
    const collection = collectionName || selectedCollection;
    if (!collection || !text.trim()) {
      throw new Error('Collection and document text are required');
    }

    try {
      setLoading(true);
      setError(null);

      const documentMetadata = {
        source: 'manual_upload',
        document_type: 'text',
        language: 'en',
        timestamp: new Date().toISOString(),
        title: 'Untitled Document',
        ...metadata
      };

      await invoke('add_documents_to_chroma', {
        request: {
          collectionName: collection,
          documents: [text.trim()],
          metadatas: [documentMetadata],
          ids: null
        }
      });

      // Refresh collection counts
      await loadCollections();

    } catch (err) {
      handleError(err, 'Failed to upload document');
      throw err; // Re-throw for component to handle
    } finally {
      setLoading(false);
    }
  }, [selectedCollection, loadCollections, handleError]);

  const deleteCollection = useCallback(async (name: string) => {
    if (!name) {
      throw new Error('Collection name is required');
    }

    try {
      setLoading(true);
      setError(null);

      await invoke('delete_chroma_collection', { 
        collectionName: name 
      });
      
      // Reload collections
      await loadCollections();
      
      // If we deleted the selected collection, clear selection
      if (selectedCollection === name) {
        setSelectedCollection('');
      }

    } catch (err) {
      handleError(err, 'Failed to delete collection');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [loadCollections, selectedCollection, handleError]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    // State
    collections,
    selectedCollection,
    loading,
    error,
    
    // Actions  
    setSelectedCollection,
    loadCollections,
    createCollection,
    searchDocuments,
    uploadDocument,
    deleteCollection,
    clearError,
    
    // Computed
    hasCollections: collections.length > 0,
    selectedCollectionData: collections.find(c => c.name === selectedCollection),
  };
}

export default useRAG;
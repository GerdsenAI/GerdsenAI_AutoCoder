import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ChromaDocument {
  id: string;
  text: string;
  metadata: Record<string, any>;
  embedding?: number[];
}

export interface ChromaCollection {
  name: string;
  count: number;
}

export interface ChromaQueryResult {
  id: string;
  text: string;
  metadata: Record<string, any>;
  distance: number;
}

export interface UseChromaOptions {
  onError?: (error: Error) => void;
}

export function useChroma(options: UseChromaOptions = {}) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [collections, setCollections] = useState<ChromaCollection[]>([]);

  const listCollections = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const collectionList = await invoke<ChromaCollection[]>('list_chroma_collections');
      setCollections(collectionList);
      return collectionList;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const createCollection = useCallback(async (name: string) => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke('create_chroma_collection', { name });
      await listCollections();
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [listCollections, options]);

  const deleteCollection = useCallback(async (name: string) => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke('delete_chroma_collection', { name });
      await listCollections();
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [listCollections, options]);

  const addDocuments = useCallback(async (
    collectionName: string,
    documents: string[],
    metadatas: Record<string, any>[],
    ids?: string[]
  ) => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke('add_documents_to_chroma', {
        collectionName,
        documents,
        metadatas,
        ids
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const query = useCallback(async (
    collectionName: string,
    queryText: string,
    nResults: number = 5,
    filter?: Record<string, any>
  ) => {
    try {
      setIsLoading(true);
      setError(null);
      
      const results = await invoke<ChromaQueryResult[]>('query_chroma', {
        collectionName,
        queryText,
        nResults,
        filter: filter ? JSON.stringify(filter) : undefined
      });
      
      return results;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const getDocuments = useCallback(async (
    collectionName: string,
    ids?: string[],
    filter?: Record<string, any>
  ) => {
    try {
      setIsLoading(true);
      setError(null);
      
      const documents = await invoke<ChromaDocument[]>('get_documents_from_chroma', {
        collectionName,
        ids,
        filter: filter ? JSON.stringify(filter) : undefined
      });
      
      return documents;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [options]);

  const deleteDocuments = useCallback(async (
    collectionName: string,
    ids: string[]
  ) => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke('delete_documents_from_chroma', {
        collectionName,
        ids
      });
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
    collections,
    listCollections,
    createCollection,
    deleteCollection,
    addDocuments,
    query,
    getDocuments,
    deleteDocuments
  };
}

export default useChroma;

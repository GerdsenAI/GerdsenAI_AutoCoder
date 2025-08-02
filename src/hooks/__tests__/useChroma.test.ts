import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useChroma } from '../useChroma';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core');

describe('useChroma', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize with default values', () => {
      const { result } = renderHook(() => useChroma());

      expect(result.current.collections).toEqual([]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe(null);
    });

    it('should load collections when listCollections is called', async () => {
      const mockCollections = [
        { name: 'default', count: 10 },
        { name: 'test', count: 5 }
      ];

      mockInvoke.mockResolvedValueOnce(mockCollections);

      const { result } = renderHook(() => useChroma());

      await act(async () => {
        await result.current.listCollections();
      });

      await waitFor(() => {
        expect(result.current.collections).toEqual(mockCollections);
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('list_chroma_collections');
    });

    it('should handle listCollections error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

      const { result } = renderHook(() => useChroma());

      await act(async () => {
        try {
          await result.current.listCollections();
        } catch (e) {
          // Expected error
        }
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe('Connection failed');
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe('Collection Management', () => {
    it('should create a new collection', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // Create collection
      mockInvoke.mockResolvedValueOnce([
        { name: 'default', count: 0 },
        { name: 'new-collection', count: 0 }
      ]); // Refresh after create

      const { result } = renderHook(() => useChroma());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.createCollection('new-collection');
      });

      expect(mockInvoke).toHaveBeenCalledWith('create_chroma_collection', {
        name: 'new-collection'
      });

      expect(result.current.collections).toHaveLength(2);
    });

    it('should delete a collection', async () => {
      const initialCollections = [
        { name: 'default', count: 10 },
        { name: 'to-delete', count: 5 }
      ];

      // First, set up the hook with initial collections
      mockInvoke.mockResolvedValueOnce(initialCollections); // For listCollections
      
      const { result } = renderHook(() => useChroma());
      
      // Load initial collections
      await act(async () => {
        await result.current.listCollections();
      });

      // Now delete
      mockInvoke.mockResolvedValueOnce(undefined); // Delete collection
      mockInvoke.mockResolvedValueOnce([
        { name: 'default', count: 10 }
      ]); // Refresh after delete

      await act(async () => {
        await result.current.deleteCollection('to-delete');
      });

      expect(mockInvoke).toHaveBeenCalledWith('delete_chroma_collection', {
        name: 'to-delete'
      });

      expect(result.current.collections).toHaveLength(1);
    });
  });

  describe('Document Management', () => {
    it('should add documents to collection', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // Add documents response

      const { result } = renderHook(() => useChroma());

      const documents = ['Document 1', 'Document 2'];
      const metadatas = [{ title: 'Doc 1' }, { title: 'Doc 2' }];
      const ids = ['doc1', 'doc2'];

      await act(async () => {
        await result.current.addDocuments('default', documents, metadatas, ids);
      });

      expect(mockInvoke).toHaveBeenCalledWith('add_documents_to_chroma', {
        collectionName: 'default',
        documents,
        metadatas,
        ids
      });
    });

    it('should query documents', async () => {
      const mockResults = [
        { id: 'doc1', text: 'Matched content', metadata: {}, distance: 0.1 }
      ];

      mockInvoke.mockResolvedValueOnce(mockResults);

      const { result } = renderHook(() => useChroma());

      let queryResults;
      await act(async () => {
        queryResults = await result.current.query('default', 'test query', 5);
      });

      expect(mockInvoke).toHaveBeenCalledWith('query_chroma', {
        collectionName: 'default',
        queryText: 'test query',
        nResults: 5,
        filter: undefined
      });

      expect(queryResults).toEqual(mockResults);
    });

    it('should get documents from collection', async () => {
      const mockDocuments = [
        { id: 'doc1', text: 'Document 1', metadata: { title: 'Doc 1' } },
        { id: 'doc2', text: 'Document 2', metadata: { title: 'Doc 2' } }
      ];

      mockInvoke.mockResolvedValueOnce(mockDocuments);

      const { result } = renderHook(() => useChroma());

      let documents;
      await act(async () => {
        documents = await result.current.getDocuments('default');
      });

      expect(mockInvoke).toHaveBeenCalledWith('get_documents_from_chroma', {
        collectionName: 'default',
        ids: undefined,
        filter: undefined
      });

      expect(documents).toEqual(mockDocuments);
    });

    it('should delete documents', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // Delete response

      const { result } = renderHook(() => useChroma());

      await act(async () => {
        await result.current.deleteDocuments('default', ['doc1', 'doc2']);
      });

      expect(mockInvoke).toHaveBeenCalledWith('delete_documents_from_chroma', {
        collectionName: 'default',
        ids: ['doc1', 'doc2']
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle errors with onError callback', async () => {
      const onError = vi.fn();
      const error = new Error('Operation failed');
      
      mockInvoke.mockRejectedValueOnce(error);

      const { result } = renderHook(() => useChroma({ onError }));

      await act(async () => {
        try {
          await result.current.createCollection('test');
        } catch (e) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith(error);
      expect(result.current.error).toBe(error);
    });

    it('should set loading state correctly', async () => {
      // Create a promise we can control
      let resolvePromise: (value: any) => void;
      const controlledPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      mockInvoke.mockReturnValueOnce(controlledPromise);

      const { result } = renderHook(() => useChroma());

      expect(result.current.isLoading).toBe(false);

      // Start an async operation
      act(() => {
        result.current.listCollections();
      });

      expect(result.current.isLoading).toBe(true);

      // Resolve the promise
      await act(async () => {
        resolvePromise!([]);
      });

      expect(result.current.isLoading).toBe(false);
    });
  });
});
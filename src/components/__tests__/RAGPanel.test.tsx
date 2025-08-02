import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RAGPanel } from '../RAGPanel';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core');

describe('RAGPanel', () => {
  const mockInvoke = vi.mocked(invoke);
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
    // Default mock implementations
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'list_chroma_collections') {
        return Promise.resolve(['default', 'test-collection']);
      }
      if (cmd === 'get_collection_count') {
        return Promise.resolve(5);
      }
      if (cmd === 'query_chroma') {
        return Promise.resolve([
          {
            id: 'doc1',
            document: 'Test document content',
            metadata: { title: 'Test Doc' },
            distance: 0.1
          }
        ]);
      }
      if (cmd === 'create_chroma_collection') {
        return Promise.resolve();
      }
      return Promise.resolve();
    });
  });

  describe('Rendering', () => {
    it('should render the RAG panel with basic elements', async () => {
      render(<RAGPanel />);
      
      expect(screen.getByText('Collection:')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('New collection name')).toBeInTheDocument();
      expect(screen.getByText('Create')).toBeInTheDocument();
      
      // Wait for collections to load
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('list_chroma_collections');
      });
    });

    it('should load collections on mount', async () => {
      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('list_chroma_collections');
      });
    });

    it('should display collection selector when collections are loaded', async () => {
      render(<RAGPanel />);
      
      await waitFor(() => {
        const select = screen.getByDisplayValue('default');
        expect(select).toBeInTheDocument();
      });
    });
  });

  describe('Collection Management', () => {
    it('should create a new collection', async () => {
      render(<RAGPanel />);
      
      const input = screen.getByPlaceholderText('New collection name');
      await user.type(input, 'new-collection');
      
      const createButton = screen.getByText('Create');
      await user.click(createButton);
      
      expect(mockInvoke).toHaveBeenCalledWith('create_chroma_collection', {
        collectionName: 'new-collection'
      });
    });

    it('should prevent creating collection with empty name', async () => {
      render(<RAGPanel />);
      
      const createButton = screen.getByText('Create');
      await user.click(createButton);
      
      expect(mockInvoke).not.toHaveBeenCalledWith('create_chroma_collection', expect.anything());
    });

    it('should change selected collection', async () => {
      render(<RAGPanel />);
      
      await waitFor(() => {
        const select = screen.getByDisplayValue('default');
        expect(select).toBeInTheDocument();
      });
      
      const select = screen.getByDisplayValue('default');
      fireEvent.change(select, { target: { value: 'test-collection' } });
      
      expect(select).toHaveValue('test-collection');
    });
  });

  describe('Document Search', () => {
    it('should perform search when query is entered', async () => {
      render(<RAGPanel />);
      
      // Wait for collections to load
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query');
      
      const searchButton = screen.getByText('Search');
      await user.click(searchButton);
      
      expect(mockInvoke).toHaveBeenCalledWith('query_chroma', {
        request: {
          collectionName: 'default',
          queryText: 'test query',
          nResults: 10,
          filter: null
        }
      });
    });

    it('should search on Enter key press', async () => {
      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query{enter}');
      
      expect(mockInvoke).toHaveBeenCalledWith('query_chroma', expect.any(Object));
    });

    it('should display search results', async () => {
      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query');
      
      const searchButton = screen.getByText('Search');
      await user.click(searchButton);
      
      await waitFor(() => {
        expect(screen.getByText('Test document content')).toBeInTheDocument();
      });
    });
  });

  describe('Document Upload', () => {
    it('should show upload form when upload button is clicked', async () => {
      render(<RAGPanel />);
      
      const uploadButton = screen.getByText('Add Document');
      await user.click(uploadButton);
      
      expect(screen.getByPlaceholderText('Document title (optional)')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Paste your document content here...')).toBeInTheDocument();
    });

    it('should handle text document upload', async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'add_documents_to_chroma') return Promise.resolve();
        if (cmd === 'list_chroma_collections') return Promise.resolve(['default']);
        if (cmd === 'get_collection_count') return Promise.resolve(5);
        return Promise.resolve();
      });

      render(<RAGPanel />);
      
      const uploadButton = screen.getByText('Add Document');
      await user.click(uploadButton);
      
      const titleInput = screen.getByPlaceholderText('Document title (optional)');
      const contentInput = screen.getByPlaceholderText('Paste your document content here...');
      
      await user.type(titleInput, 'Test Document');
      await user.type(contentInput, 'This is test content');
      
      const submitButton = screen.getByText('Upload Document');
      await user.click(submitButton);
      
      expect(mockInvoke).toHaveBeenCalledWith('add_documents_to_chroma', expect.any(Object));
    });
  });

  describe('Error Handling', () => {
    it('should display error when collection loading fails', async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'list_chroma_collections') {
          return Promise.reject(new Error('Connection failed'));
        }
        return Promise.resolve();
      });

      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch collections/)).toBeInTheDocument();
      });
    });

    it('should display error when search fails', async () => {
      render(<RAGPanel />);
      
      // Wait for initial collections to load
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      // Now set up the mock for search failure
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'query_chroma') return Promise.reject(new Error('Query failed'));
        return Promise.resolve();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query');
      
      const searchButton = screen.getByText('Search');
      await user.click(searchButton);
      
      await waitFor(() => {
        expect(screen.getByText(/Query failed/)).toBeInTheDocument();
      });
    });
  });

  describe('Integration', () => {
    it('should call onDocumentSelect when result is clicked', async () => {
      const onDocumentSelect = vi.fn();
      render(<RAGPanel onDocumentSelect={onDocumentSelect} />);
      
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query');
      
      const searchButton = screen.getByText('Search');
      await user.click(searchButton);
      
      await waitFor(() => {
        expect(screen.getByText('Test document content')).toBeInTheDocument();
      });
      
      // Click on the result container, not just the text
      const resultContainer = screen.getByText('Test document content').closest('.rag-result');
      await user.click(resultContainer!);
      
      expect(onDocumentSelect).toHaveBeenCalledWith({
        id: 'doc1',
        document: 'Test document content',
        metadata: { title: 'Test Doc' },
        distance: 0.1
      });
    });
  });

  describe('Loading States', () => {
    it('should show loading state during search', async () => {
      let resolveSearch: (value: any) => void;
      const searchPromise = new Promise(resolve => {
        resolveSearch = resolve;
      });

      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'list_chroma_collections') return Promise.resolve(['default']);
        if (cmd === 'get_collection_count') return Promise.resolve(5);
        if (cmd === 'query_chroma') return searchPromise;
        return Promise.resolve();
      });

      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
      
      const queryInput = screen.getByPlaceholderText('Search knowledge base...');
      await user.type(queryInput, 'test query');
      
      const searchButton = screen.getByText('Search');
      await user.click(searchButton);
      
      expect(screen.getByText('Searching')).toBeInTheDocument();
      
      // Resolve the search
      resolveSearch!([]);
      
      await waitFor(() => {
        expect(screen.queryByText('Searching')).not.toBeInTheDocument();
      });
    });
  });
});
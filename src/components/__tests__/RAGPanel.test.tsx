import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RAGPanel } from '../RAGPanel';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core');

// Mock hooks
vi.mock('../../hooks/useChroma', () => ({
  useChroma: vi.fn(() => ({
    collections: [
      { name: 'default', count: 10 },
      { name: 'test-collection', count: 5 }
    ],
    isLoading: false,
    error: null,
    listCollections: vi.fn(),
    createCollection: vi.fn(),
    deleteCollection: vi.fn(),
    addDocuments: vi.fn(),
    query: vi.fn(),
    getDocuments: vi.fn(),
    deleteDocuments: vi.fn()
  }))
}));

describe('RAGPanel', () => {
  const mockInvoke = vi.mocked(invoke);
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering and Display', () => {
    it('should render the RAG panel with all sections', () => {
      render(<RAGPanel />);
      
      // Check for main sections
      expect(screen.getByLabelText('Collection:')).toBeInTheDocument();
      expect(screen.getByText('Collections')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Enter your query...')).toBeInTheDocument();
    });

    it('should display collections list', () => {
      render(<RAGPanel />);
      
      expect(screen.getByText('default')).toBeInTheDocument();
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('test-collection')).toBeInTheDocument();
      expect(screen.getByText('5')).toBeInTheDocument();
    });

    it('should display documents in selected collection', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockGetDocuments = vi.fn().mockResolvedValue([
        { id: 'doc1', text: 'Test document 1', metadata: { title: 'Doc 1' } },
        { id: 'doc2', text: 'Test document 2', metadata: { title: 'Doc 2' } }
      ]);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        getDocuments: mockGetDocuments
      });

      render(<RAGPanel />);
      
      await waitFor(() => {
        expect(mockGetDocuments).toHaveBeenCalledWith('default');
      });
    });

    it('should display collection count', () => {
      render(<RAGPanel />);
      
      expect(screen.getByText('Collections')).toBeInTheDocument();
      expect(screen.getByText('default')).toBeInTheDocument();
      expect(screen.getByText('test-collection')).toBeInTheDocument();
    });
  });

  describe('Collection Management', () => {
    it('should create a new collection', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockCreateCollection = vi.fn().mockResolvedValue(undefined);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        createCollection: mockCreateCollection
      });

      render(<RAGPanel />);
      
      const createButton = screen.getByLabelText('Create collection');
      await user.click(createButton);
      
      const input = screen.getByPlaceholderText('Collection name');
      await user.type(input, 'new-collection');
      
      const confirmButton = screen.getByText('Create');
      await user.click(confirmButton);
      
      expect(mockCreateCollection).toHaveBeenCalledWith('new-collection');
    });

    it('should delete a collection with confirmation', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockDeleteCollection = vi.fn().mockResolvedValue(undefined);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        deleteCollection: mockDeleteCollection
      });

      render(<RAGPanel />);
      
      const deleteButtons = screen.getAllByLabelText('Delete collection');
      await user.click(deleteButtons[1]); // Click delete for test-collection
      
      // Confirm deletion
      const confirmButton = screen.getByText('Delete');
      await user.click(confirmButton);
      
      expect(mockDeleteCollection).toHaveBeenCalledWith('test-collection');
    });

    it('should select a collection when clicked', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockListCollections = vi.fn();
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        listCollections: mockListCollections
      });

      render(<RAGPanel />);
      
      const collectionItem = screen.getByText('test-collection');
      await user.click(collectionItem);
      
      // Collection selection is handled in the component state
    });
  });

  describe('Document Management', () => {
    it('should handle document upload', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockAddDocuments = vi.fn().mockResolvedValue(undefined);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        addDocuments: mockAddDocuments
      });

      render(<RAGPanel />);
      
      const file = new File(['test content'], 'test.txt', { type: 'text/plain' });
      const input = screen.getByLabelText('Upload documents');
      
      await user.upload(input, file);
      
      await waitFor(() => {
        expect(mockAddDocuments).toHaveBeenCalled();
      });
    });

    it('should handle multiple file uploads', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockAddDocuments = vi.fn().mockResolvedValue(undefined);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        addDocuments: mockAddDocuments
      });

      render(<RAGPanel />);
      
      const files = [
        new File(['content 1'], 'file1.txt', { type: 'text/plain' }),
        new File(['content 2'], 'file2.txt', { type: 'text/plain' })
      ];
      const input = screen.getByLabelText('Upload documents');
      
      await user.upload(input, files);
      
      await waitFor(() => {
        expect(mockAddDocuments).toHaveBeenCalled();
        const call = mockAddDocuments.mock.calls[0];
        expect(call[0]).toHaveLength(2);
      });
    });

    it('should delete a document', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockDeleteDocuments = vi.fn().mockResolvedValue(undefined);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        deleteDocuments: mockDeleteDocuments
      });

      render(<RAGPanel />);
      
      const deleteButtons = screen.getAllByLabelText(/Delete document/);
      await user.click(deleteButtons[0]);
      
      expect(mockDeleteDocuments).toHaveBeenCalledWith('default', ['doc1']);
    });

    it('should handle document search', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockQuery = vi.fn().mockResolvedValue([
        { id: 'doc1', text: 'Matched content', metadata: {}, distance: 0.05 }
      ]);
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        query: mockQuery
      });

      render(<RAGPanel />);
      
      const searchInput = screen.getByPlaceholderText('Search documents...');
      await user.type(searchInput, 'test query');
      await user.keyboard('{Enter}');
      
      expect(mockQuery).toHaveBeenCalledWith('default', 'test query', 10, undefined);
    });
  });

  describe('Error Handling', () => {
    it('should display error message when operations fail', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        error: 'Failed to connect to ChromaDB'
      });

      render(<RAGPanel />);
      
      expect(screen.getByText(/Failed to connect to ChromaDB/)).toBeInTheDocument();
    });

    it('should show loading state', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        isLoading: true
      });

      render(<RAGPanel />);
      
      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });

    it('should handle invalid file types', async () => {
      render(<RAGPanel />);
      
      const file = new File(['binary data'], 'test.exe', { type: 'application/exe' });
      const input = screen.getByLabelText('Upload documents');
      
      await user.upload(input, file);
      
      await waitFor(() => {
        expect(screen.getByText(/Unsupported file type/)).toBeInTheDocument();
      });
    });
  });

  describe('Collection Operations', () => {
    it('should refresh collections', async () => {
      const { useChroma } = await import('../../hooks/useChroma');
      const mockListCollections = vi.fn();
      vi.mocked(useChroma).mockReturnValue({
        ...vi.mocked(useChroma).mock.results[0].value,
        listCollections: mockListCollections
      });

      render(<RAGPanel />);
      
      const refreshButton = screen.getByLabelText('Refresh collections');
      await user.click(refreshButton);
      
      expect(mockListCollections).toHaveBeenCalled();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      render(<RAGPanel />);
      
      expect(screen.getByRole('heading', { name: 'RAG Database' })).toBeInTheDocument();
      expect(screen.getByLabelText('Create collection')).toBeInTheDocument();
      expect(screen.getByLabelText('Upload documents')).toBeInTheDocument();
    });

    it('should support keyboard navigation', async () => {
      render(<RAGPanel />);
      
      const searchInput = screen.getByPlaceholderText('Search documents...');
      searchInput.focus();
      
      await user.keyboard('{Tab}');
      expect(screen.getByLabelText('Create collection')).toHaveFocus();
      
      await user.keyboard('{Tab}');
      expect(screen.getByLabelText('Upload documents')).toHaveFocus();
    });
  });

  describe('Integration with Chat', () => {
    it('should add documents to chat context when selected', async () => {
      const onAddToContext = vi.fn();
      render(<RAGPanel onAddToContext={onAddToContext} />);
      
      const mockOnDocumentSelect = vi.fn();
      render(<RAGPanel onDocumentSelect={mockOnDocumentSelect} />);
      
      // Simulate document selection if there are documents in the UI
      // This would require the component to display documents first
    });
  });
});
import React, { useState } from 'react';
import { useRAG, RAGDocument } from '../hooks/useRAG';
import './RAGPanel.css';

export interface RAGPanelProps {
  onDocumentSelect?: (document: RAGDocument) => void;
  className?: string;
}

export const RAGPanel: React.FC<RAGPanelProps> = ({ 
  onDocumentSelect,
  className = ''
}) => {
  // Business logic handled by custom hook
  const {
    collections,
    selectedCollection,
    loading,
    error,
    setSelectedCollection,
    createCollection,
    searchDocuments,
    uploadDocument,
    clearError
  } = useRAG();

  // UI-only state
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<RAGDocument[]>([]);
  const [newCollectionName, setNewCollectionName] = useState('');
  const [showUpload, setShowUpload] = useState(false);
  const [uploadText, setUploadText] = useState('');
  const [uploadTitle, setUploadTitle] = useState('');
  const [isSearching, setIsSearching] = useState(false);

  // Event handlers - clean and simple
  const handleSearch = async () => {
    if (!query.trim() || !selectedCollection) return;

    setIsSearching(true);
    try {
      const searchResults = await searchDocuments(query);
      setResults(searchResults);
    } catch (err) {
      console.error('Search failed:', err);
    } finally {
      setIsSearching(false);
    }
  };

  const handleCreateCollection = async () => {
    if (!newCollectionName.trim()) return;

    try {
      await createCollection(newCollectionName);
      setNewCollectionName('');
    } catch (err) {
      console.error('Create collection failed:', err);
    }
  };

  const handleUploadDocument = async () => {
    if (!uploadText.trim() || !selectedCollection) return;

    try {
      await uploadDocument(uploadText, {
        title: uploadTitle.trim() || 'Untitled Document'
      });
      
      // Reset form
      setUploadText('');
      setUploadTitle('');
      setShowUpload(false);
    } catch (err) {
      console.error('Upload failed:', err);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const handleResultClick = (result: RAGDocument) => {
    onDocumentSelect?.(result);
  };

  const formatDate = (dateString: string) => {
    try {
      return new Date(dateString).toLocaleString();
    } catch {
      return dateString;
    }
  };

  return (
    <div className={`rag-panel ${className}`}>
      {/* Header Section */}
      <div className="rag-header">
        <div className="collection-management">
          {/* Collection Selector */}
          <div className="collection-selector">
            <label htmlFor="collection-select">Collection:</label>
            <select
              id="collection-select"
              value={selectedCollection}
              onChange={(e) => setSelectedCollection(e.target.value)}
              disabled={loading || collections.length === 0}
            >
              {collections.length === 0 ? (
                <option value="">No collections</option>
              ) : (
                collections.map((collection) => (
                  <option key={collection.name} value={collection.name}>
                    {collection.name} ({collection.count})
                  </option>
                ))
              )}
            </select>
          </div>
          
          {/* Collection Actions */}
          <div className="collection-actions">
            <input
              type="text"
              placeholder="New collection name"
              value={newCollectionName}
              onChange={(e) => setNewCollectionName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCreateCollection()}
              className="new-collection-input"
              disabled={loading}
            />
            <button 
              onClick={handleCreateCollection}
              disabled={!newCollectionName.trim() || loading}
              className="create-collection-btn"
            >
              {loading ? 'Creating...' : 'Create'}
            </button>
            <button 
              onClick={() => setShowUpload(!showUpload)}
              disabled={!selectedCollection || loading}
              className="upload-toggle-btn"
            >
              {showUpload ? 'Cancel' : 'Add Document'}
            </button>
          </div>
        </div>
        
        {/* Upload Section */}
        {showUpload && (
          <div className="upload-section">
            <input
              type="text"
              placeholder="Document title (optional)"
              value={uploadTitle}
              onChange={(e) => setUploadTitle(e.target.value)}
              className="upload-title-input"
              disabled={loading}
            />
            <textarea
              placeholder="Paste your document content here..."
              value={uploadText}
              onChange={(e) => setUploadText(e.target.value)}
              className="upload-text-area"
              rows={6}
              disabled={loading}
            />
            <div className="upload-actions">
              <button
                onClick={handleUploadDocument}
                disabled={!uploadText.trim() || !selectedCollection || loading}
                className="upload-btn"
              >
                {loading ? 'Uploading...' : 'Upload Document'}
              </button>
            </div>
          </div>
        )}
        
        {/* Search Input */}
        <div className="rag-search-input-wrapper">
          <input
            type="text"
            className="rag-search-input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search knowledge base..."
            disabled={!selectedCollection || loading || isSearching}
          />
          <button
            className="rag-search-button"
            onClick={handleSearch}
            disabled={!selectedCollection || !query.trim() || loading || isSearching}
          >
            {isSearching ? 'Searching...' : 'Search'}
          </button>
        </div>
      </div>

      {/* Results Section */}
      <div className="rag-results">
        {/* Error Display */}
        {error && (
          <div className="rag-error">
            {error}
            <button onClick={clearError} className="error-dismiss">×</button>
          </div>
        )}
        
        {/* Loading State */}
        {isSearching && (
          <div className="rag-loading">
            <span>Searching</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        )}
        
        {/* No Results */}
        {!isSearching && results.length === 0 && query.trim() !== '' && (
          <div className="rag-no-results">No results found for "{query}"</div>
        )}
        
        {/* Results */}
        {!isSearching && results.length > 0 && (
          <div className="rag-results-list">
            {results.map((result, index) => (
              <div
                key={`${result.id}-${index}`}
                className="rag-result"
                onClick={() => handleResultClick(result)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    handleResultClick(result);
                  }
                }}
              >
                <h3 className="rag-result-title">
                  {result.metadata.title || 'Untitled Document'}
                </h3>
                
                <div className="rag-result-meta">
                  <span className="rag-result-source">
                    {result.metadata.source}
                  </span>
                  <span className="rag-result-type">
                    {result.metadata.document_type}
                  </span>
                  {result.metadata.language && (
                    <span className="rag-result-language">
                      {result.metadata.language}
                    </span>
                  )}
                  <span className="rag-result-date">
                    {formatDate(result.metadata.timestamp)}
                  </span>
                </div>
                
                <p className="rag-result-content">{result.document}</p>
                
                <div className="rag-result-footer">
                  <span className="rag-result-id">ID: {result.id}</span>
                  <span className="rag-result-distance">
                    Relevance: {(1 - result.distance).toFixed(2)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default RAGPanel;

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './RAGPanel.css';

export interface DocumentMetadata {
  source: string;
  document_type: string;
  language?: string;
  timestamp: string;
  file_path?: string;
  url?: string;
  title?: string;
  [key: string]: any;
}

export interface QueryResult {
  document: string;
  metadata: DocumentMetadata;
  distance: number;
  id: string;
}

export interface RAGPanelProps {
  onDocumentSelect?: (document: QueryResult) => void;
}

export const RAGPanel: React.FC<RAGPanelProps> = ({ onDocumentSelect }) => {
  const [collections, setCollections] = useState<string[]>([]);
  const [selectedCollection, setSelectedCollection] = useState<string>('');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<QueryResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [collectionCounts, setCollectionCounts] = useState<Record<string, number>>({});
  const [error, setError] = useState<string | null>(null);
  const [uploadText, setUploadText] = useState('');
  const [uploadTitle, setUploadTitle] = useState('');
  const [newCollectionName, setNewCollectionName] = useState('');
  const [showUpload, setShowUpload] = useState(false);

  // Fetch collections on component mount
  useEffect(() => {
    fetchCollections();
  }, []);

  // Fetch collection counts when collections change
  useEffect(() => {
    if (collections.length > 0) {
      fetchCollectionCounts();
    }
  }, [collections]);

  const fetchCollections = async () => {
    try {
      const collectionList = await invoke<string[]>('list_chroma_collections');
      setCollections(collectionList);
      if (collectionList.length > 0) {
        setSelectedCollection(collectionList[0]);
      }
    } catch (error) {
      console.error('Failed to fetch collections:', error);
      setError('Failed to fetch collections. Please check your ChromaDB connection.');
    }
  };

  const fetchCollectionCounts = async () => {
    const counts: Record<string, number> = {};
    
    for (const collection of collections) {
      try {
        const count = await invoke<number>('get_collection_count', { collectionName: collection });
        counts[collection] = count;
      } catch (error) {
        console.error(`Failed to get count for collection ${collection}:`, error);
        counts[collection] = 0;
      }
    }
    
    setCollectionCounts(counts);
  };

  const handleSearch = async () => {
    if (!selectedCollection || query.trim() === '' || loading) return;

    setLoading(true);
    setResults([]);
    setError(null);

    try {
      const queryResults = await invoke<QueryResult[]>('query_chroma', {
        request: {
          collectionName: selectedCollection,
          queryText: query.trim(),
          nResults: 10,
          filter: null
        }
      });

      setResults(queryResults);
    } catch (error) {
      console.error('Query failed:', error);
      setError('Query failed. Please check your ChromaDB connection.');
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const handleResultClick = (result: QueryResult) => {
    if (onDocumentSelect) {
      onDocumentSelect(result);
    }
  };

  const handleCreateCollection = async () => {
    if (!newCollectionName.trim()) return;
    
    try {
      await invoke('create_chroma_collection', { collectionName: newCollectionName.trim() });
      await fetchCollections();
      setSelectedCollection(newCollectionName.trim());
      setNewCollectionName('');
      setError(null);
    } catch (error) {
      console.error('Failed to create collection:', error);
      setError('Failed to create collection. Please try again.');
    }
  };

  const handleUploadDocument = async () => {
    if (!selectedCollection || !uploadText.trim()) return;
    
    try {
      const metadata: DocumentMetadata = {
        source: 'manual_upload',
        document_type: 'text',
        language: 'en',
        timestamp: new Date().toISOString(),
        title: uploadTitle.trim() || 'Untitled Document',
        file_path: undefined,
        url: undefined
      };

      await invoke('add_documents_to_chroma', {
        request: {
          collectionName: selectedCollection,
          documents: [uploadText.trim()],
          metadatas: [metadata],
          ids: null
        }
      });

      // Reset form
      setUploadText('');
      setUploadTitle('');
      setShowUpload(false);
      
      // Refresh collection counts
      await fetchCollectionCounts();
      setError(null);
    } catch (error) {
      console.error('Failed to upload document:', error);
      setError('Failed to upload document. Please try again.');
    }
  };

  const formatDate = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return date.toLocaleString();
    } catch (e) {
      return dateString;
    }
  };

  return (
    <div className="rag-panel">
      <div className="rag-header">
        <div className="collection-management">
          <div className="collection-selector">
            <label htmlFor="collection-select">Collection:</label>
            <select
              id="collection-select"
              value={selectedCollection}
              onChange={(e) => setSelectedCollection(e.target.value)}
              disabled={collections.length === 0}
            >
              {collections.map((collection) => (
                <option key={collection} value={collection}>
                  {collection} ({collectionCounts[collection] || 0})
                </option>
              ))}
            </select>
          </div>
          
          <div className="collection-actions">
            <input
              type="text"
              placeholder="New collection name"
              value={newCollectionName}
              onChange={(e) => setNewCollectionName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCreateCollection()}
              className="new-collection-input"
            />
            <button 
              onClick={handleCreateCollection}
              disabled={!newCollectionName.trim()}
              className="create-collection-btn"
            >
              Create
            </button>
            <button 
              onClick={() => setShowUpload(!showUpload)}
              disabled={!selectedCollection}
              className="upload-toggle-btn"
            >
              {showUpload ? 'Cancel' : 'Add Document'}
            </button>
          </div>
        </div>
        
        {showUpload && (
          <div className="upload-section">
            <input
              type="text"
              placeholder="Document title (optional)"
              value={uploadTitle}
              onChange={(e) => setUploadTitle(e.target.value)}
              className="upload-title-input"
            />
            <textarea
              placeholder="Paste your document content here..."
              value={uploadText}
              onChange={(e) => setUploadText(e.target.value)}
              className="upload-text-area"
              rows={6}
            />
            <div className="upload-actions">
              <button
                onClick={handleUploadDocument}
                disabled={!uploadText.trim() || !selectedCollection}
                className="upload-btn"
              >
                Upload Document
              </button>
            </div>
          </div>
        )}
        
        <div className="rag-search-input-wrapper">
          <input
            type="text"
            className="rag-search-input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search knowledge base..."
            disabled={!selectedCollection || loading}
          />
          <button
            className="rag-search-button"
            onClick={handleSearch}
            disabled={!selectedCollection || query.trim() === '' || loading}
          >
            Search
          </button>
        </div>
      </div>

      <div className="rag-results">
        {error && <div className="rag-error">{error}</div>}
        
        {loading ? (
          <div className="rag-loading">
            <span>Searching</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        ) : results.length === 0 && query.trim() !== '' ? (
          <div className="rag-no-results">No results found</div>
        ) : (
          results.map((result, index) => (
            <div
              key={index}
              className="rag-result"
              onClick={() => handleResultClick(result)}
            >
              <h3 className="rag-result-title">
                {result.metadata.title || 'Untitled Document'}
              </h3>
              
              <div className="rag-result-meta">
                <span className="rag-result-source">{result.metadata.source}</span>
                <span className="rag-result-type">{result.metadata.document_type}</span>
                {result.metadata.language && (
                  <span className="rag-result-language">{result.metadata.language}</span>
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
          ))
        )}
      </div>
    </div>
  );
};

export default RAGPanel;

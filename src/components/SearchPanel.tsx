import React from 'react';
import { useSearch, SearchResult } from '../hooks/useSearch';
import SearchHealthIndicator from './SearchHealthIndicator';
import { SearchHealthStatus } from '../hooks/useSearchHealth';
import './SearchPanel.css';

export interface SearchPanelProps {
  onResultSelect?: (result: SearchResult) => void;
  className?: string;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({ 
  onResultSelect,
  className = ''
}) => {
  // Business logic handled by custom hook
  const {
    query,
    results,
    isLoading,
    error,
    availableEngines,
    activeEngines,
    isHealthy,
    setQuery,
    search,
    toggleEngine,
    updateHealthStatus,
    clearError
  } = useSearch();

  // Event handlers - clean and simple
  const handleSearch = async () => {
    await search();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const handleResultClick = (result: SearchResult) => {
    onResultSelect?.(result);
  };

  const handleHealthChange = (status: SearchHealthStatus) => {
    updateHealthStatus(status.isHealthy);
  };

  return (
    <div className={`search-panel ${className}`}>
      {/* Header Section */}
      <div className="search-header">
        <div className="search-input-wrapper">
          <span className="search-icon">🔍</span>
          <input
            type="text"
            className="search-input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search documentation, Stack Overflow, GitHub..."
            disabled={isLoading || !isHealthy}
          />
        </div>
        <SearchHealthIndicator 
          showDetails={false}
          onHealthChange={handleHealthChange}
          className="search-health-status"
        />
      </div>

      {/* Engine Filters */}
      <div className="search-filters">
        {availableEngines.map((engine) => (
          <button
            key={engine}
            className={`filter-chip ${activeEngines.includes(engine) ? 'active' : ''}`}
            onClick={() => toggleEngine(engine)}
            disabled={isLoading}
          >
            {engine}
          </button>
        ))}
      </div>

      {/* Results Section */}
      <div className="search-results">
        {/* Error Display */}
        {error && (
          <div className="search-error">
            {error}
            <button onClick={clearError} className="error-dismiss">×</button>
          </div>
        )}

        {/* Loading State */}
        {isLoading ? (
          <div className="loading-indicator">
            <span>Searching</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        ) : (results || []).length === 0 && query.trim() !== '' ? (
          <div className="no-results">No results found for "{query}"</div>
        ) : (
          /* Results */
          (results || []).map((result, index) => (
            <div
              key={`${result.url}-${index}`}
              className="search-result"
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
              <h3 className="result-title">{result.title}</h3>
              <div className="result-url">{result.url}</div>
              <p className="result-snippet">{result.content}</p>
              <div className="result-meta">
                <span className="result-engine">{result.engine}</span>
                {result.score !== undefined && (
                  <span className="result-score">Score: {result.score.toFixed(2)}</span>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default SearchPanel;

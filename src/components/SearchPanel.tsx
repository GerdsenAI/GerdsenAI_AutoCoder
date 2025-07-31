import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SearchHealthIndicator from './SearchHealthIndicator';
import { SearchHealthStatus } from '../hooks/useSearchHealth';
import './SearchPanel.css';

interface SearchResult {
  title: string;
  url: string;
  content: string;
  engine: string;
  score?: number;
}

interface SearchPanelProps {
  onResultSelect?: (result: SearchResult) => void;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({ onResultSelect }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [activeEngines, setActiveEngines] = useState<string[]>(['github', 'stackoverflow', 'google']);
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
  const [searchServiceHealthy, setSearchServiceHealthy] = useState<boolean>(true);

  // Fetch available engines on component mount
  useEffect(() => {
    const fetchEngines = async () => {
      try {
        const engines = await invoke<string[]>('get_available_engines');
        if (engines && engines.length > 0) {
          setAvailableEngines(engines);
          setActiveEngines(engines.slice(0, 3)); // Default to first 3 engines
        }
      } catch (error) {
        console.error('Failed to fetch available engines:', error);
      }
    };

    fetchEngines();
  }, []);

  const handleHealthChange = (status: SearchHealthStatus) => {
    setSearchServiceHealthy(status.isHealthy);
  };

  const handleSearch = async () => {
    if (query.trim() === '' || loading) return;

    // Check if search service is healthy before attempting search
    if (!searchServiceHealthy) {
      console.warn('Search service is not healthy, attempting search anyway...');
    }

    setLoading(true);
    setResults([]);

    try {
      const searchResults = await invoke<SearchResult[]>('search_web', {
        query: query.trim(),
        engines: activeEngines,
        limit: 10
      });

      setResults(searchResults);
    } catch (error) {
      console.error('Search failed:', error);
      // Show user-friendly error message when service is unavailable
      if (!searchServiceHealthy) {
        alert('Search service is currently unavailable. Please check the service status or try again later.');
      }
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const toggleEngine = (engine: string) => {
    setActiveEngines(prev => 
      prev.includes(engine)
        ? prev.filter(e => e !== engine)
        : [...prev, engine]
    );
  };

  const handleResultClick = (result: SearchResult) => {
    if (onResultSelect) {
      onResultSelect(result);
    }
  };

  return (
    <div className="search-panel">
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
            disabled={loading || !searchServiceHealthy}
          />
        </div>
        <SearchHealthIndicator 
          showDetails={false}
          onHealthChange={handleHealthChange}
          className="search-health-status"
        />
      </div>

      <div className="search-filters">
        {availableEngines.map((engine) => (
          <button
            key={engine}
            className={`filter-chip ${activeEngines.includes(engine) ? 'active' : ''}`}
            onClick={() => toggleEngine(engine)}
          >
            {engine}
          </button>
        ))}
      </div>

      <div className="search-results">
        {loading ? (
          <div className="loading-indicator">
            <span>Searching</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        ) : results.length === 0 && query.trim() !== '' ? (
          <div className="no-results">No results found</div>
        ) : (
          results.map((result, index) => (
            <div
              key={index}
              className="search-result"
              onClick={() => handleResultClick(result)}
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

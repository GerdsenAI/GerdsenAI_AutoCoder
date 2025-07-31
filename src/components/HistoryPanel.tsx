import React, { useState, useEffect, useCallback, useMemo } from 'react';
import './HistoryPanel.css';
import { ChatSession } from '../types';

// Utility hook for debouncing
const useDebounce = (value: string, delay: number) => {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
};

export interface HistoryPanelProps {
  sessions: ChatSession[];
  onSelectSession: (session: ChatSession | null) => void;
  onDeleteSession: (sessionId: string) => Promise<void>;
  onCreateNewSession: () => Promise<void>;
}

// --- Constants ---
const SEARCH_DEBOUNCE_DELAY = 300;

export const HistoryPanel: React.FC<HistoryPanelProps> = ({ 
  sessions, 
  onSelectSession, 
  onDeleteSession, 
  onCreateNewSession 
}) => {
  const [loading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const debouncedSearchQuery = useDebounce(searchQuery, SEARCH_DEBOUNCE_DELAY);
  const [filterTag, setFilterTag] = useState<string | null>(null);
  const [deletingSessionId, setDeletingSessionId] = useState<string | null>(null);
  const [filteredSessions, setFilteredSessions] = useState<ChatSession[]>(sessions);

  // Update filtered sessions when sessions change
  useEffect(() => {
    setFilteredSessions(sessions);
  }, [sessions]);

  // --- Search and Filter Logic ---
  useEffect(() => {
    const filterSessions = () => {
      if (!debouncedSearchQuery.trim() && !filterTag) {
        setFilteredSessions(sessions);
        return;
      }

      let filtered = [...sessions];

      // Apply search query filter
      if (debouncedSearchQuery.trim()) {
        const query = debouncedSearchQuery.toLowerCase();
        filtered = filtered.filter(session => 
          session.title.toLowerCase().includes(query) || 
          session.messages.some(msg => msg.content.toLowerCase().includes(query))
        );
      }

      // Apply tag filter
      if (filterTag) {
        filtered = filtered.filter(session => 
          session.tags.includes(filterTag)
        );
      }

      setFilteredSessions(filtered);
    };

    filterSessions();
  }, [debouncedSearchQuery, filterTag, sessions]);

  const handleFilterByTag = useCallback((tag: string | null) => {
    setFilterTag(tag);
  }, []);

  // --- Session Actions ---
  const handleDeleteSession = useCallback(async (id: string, event: React.MouseEvent) => {
    event.stopPropagation();
    if (!window.confirm('Are you sure you want to delete this session?')) {
      return;
    }

    setDeletingSessionId(id);
    try {
      await onDeleteSession(id);
    } catch (err: any) {
      console.error('Failed to delete session:', err);
      setError(`Failed to delete session: ${err.message || err}`);
    } finally {
      setDeletingSessionId(null);
    }
  }, [onDeleteSession]);

  // --- Utility Functions ---
  const allTags = useMemo(() => {
    const tags = new Set<string>();
    sessions.forEach(session => {
      session.tags.forEach(tag => tags.add(tag));
    });
    return Array.from(tags).sort();
  }, [sessions]);

  const formatDate = useCallback((dateString: string) => {
    try {
      const date = new Date(dateString);
      const now = new Date();
      const yesterday = new Date(now);
      yesterday.setDate(now.getDate() - 1);

      if (date.toDateString() === now.toDateString()) {
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      } else if (date.toDateString() === yesterday.toDateString()) {
        return 'Yesterday';
      } else if (date.getFullYear() === now.getFullYear()) {
        return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
      } else {
        return date.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
      }
    } catch (e) {
      return dateString;
    }
  }, []);

  const getSessionPreview = useCallback((session: ChatSession) => {
    // Find the last user message for preview
    const lastUserMessage = session.messages.slice().reverse().find(msg => msg.role === 'user');
    if (!lastUserMessage) return 'No user messages';
    
    const maxLength = 60;
    const content = lastUserMessage.content.length > maxLength
      ? `${lastUserMessage.content.substring(0, maxLength)}...`
      : lastUserMessage.content;
      
    return content;
  }, []);

  return (
    <div className="history-panel" role="region" aria-label="Chat History">
      <div className="history-header">
        <div className="search-input-wrapper">
          <input
            type="text"
            className="search-input"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search chat history..."
            aria-label="Search chat history"
          />
          <button className="search-button" aria-label="Search">
            🔍
          </button>
        </div>
        
        <button 
          className="new-chat-button"
          onClick={onCreateNewSession}
          aria-label="Create new chat session"
          disabled={loading} // Disable if loading
        >
          + New Chat
        </button>
      </div>

      {error && (
        <div className="history-error-banner" role="alert">
          <span>{error}</span>
          <button onClick={() => setError(null)} aria-label="Dismiss error">×</button>
        </div>
      )}

      {allTags.length > 0 && (
        <div className="tag-filters" role="group" aria-label="Filter by tag">
          <button
            className={`tag-filter ${filterTag === null ? 'active' : ''}`}
            onClick={() => handleFilterByTag(null)}
            aria-pressed={filterTag === null}
          >
            All
          </button>
          {allTags.map(tag => (
            <button
              key={tag}
              className={`tag-filter ${filterTag === tag ? 'active' : ''}`}
              onClick={() => handleFilterByTag(tag)}
              aria-pressed={filterTag === tag}
            >
              {tag}
            </button>
          ))}
        </div>
      )}

      <div className="sessions-list" role="list" aria-label="Chat Sessions">
        {loading ? (
          <div className="loading-indicator" role="status">Loading sessions...</div>
        ) : filteredSessions.length === 0 ? (
          <div className="no-sessions" role="note">
            {searchQuery || filterTag
              ? 'No matching sessions found' 
              : 'No chat sessions yet. Start a new chat!'}
          </div>
        ) : (
          filteredSessions.map((session) => (
            <div
              key={session.id}
              className={`session-item ${deletingSessionId === session.id ? 'deleting' : ''}`}
              onClick={() => onSelectSession(session)}
              role="listitem"
              tabIndex={0}
              aria-label={`Chat session: ${session.title}`}
            >
              <div className="session-header">
                <h3 className="session-title">{session.title}</h3>
                <span className="session-date">{formatDate(session.updated_at)}</span>
              </div>
              
              <p className="session-preview">{getSessionPreview(session)}</p>
              
              <div className="session-meta">
                <span className="session-message-count" aria-label={`${session.messages.length} messages`}>
                  {session.messages.length} messages
                </span>
                <span className="session-model">{session.model}</span>
                <div className="session-tags">
                  {session.tags.map(tag => (
                    <span key={tag} className="session-tag">{tag}</span>
                  ))}
                </div>
                <button
                  className="delete-button"
                  onClick={(e) => handleDeleteSession(session.id, e)}
                  title="Delete session"
                  aria-label="Delete session"
                  disabled={deletingSessionId === session.id}
                >
                  {deletingSessionId === session.id ? 'Deleting...' : '🗑️'}
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default HistoryPanel;

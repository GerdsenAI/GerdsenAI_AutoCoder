import { useState, useCallback } from 'react';
import './ContextFileList.css';

export interface ContextFile {
  path: string;
  token_count: number;
  relevance_score: number;
  is_pinned: boolean;
  file_type: string;
}

export interface BuiltContext {
  files: ContextFile[];
  total_tokens: number;
  budget: {
    total: number;
    used: number;
    available: number;
    breakdown: {
      conversation: number;
      rag_documents: number;
      pinned_files: number;
      suggested_files: number;
      reserved: number;
    };
  };
}

interface ContextFileListProps {
  includedFiles: ContextFile[];
  suggestedFiles: ContextFile[];
  availableTokens: number;
  onFilePin: (filePath: string) => void;
  onFileUnpin: (filePath: string) => void;
  onFileInclude: (filePath: string) => void;
  onFileRemove: (filePath: string) => void;
  className?: string;
}

export const ContextFileList = ({
  includedFiles,
  suggestedFiles,
  availableTokens,
  onFilePin,
  onFileUnpin,
  onFileInclude,
  onFileRemove,
  className = ''
}: ContextFileListProps) => {
  const [loadingActions, setLoadingActions] = useState<Set<string>>(new Set());

  const formatTokens = useCallback((tokens: number): string => {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
  }, []);

  const getFileIcon = useCallback((fileType: string): string => {
    const iconMap: Record<string, string> = {
      'tsx': 'TS',
      'ts': 'TS',
      'jsx': 'JS',
      'js': 'JS',
      'rs': 'RS',
      'py': 'PY',
      'json': '{}',
      'md': 'MD',
      'css': 'CS',
      'html': 'HT',
      'yml': 'YM',
      'yaml': 'YM',
      'toml': 'TM',
      'txt': 'TX',
      'unknown': '?'
    };
    return iconMap[fileType.toLowerCase()] || '?';
  }, []);

  const getFileName = useCallback((path: string): string => {
    return path.split('/').pop() || path;
  }, []);

  const handleAction = useCallback(async (action: () => void, filePath: string) => {
    setLoadingActions(prev => new Set(prev).add(filePath));
    try {
      action();
    } finally {
      setLoadingActions(prev => {
        const next = new Set(prev);
        next.delete(filePath);
        return next;
      });
    }
  }, []);

  const handlePin = useCallback((filePath: string) => {
    handleAction(() => onFilePin(filePath), filePath);
  }, [onFilePin, handleAction]);

  const handleUnpin = useCallback((filePath: string) => {
    handleAction(() => onFileUnpin(filePath), filePath);
  }, [onFileUnpin, handleAction]);

  const handleInclude = useCallback((filePath: string) => {
    handleAction(() => onFileInclude(filePath), filePath);
  }, [onFileInclude, handleAction]);

  const handleRemove = useCallback((filePath: string) => {
    handleAction(() => onFileRemove(filePath), filePath);
  }, [onFileRemove, handleAction]);

  const includedTokens = includedFiles.reduce((sum, file) => sum + file.token_count, 0);

  return (
    <div className={`context-file-list ${className}`}>
      <div className="context-grid">
        {/* Included Files Section */}
        <div className="context-section">
          <div className="section-header">
            <div className="section-title">
              <div className="section-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                  <path d="M14 2v6h6"/>
                  <path d="M16 13H8"/>
                  <path d="M16 17H8"/>
                </svg>
              </div>
              <span>Included Context</span>
            </div>
            <div className="token-count">{formatTokens(includedTokens)} tokens</div>
          </div>
          
          <div className="file-list">
            {includedFiles.length === 0 ? (
              <div className="empty-state">
                <span>No files included yet</span>
                <p>Pin files or let AI suggest relevant context</p>
              </div>
            ) : (
              includedFiles.map((file) => (
                <div key={file.path} className="file-item">
                  <div className="file-info">
                    <div className="file-icon" data-type={file.file_type}>
                      {getFileIcon(file.file_type)}
                    </div>
                    <div className="file-details">
                      <div className="file-name" title={file.path}>
                        {getFileName(file.path)}
                      </div>
                      <div className="file-meta">
                        <span>{formatTokens(file.token_count)} tokens</span>
                        <div className="relevance-score">
                          <span>Relevance:</span>
                          <div className="relevance-bar">
                            <div 
                              className="relevance-fill" 
                              style={{ width: `${file.relevance_score * 100}%` }}
                            />
                          </div>
                          <span>{Math.round(file.relevance_score * 100)}%</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="file-actions">
                    <button
                      className={`action-btn ${file.is_pinned ? 'pinned' : ''}`}
                      title={file.is_pinned ? 'Unpin' : 'Pin'}
                      onClick={() => file.is_pinned ? handleUnpin(file.path) : handlePin(file.path)}
                      disabled={loadingActions.has(file.path)}
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M12 17l-5-5h3V4h4v8h3l-5 5z"/>
                      </svg>
                    </button>
                    <button
                      className="action-btn"
                      title="Remove"
                      onClick={() => handleRemove(file.path)}
                      disabled={loadingActions.has(file.path)}
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M18 6L6 18"/>
                        <path d="M6 6l12 12"/>
                      </svg>
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Suggested Files Section */}
        <div className="context-section">
          <div className="section-header">
            <div className="section-title">
              <div className="section-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"/>
                </svg>
              </div>
              <span>AI Suggested</span>
            </div>
            <div className="token-count">{formatTokens(availableTokens)} tokens available</div>
          </div>
          
          <div className="file-list">
            {suggestedFiles.length === 0 ? (
              <div className="empty-state">
                <span>No suggestions yet</span>
                <p>Start a conversation to get AI-suggested context</p>
              </div>
            ) : (
              suggestedFiles.map((file) => (
                <div key={file.path} className="file-item suggested">
                  <div className="file-info">
                    <div className="file-icon" data-type={file.file_type}>
                      {getFileIcon(file.file_type)}
                    </div>
                    <div className="file-details">
                      <div className="file-name" title={file.path}>
                        {getFileName(file.path)}
                      </div>
                      <div className="file-meta">
                        <span>{formatTokens(file.token_count)} tokens</span>
                        <div className="relevance-score">
                          <span>Relevance:</span>
                          <div className="relevance-bar">
                            <div 
                              className="relevance-fill" 
                              style={{ width: `${file.relevance_score * 100}%` }}
                            />
                          </div>
                          <span>{Math.round(file.relevance_score * 100)}%</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="file-actions">
                    <button
                      className="action-btn include"
                      title="Include"
                      onClick={() => handleInclude(file.path)}
                      disabled={loadingActions.has(file.path)}
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M12 5v14"/>
                        <path d="M5 12h14"/>
                      </svg>
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ContextFileList;

import React, { useState } from 'react';
import './UserErrorDisplay.css';

export interface UserError {
  title: string;
  message: string;
  suggestion?: string;
  help_link?: string;
  error_code: string;
  technical_details?: string;
}

interface UserErrorDisplayProps {
  error: UserError | string;
  onRetry?: () => void;
  onDismiss?: () => void;
  className?: string;
}

export const UserErrorDisplay: React.FC<UserErrorDisplayProps> = ({
  error,
  onRetry,
  onDismiss,
  className = ''
}) => {
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(false);
  
  // Parse error if it's a JSON string
  let userError: UserError;
  
  if (typeof error === 'string') {
    try {
      userError = JSON.parse(error);
    } catch {
      // Fallback for plain string errors
      userError = {
        title: 'Error',
        message: error,
        error_code: 'GENERIC_ERROR'
      };
    }
  } else {
    userError = error;
  }

  const getErrorIcon = (errorCode: string) => {
    switch (errorCode) {
      case 'OLLAMA_OFFLINE':
      case 'OLLAMA_REQUIRED':
        return '🤖';
      case 'SEARXNG_OFFLINE':
        return '🔍';
      case 'CHROMADB_OFFLINE':
        return '📚';
      case 'MODEL_NOT_FOUND':
      case 'MODEL_REQUIRED':
        return '🧠';
      case 'CONNECTION_FAILED':
        return '🌐';
      case 'TIMEOUT':
        return '⏰';
      case 'ACCESS_DENIED':
        return '🔒';
      case 'FILE_NOT_FOUND':
        return '📄';
      case 'OUT_OF_MEMORY':
        return '💾';
      default:
        return '⚠️';
    }
  };

  const getSeverityClass = (errorCode: string) => {
    switch (errorCode) {
      case 'OLLAMA_OFFLINE':
      case 'OLLAMA_REQUIRED':
      case 'MODEL_NOT_FOUND':
      case 'MODEL_REQUIRED':
        return 'error-high';
      case 'SEARXNG_OFFLINE':
      case 'CHROMADB_OFFLINE':
      case 'SERVICE_UNAVAILABLE':
        return 'error-medium';
      case 'TIMEOUT':
      case 'CONNECTION_FAILED':
        return 'error-low';
      default:
        return 'error-medium';
    }
  };

  return (
    <div className={`user-error-display ${getSeverityClass(userError.error_code)} ${className}`}>
      <div className="error-header">
        <span className="error-icon">{getErrorIcon(userError.error_code)}</span>
        <h3 className="error-title">{userError.title}</h3>
        {onDismiss && (
          <button className="error-dismiss" onClick={onDismiss} aria-label="Dismiss error">
            ✕
          </button>
        )}
      </div>

      <div className="error-content">
        <p className="error-message">{userError.message}</p>
        
        {userError.suggestion && (
          <div className="error-suggestion">
            <h4>💡 What you can do:</h4>
            <p>{userError.suggestion}</p>
          </div>
        )}

        {userError.help_link && (
          <div className="error-help-link">
            <a 
              href={userError.help_link} 
              target="_blank" 
              rel="noopener noreferrer"
              className="help-link"
            >
              📖 Learn more
            </a>
          </div>
        )}

        <div className="error-actions">
          {onRetry && (
            <button className="retry-button" onClick={onRetry}>
              🔄 Try Again
            </button>
          )}
          
          {userError.technical_details && (
            <button 
              className="technical-details-toggle"
              onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
            >
              {showTechnicalDetails ? '🔼 Hide' : '🔽 Show'} Technical Details
            </button>
          )}
        </div>

        {showTechnicalDetails && userError.technical_details && (
          <div className="technical-details">
            <h4>Technical Details:</h4>
            <pre className="technical-text">{userError.technical_details}</pre>
            <small className="error-code">Error Code: {userError.error_code}</small>
          </div>
        )}
      </div>
    </div>
  );
};

// Hook for displaying errors with automatic dismissal
export const useUserError = () => {
  const [error, setError] = useState<UserError | null>(null);

  const showError = (error: UserError | string) => {
    setError(typeof error === 'string' ? JSON.parse(error) : error);
  };

  const clearError = () => {
    setError(null);
  };

  const ErrorComponent = error ? (
    <UserErrorDisplay
      error={error}
      onDismiss={clearError}
    />
  ) : null;

  return {
    error,
    showError,
    clearError,
    ErrorComponent
  };
};

// Compact error banner for inline display
export const UserErrorBanner: React.FC<{
  error: UserError | string;
  onDismiss?: () => void;
}> = ({ error, onDismiss }) => {
  let userError: UserError;
  
  if (typeof error === 'string') {
    try {
      userError = JSON.parse(error);
    } catch {
      userError = {
        title: 'Error',
        message: error,
        error_code: 'GENERIC_ERROR'
      };
    }
  } else {
    userError = error;
  }

  return (
    <div className="user-error-banner">
      <span className="error-icon-small">⚠️</span>
      <span className="error-message-compact">{userError.message}</span>
      {onDismiss && (
        <button className="error-dismiss-small" onClick={onDismiss}>✕</button>
      )}
    </div>
  );
};

export default UserErrorDisplay;
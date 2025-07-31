import React from 'react';
import { useSearchHealth, SearchHealthStatus } from '../hooks/useSearchHealth';
import './SearchHealthIndicator.css';

export interface SearchHealthIndicatorProps {
  showDetails?: boolean;
  onHealthChange?: (status: SearchHealthStatus) => void;
  className?: string;
}

export const SearchHealthIndicator: React.FC<SearchHealthIndicatorProps> = ({
  showDetails = false,
  onHealthChange,
  className = ''
}) => {
  const { status, checkHealthNow } = useSearchHealth({
    checkInterval: 30000, // Check every 30 seconds
    enabled: true,
    onHealthChange
  });

  const getStatusColor = () => {
    if (status.isChecking) return '#ffa500'; // Orange
    if (status.isHealthy) return '#4caf50'; // Green
    return '#f44336'; // Red
  };

  const getStatusText = () => {
    if (status.isChecking) return 'Checking...';
    if (status.isHealthy) return 'Search Service Online';
    return 'Search Service Offline';
  };

  const getStatusIcon = () => {
    if (status.isChecking) return '⏳';
    if (status.isHealthy) return '✅';
    return '❌';
  };

  const formatResponseTime = (time: number | null) => {
    if (time === null) return 'N/A';
    if (time < 1000) return `${time}ms`;
    return `${(time / 1000).toFixed(1)}s`;
  };

  const formatLastChecked = (date: Date | null) => {
    if (!date) return 'Never';
    
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    return date.toLocaleTimeString();
  };

  const handleRefresh = async () => {
    try {
      await checkHealthNow();
    } catch (error) {
      console.error('Manual health check failed:', error);
    }
  };

  return (
    <div className={`search-health-indicator ${className}`}>
      <div className="health-status-main">
        <div 
          className="health-indicator-dot"
          style={{ backgroundColor: getStatusColor() }}
          title={getStatusText()}
        />
        
        <span className="health-status-text">
          {getStatusIcon()} {getStatusText()}
        </span>
        
        <button 
          className="health-refresh-button"
          onClick={handleRefresh}
          disabled={status.isChecking}
          title="Check health now"
        >
          🔄
        </button>
      </div>

      {showDetails && (
        <div className="health-details">
          <div className="health-detail-row">
            <span className="health-detail-label">Response Time:</span>
            <span className="health-detail-value">
              {formatResponseTime(status.responseTime)}
            </span>
          </div>
          
          <div className="health-detail-row">
            <span className="health-detail-label">Last Checked:</span>
            <span className="health-detail-value">
              {formatLastChecked(status.lastChecked)}
            </span>
          </div>
          
          {status.error && (
            <div className="health-detail-row health-error">
              <span className="health-detail-label">Error:</span>
              <span className="health-detail-value health-error-text">
                {status.error}
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default SearchHealthIndicator;
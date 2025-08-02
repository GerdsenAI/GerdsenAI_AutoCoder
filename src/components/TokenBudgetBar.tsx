import './TokenBudgetBar.css';

export interface ContextBudget {
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
}

interface TokenBudgetBarProps {
  budget: ContextBudget;
  className?: string;
}

export const TokenBudgetBar = ({ 
  budget, 
  className = '' 
}: TokenBudgetBarProps) => {
  const { total, breakdown } = budget;
  
  // Calculate percentages for visual representation
  const getPercentage = (value: number) => Math.max(0, (value / total) * 100);
  
  const segments = [
    { 
      label: 'Conversation', 
      value: breakdown.conversation,
      percentage: getPercentage(breakdown.conversation),
      color: 'conversation',
      description: 'Current chat messages and context'
    },
    { 
      label: 'RAG Documents', 
      value: breakdown.rag_documents,
      percentage: getPercentage(breakdown.rag_documents),
      color: 'rag',
      description: 'Retrieved documents from knowledge base'
    },
    { 
      label: 'Pinned Files', 
      value: breakdown.pinned_files,
      percentage: getPercentage(breakdown.pinned_files),
      color: 'pinned',
      description: 'Files you\'ve manually pinned to context'
    },
    { 
      label: 'Suggested Files', 
      value: breakdown.suggested_files,
      percentage: getPercentage(breakdown.suggested_files),
      color: 'suggested',
      description: 'AI-suggested relevant files'
    },
    { 
      label: 'Reserved', 
      value: breakdown.reserved,
      percentage: getPercentage(breakdown.reserved),
      color: 'reserved',
      description: 'Reserved for model response generation'
    }
  ];
  
  // Calculate available space
  const usedPercentage = getPercentage(budget.used);
  const availablePercentage = Math.max(0, 100 - usedPercentage);
  
  const formatTokens = (tokens: number): string => {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
  };
  
  return (
    <div className={`token-budget-bar ${className}`}>
      <div className="budget-header">
        <div className="budget-title">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="4" width="18" height="2" rx="1"/>
            <rect x="3" y="8" width="18" height="2" rx="1"/>
            <rect x="3" y="12" width="18" height="2" rx="1"/>
            <rect x="3" y="16" width="18" height="2" rx="1"/>
          </svg>
          <span>Context Budget</span>
        </div>
        <div className="budget-summary">
          <span className="used-tokens">{formatTokens(budget.used)}</span>
          <span className="separator">/</span>
          <span className="total-tokens">{formatTokens(total)}</span>
          <span className="percentage">({Math.round(usedPercentage)}%)</span>
        </div>
      </div>
      
      <div className="budget-bar">
        <div className="bar-track">
          {segments.map((segment) => (
            segment.percentage > 0 && (
              <div
                key={segment.label}
                className={`bar-segment ${segment.color}`}
                style={{ width: `${segment.percentage}%` }}
                title={`${segment.label}: ${formatTokens(segment.value)} tokens (${Math.round(segment.percentage)}%) - ${segment.description}`}
              >
                {segment.percentage > 8 && (
                  <span className="segment-label">
                    {formatTokens(segment.value)}
                  </span>
                )}
              </div>
            )
          ))}
          {availablePercentage > 0 && (
            <div
              className="bar-segment available"
              style={{ width: `${availablePercentage}%` }}
              title={`Available: ${formatTokens(budget.available)} tokens (${Math.round(availablePercentage)}%)`}
            >
              {availablePercentage > 8 && (
                <span className="segment-label available-label">
                  {formatTokens(budget.available)} free
                </span>
              )}
            </div>
          )}
        </div>
      </div>
      
      <div className="budget-legend">
        {segments.map((segment) => (
          segment.value > 0 && (
            <div key={segment.label} className="legend-item">
              <div className={`legend-color ${segment.color}`}></div>
              <span className="legend-label">{segment.label}</span>
              <span className="legend-value">{formatTokens(segment.value)}</span>
            </div>
          )
        ))}
        {budget.available > 0 && (
          <div className="legend-item">
            <div className="legend-color available"></div>
            <span className="legend-label">Available</span>
            <span className="legend-value">{formatTokens(budget.available)}</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default TokenBudgetBar;

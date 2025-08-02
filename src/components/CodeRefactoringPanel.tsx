import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './CodeRefactoringPanel.css';

interface CodeRefactoringProps {
  repoPath: string;
}

interface RefactoringScope {
  file_patterns: string[];
  exclude_patterns: string[];
  max_files: number;
  target_areas: string[];
}

interface RefactoringSuggestion {
  id: string;
  title: string;
  description: string;
  before_code: string;
  after_code: string;
  file_path: string;
  line_range: [number, number];
  effort_level: 'Low' | 'Medium' | 'High';
  benefits: string[];
  category: 'Performance' | 'Readability' | 'Maintainability' | 'Security' | 'Accessibility' | 'Other';
}

interface RefactoringResults {
  suggestions: RefactoringSuggestion[];
  summary: string;
  time_taken: number;
  files_analyzed: number;
}

const CodeRefactoringPanel: React.FC<CodeRefactoringProps> = ({ repoPath }) => {
  const [filePatterns, setFilePatterns] = useState<string>('*.ts,*.tsx,*.js,*.jsx');
  const [excludePatterns, setExcludePatterns] = useState<string>('node_modules,dist,build');
  const [maxFiles, setMaxFiles] = useState<number>(50);
  const [targetAreas, setTargetAreas] = useState<string[]>(['Performance', 'Readability', 'Maintainability']);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [results, setResults] = useState<RefactoringResults | null>(null);
  const [selectedSuggestion, setSelectedSuggestion] = useState<RefactoringSuggestion | null>(null);
  const [expandedSuggestions, setExpandedSuggestions] = useState<Set<string>>(new Set());

  const handleTargetAreaChange = (area: string) => {
    setTargetAreas(prev => 
      prev.includes(area) 
        ? prev.filter(a => a !== area) 
        : [...prev, area]
    );
  };

  const toggleSuggestionExpanded = (id: string) => {
    setExpandedSuggestions(prev => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  const findRefactoringSuggestions = async () => {
    if (!repoPath) {
      setError('Repository path is required');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const scope: RefactoringScope = {
        file_patterns: filePatterns.split(',').map(p => p.trim()),
        exclude_patterns: excludePatterns.split(',').map(p => p.trim()),
        max_files: maxFiles,
        target_areas: targetAreas
      };

      const results = await invoke<RefactoringResults>('analyze_code_quality', {
        repoPath,
        scope
      });

      setResults(results);
      setSelectedSuggestion(null);
      setExpandedSuggestions(new Set());
    } catch (err) {
      console.error('Error finding refactoring suggestions:', err);
      setError(`Error finding refactoring suggestions: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const getCategoryColor = (category: string): string => {
    switch (category) {
      case 'Performance':
        return '#4caf50';
      case 'Readability':
        return '#2196f3';
      case 'Maintainability':
        return '#9c27b0';
      case 'Security':
        return '#f44336';
      case 'Accessibility':
        return '#ff9800';
      default:
        return '#607d8b';
    }
  };

  const getEffortLevelIcon = (level: 'Low' | 'Medium' | 'High'): string => {
    switch (level) {
      case 'Low':
        return '●';
      case 'Medium':
        return '●●';
      case 'High':
        return '●●●';
      default:
        return '';
    }
  };

  const getEffortLevelClass = (level: 'Low' | 'Medium' | 'High'): string => {
    switch (level) {
      case 'Low':
        return 'effort-low';
      case 'Medium':
        return 'effort-medium';
      case 'High':
        return 'effort-high';
      default:
        return '';
    }
  };
  
  const applySuggestion = async (suggestion: RefactoringSuggestion) => {
    try {
      await invoke('apply_refactoring', {
        repoPath,
        suggestion
      });
      // Show success message
      alert(`Refactoring applied successfully to ${suggestion.file_path}`);
    } catch (err) {
      console.error('Error applying refactoring:', err);
      alert(`Error applying refactoring: ${err}`);
    }
  };

  return (
    <div className="refactoring-panel">
      <h2>Code Refactoring Suggestions</h2>

      <div className="refactoring-form">
        <div className="form-group">
          <label htmlFor="file-patterns">File Patterns (comma-separated):</label>
          <input
            id="file-patterns"
            type="text"
            value={filePatterns}
            onChange={(e) => setFilePatterns(e.target.value)}
            placeholder="e.g., *.ts,*.tsx,*.js"
          />
        </div>

        <div className="form-group">
          <label htmlFor="exclude-patterns">Exclude Patterns (comma-separated):</label>
          <input
            id="exclude-patterns"
            type="text"
            value={excludePatterns}
            onChange={(e) => setExcludePatterns(e.target.value)}
            placeholder="e.g., node_modules,dist"
          />
        </div>

        <div className="form-row">
          <div className="form-group">
            <label htmlFor="max-files">Max Files:</label>
            <input
              id="max-files"
              type="number"
              value={maxFiles}
              onChange={(e) => setMaxFiles(parseInt(e.target.value))}
              min={1}
              max={500}
            />
          </div>

          <div className="form-group target-areas">
            <label>Target Areas:</label>
            <div className="checkbox-group">
              {['Performance', 'Readability', 'Maintainability', 'Security', 'Accessibility'].map(area => (
                <label key={area} className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={targetAreas.includes(area)}
                    onChange={() => handleTargetAreaChange(area)}
                  />
                  {area}
                </label>
              ))}
            </div>
          </div>
        </div>

        <button
          className="analyze-button"
          onClick={findRefactoringSuggestions}
          disabled={loading}
        >
          {loading ? 'Analyzing...' : 'Find Refactoring Suggestions'}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {results && (
        <div className="refactoring-results">
          <div className="results-summary">
            <h3>Analysis Results</h3>
            <p>{results.summary}</p>
            <div className="analysis-stats">
              <div className="stat">
                <span className="stat-label">Files Analyzed:</span>
                <span className="stat-value">{results.files_analyzed}</span>
              </div>
              <div className="stat">
                <span className="stat-label">Suggestions Found:</span>
                <span className="stat-value">{results.suggestions.length}</span>
              </div>
              <div className="stat">
                <span className="stat-label">Time Taken:</span>
                <span className="stat-value">{results.time_taken.toFixed(2)}s</span>
              </div>
            </div>
          </div>

          <div className="suggestions-list">
            <h3>Refactoring Suggestions</h3>
            
            {results.suggestions.length === 0 ? (
              <p className="no-suggestions">No refactoring suggestions found. Your code looks good!</p>
            ) : (
              <div className="suggestions-container">
                {results.suggestions.map((suggestion) => (
                  <div 
                    key={suggestion.id} 
                    className={`suggestion-card ${expandedSuggestions.has(suggestion.id) ? 'expanded' : ''}`}
                  >
                    <div 
                      className="suggestion-header"
                      onClick={() => toggleSuggestionExpanded(suggestion.id)}
                    >
                      <div className="suggestion-title-row">
                        <span 
                          className="category-badge" 
                          style={{ backgroundColor: getCategoryColor(suggestion.category) }}
                        >
                          {suggestion.category}
                        </span>
                        <h4 className="suggestion-title">{suggestion.title}</h4>
                        <span className={`effort-level ${getEffortLevelClass(suggestion.effort_level)}`}>
                          {getEffortLevelIcon(suggestion.effort_level)}
                        </span>
                      </div>
                      <div className="suggestion-file">
                        <span className="file-path">{suggestion.file_path}</span>
                        <span className="line-range">Lines {suggestion.line_range[0]}-{suggestion.line_range[1]}</span>
                      </div>
                    </div>

                    {expandedSuggestions.has(suggestion.id) && (
                      <div className="suggestion-details">
                        <p className="suggestion-description">{suggestion.description}</p>
                        
                        <div className="benefits-section">
                          <h5>Benefits:</h5>
                          <ul className="benefits-list">
                            {suggestion.benefits.map((benefit, i) => (
                              <li key={`benefit-${i}`}>{benefit}</li>
                            ))}
                          </ul>
                        </div>

                        <div className="code-comparison">
                          <div className="code-before">
                            <h5>Before:</h5>
                            <pre className="code-block">{suggestion.before_code}</pre>
                          </div>
                          <div className="code-after">
                            <h5>After:</h5>
                            <pre className="code-block">{suggestion.after_code}</pre>
                          </div>
                        </div>

                        <div className="suggestion-actions">
                          <button 
                            className="apply-button" 
                            onClick={() => applySuggestion(suggestion)}
                          >
                            Apply Refactoring
                          </button>
                          <button 
                            className="view-button"
                            onClick={() => setSelectedSuggestion(suggestion)}
                          >
                            View Full Context
                          </button>
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {selectedSuggestion && (
        <div className="suggestion-modal">
          <div className="modal-content">
            <div className="modal-header">
              <h3>{selectedSuggestion.title}</h3>
              <button className="close-button" onClick={() => setSelectedSuggestion(null)}>×</button>
            </div>
            <div className="modal-body">
              <div className="modal-file-info">
                <strong>File:</strong> {selectedSuggestion.file_path}
              </div>
              
              <p className="modal-description">{selectedSuggestion.description}</p>
              
              <div className="modal-code-comparison">
                <div className="modal-code-section">
                  <h4>Before:</h4>
                  <pre className="code-block">{selectedSuggestion.before_code}</pre>
                </div>
                <div className="modal-code-section">
                  <h4>After:</h4>
                  <pre className="code-block">{selectedSuggestion.after_code}</pre>
                </div>
              </div>
              
              <div className="modal-actions">
                <button 
                  className="apply-button"
                  onClick={() => {
                    applySuggestion(selectedSuggestion);
                    setSelectedSuggestion(null);
                  }}
                >
                  Apply Refactoring
                </button>
                <button 
                  className="cancel-button"
                  onClick={() => setSelectedSuggestion(null)}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default CodeRefactoringPanel;

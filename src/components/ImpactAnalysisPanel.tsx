import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './ImpactAnalysisPanel.css';

interface ImpactAnalysisProps {
  repoPath: string;
}

interface CodeChange {
  file_path: string;
  start_line: number;
  end_line: number;
  change_type: 'Add' | 'Modify' | 'Delete';
  code_snippet: string;
}

interface AffectedFile {
  file_path: string;
  impact_level: 'High' | 'Medium' | 'Low';
  affected_symbols: string[];
  reason: string;
}

interface ImpactAnalysisResult {
  affected_files: AffectedFile[];
  risk_assessment: {
    risk_level: 'High' | 'Medium' | 'Low';
    risk_factors: string[];
    mitigations: string[];
  };
  summary: string;
}

const ImpactAnalysisPanel: React.FC<ImpactAnalysisProps> = ({ repoPath }) => {
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [startLine, setStartLine] = useState<number>(1);
  const [endLine, setEndLine] = useState<number>(10);
  const [changeType, setChangeType] = useState<'Add' | 'Modify' | 'Delete'>('Modify');
  const [codeSnippet, setCodeSnippet] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ImpactAnalysisResult | null>(null);

  const analyzeImpact = async () => {
    if (!repoPath) {
      setError('Repository path is required');
      return;
    }

    if (!selectedFile) {
      setError('File path is required');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const change: CodeChange = {
        file_path: selectedFile,
        start_line: startLine,
        end_line: endLine,
        change_type: changeType,
        code_snippet: codeSnippet
      };

      const result = await invoke<ImpactAnalysisResult>('analyze_code_impact', {
        repoPath,
        change
      });

      setResult(result);
    } catch (err) {
      console.error('Error analyzing impact:', err);
      setError(`Error analyzing impact: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const getRiskLevelClass = (level: 'High' | 'Medium' | 'Low'): string => {
    switch (level) {
      case 'High':
        return 'risk-high';
      case 'Medium':
        return 'risk-medium';
      case 'Low':
        return 'risk-low';
      default:
        return '';
    }
  };

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedFile(e.target.value);

    if (e.target.value) {
      try {
        // Get file content to help with line selection
        const content = await invoke<string>('read_file_content', {
          path: e.target.value
        });
        
        // Calculate reasonable default for end line
        const lineCount = content.split('\n').length;
        setEndLine(Math.min(startLine + 10, lineCount));
      } catch (err) {
        console.error('Error reading file:', err);
      }
    }
  };

  return (
    <div className="impact-analysis-panel">
      <h2>Impact Analysis</h2>

      <div className="impact-form">
        <div className="form-group">
          <label htmlFor="file-path">File Path:</label>
          <input
            id="file-path"
            type="text"
            value={selectedFile}
            onChange={handleFileSelect}
            placeholder="Path to file you want to change (relative to repo)"
          />
        </div>

        <div className="form-row">
          <div className="form-group">
            <label htmlFor="start-line">Start Line:</label>
            <input
              id="start-line"
              type="number"
              value={startLine}
              onChange={(e) => setStartLine(parseInt(e.target.value))}
              min={1}
            />
          </div>

          <div className="form-group">
            <label htmlFor="end-line">End Line:</label>
            <input
              id="end-line"
              type="number"
              value={endLine}
              onChange={(e) => setEndLine(parseInt(e.target.value))}
              min={1}
            />
          </div>

          <div className="form-group">
            <label htmlFor="change-type">Change Type:</label>
            <select 
              id="change-type" 
              value={changeType}
              onChange={(e) => setChangeType(e.target.value as 'Add' | 'Modify' | 'Delete')}
            >
              <option value="Add">Add</option>
              <option value="Modify">Modify</option>
              <option value="Delete">Delete</option>
            </select>
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="code-snippet">
            {changeType === 'Delete' ? 'Code to be deleted:' : 'New/Modified Code:'}
          </label>
          <textarea
            id="code-snippet"
            value={codeSnippet}
            onChange={(e) => setCodeSnippet(e.target.value)}
            placeholder={
              changeType === 'Delete' 
                ? 'Optional: Paste the code that will be deleted for better analysis' 
                : 'Paste your new/modified code here for analysis'
            }
            rows={8}
          />
        </div>

        <button 
          className="analyze-button"
          onClick={analyzeImpact}
          disabled={loading}
        >
          {loading ? 'Analyzing...' : 'Analyze Impact'}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {result && (
        <div className="analysis-results">
          <div className="result-summary">
            <h3>Impact Analysis Summary</h3>
            <p>{result.summary}</p>
            
            <div className={`risk-assessment ${getRiskLevelClass(result.risk_assessment.risk_level)}`}>
              <h4>Risk Level: {result.risk_assessment.risk_level}</h4>
              <div className="risk-details">
                <div className="risk-section">
                  <h5>Risk Factors:</h5>
                  <ul>
                    {result.risk_assessment.risk_factors.map((factor, i) => (
                      <li key={`factor-${i}`}>{factor}</li>
                    ))}
                  </ul>
                </div>
                
                <div className="risk-section">
                  <h5>Recommended Mitigations:</h5>
                  <ul>
                    {result.risk_assessment.mitigations.map((mitigation, i) => (
                      <li key={`mitigation-${i}`}>{mitigation}</li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          </div>
          
          <div className="affected-files">
            <h3>Affected Files</h3>
            <p className="affected-count">
              {result.affected_files.length} file(s) potentially affected by this change
            </p>
            
            {result.affected_files.map((file, i) => (
              <div 
                key={`file-${i}`} 
                className={`affected-file ${getRiskLevelClass(file.impact_level)}`}
              >
                <div className="file-header">
                  <span className="file-path">{file.file_path}</span>
                  <span className={`impact-badge ${getRiskLevelClass(file.impact_level)}`}>
                    {file.impact_level} Impact
                  </span>
                </div>
                
                <div className="file-details">
                  <div className="file-reason">
                    <strong>Reason:</strong> {file.reason}
                  </div>
                  
                  <div className="file-symbols">
                    <strong>Affected Symbols:</strong>
                    <ul className="symbols-list">
                      {file.affected_symbols.map((symbol, j) => (
                        <li key={`symbol-${i}-${j}`}>{symbol}</li>
                      ))}
                    </ul>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ImpactAnalysisPanel;

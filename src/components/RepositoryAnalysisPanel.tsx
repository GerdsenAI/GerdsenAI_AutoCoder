import React, { useState } from 'react';
import { useRepositoryAnalysis } from '../hooks/useRepositoryAnalysis';
import type { DependencyRelation, CodeMetrics } from '../services/repository-analyzer';
import './RepositoryAnalysisPanel.css';

export interface RepositoryAnalysisPanelProps {
  onFileSelect?: (filePath: string) => void;
  className?: string;
  defaultPath?: string;
}

export const RepositoryAnalysisPanel: React.FC<RepositoryAnalysisPanelProps> = ({
  onFileSelect,
  className = '',
  defaultPath = ''
}) => {
  // Business logic handled by custom hook (Socratic architecture)
  const {
    loading,
    error,
    currentPath,
    selectedFile,
    analysisResult,
    fileStructure,
    dependencies,
    metrics,
    aiAnalysis,
    documentation,
    testCoverage,
    analyzeRepository,
    analyzeDependencies,
    getCodeMetrics,
    performAiAnalysis,
    generateFileDocumentation,
    analyzeProjectTestCoverage,
    selectFile,
    resetAnalysis,
    clearError,
    selectedFileMetrics,
    selectedFileDependencies,
    hasResults,
    canAnalyze,
    isFileSelected
  } = useRepositoryAnalysis();

  // UI-only state
  const [repositoryPath, setRepositoryPath] = useState(defaultPath);
  const [activeTab, setActiveTab] = useState<'overview' | 'dependencies' | 'metrics' | 'ai' | 'coverage'>('overview');
  const [analysisType, setAnalysisType] = useState<'refactoring' | 'optimization' | 'testing' | 'security'>('refactoring');

  // Event handlers - clean and simple (Socratic principle: delegate to hooks)
  const handleAnalyzeRepository = async () => {
    if (!repositoryPath.trim()) return;
    await analyzeRepository(repositoryPath);
  };

  const handleFileSelect = (filePath: string) => {
    selectFile(filePath);
    onFileSelect?.(filePath);
  };

  const handleAnalyzeDependencies = async () => {
    if (!selectedFile) return;
    await analyzeDependencies(selectedFile);
  };

  const handleGetMetrics = async () => {
    if (!selectedFile) return;
    await getCodeMetrics(selectedFile);
  };

  const handleAiAnalysis = async () => {
    if (!selectedFile) return;
    await performAiAnalysis(selectedFile, analysisType);
  };

  const handleGenerateDocumentation = async () => {
    if (!selectedFile) return;
    await generateFileDocumentation(selectedFile);
  };

  const handleAnalyzeCoverage = async () => {
    if (!repositoryPath.trim()) return;
    await analyzeProjectTestCoverage(repositoryPath);
  };

  // Render helpers
  const renderFileStructure = (node: any, level = 0): React.ReactNode => {
    if (!node) return null;

    const indent = level * 20;
    
    return (
      <div key={node.path} style={{ marginLeft: `${indent}px` }} className="file-node">
        <div 
          className={`file-item ${node.type} ${selectedFile === node.path ? 'selected' : ''}`}
          onClick={() => node.type === 'file' && handleFileSelect(node.path)}
        >
          <span className="file-icon">
            {node.type === 'directory' ? '📁' : '📄'}
          </span>
          <span className="file-name">{node.name}</span>
          {node.type === 'file' && node.language && (
            <span className="file-language">{node.language}</span>
          )}
        </div>
        {node.children?.map((child: any) => renderFileStructure(child, level + 1))}
      </div>
    );
  };

  const renderDependencies = (deps: DependencyRelation[]) => (
    <div className="dependencies-list">
      {deps.length === 0 ? (
        <div className="empty-state">No dependencies found</div>
      ) : (
        deps.map((dep, index) => (
          <div key={index} className="dependency-item">
            <div className="dependency-type">{dep.type}</div>
            <div className="dependency-path">
              <strong>From:</strong> {dep.from}
            </div>
            <div className="dependency-path">
              <strong>To:</strong> {dep.to}
            </div>
            {dep.import_name && (
              <div className="dependency-import">
                <strong>Import:</strong> {dep.import_name}
              </div>
            )}
            {dep.line_number && (
              <div className="dependency-line">
                <strong>Line:</strong> {dep.line_number}
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );

  const renderMetrics = (metricsData: CodeMetrics[]) => (
    <div className="metrics-list">
      {metricsData.length === 0 ? (
        <div className="empty-state">No metrics available</div>
      ) : (
        metricsData.map((metric, index) => (
          <div 
            key={index} 
            className={`metrics-item ${selectedFile === metric.file_path ? 'selected' : ''}`}
            onClick={() => handleFileSelect(metric.file_path)}
          >
            <div className="metrics-header">
              <span className="file-path">{metric.file_path}</span>
            </div>
            <div className="metrics-data">
              <div className="metric">
                <span className="label">Lines of Code:</span>
                <span className="value">{metric.lines_of_code}</span>
              </div>
              <div className="metric">
                <span className="label">Complexity:</span>
                <span className={`value complexity-${metric.complexity_score > 10 ? 'high' : metric.complexity_score > 5 ? 'medium' : 'low'}`}>
                  {metric.complexity_score}
                </span>
              </div>
              <div className="metric">
                <span className="label">Maintainability:</span>
                <span className={`value maintainability-${metric.maintainability_index < 50 ? 'low' : metric.maintainability_index < 80 ? 'medium' : 'high'}`}>
                  {metric.maintainability_index}
                </span>
              </div>
              <div className="metric">
                <span className="label">Dependencies:</span>
                <span className="value">{metric.dependencies_count}</span>
              </div>
              <div className="metric">
                <span className="label">Dependents:</span>
                <span className="value">{metric.dependents_count}</span>
              </div>
              {metric.test_coverage !== undefined && (
                <div className="metric">
                  <span className="label">Test Coverage:</span>
                  <span className={`value coverage-${metric.test_coverage < 50 ? 'low' : metric.test_coverage < 80 ? 'medium' : 'high'}`}>
                    {metric.test_coverage}%
                  </span>
                </div>
              )}
            </div>
          </div>
        ))
      )}
    </div>
  );

  return (
    <div className={`repository-analysis-panel ${className}`}>
      {/* Header Section */}
      <div className="analysis-header">
        <div className="path-input-section">
          <label htmlFor="repo-path">Repository Path:</label>
          <input
            id="repo-path"
            type="text"
            value={repositoryPath}
            onChange={(e) => setRepositoryPath(e.target.value)}
            placeholder="Enter repository path..."
            className="repository-path-input"
            disabled={loading}
          />
          <button
            onClick={handleAnalyzeRepository}
            disabled={!canAnalyze || !repositoryPath.trim()}
            className="analyze-button primary"
          >
            {loading ? 'Analyzing...' : 'Analyze Repository'}
          </button>
          {hasResults && (
            <button
              onClick={resetAnalysis}
              disabled={loading}
              className="reset-button secondary"
            >
              Reset
            </button>
          )}
        </div>

        {/* Tab Navigation */}
        {hasResults && (
          <div className="tab-navigation">
            <button
              className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
              onClick={() => setActiveTab('overview')}
            >
              Overview
            </button>
            <button
              className={`tab ${activeTab === 'dependencies' ? 'active' : ''}`}
              onClick={() => setActiveTab('dependencies')}
            >
              Dependencies
            </button>
            <button
              className={`tab ${activeTab === 'metrics' ? 'active' : ''}`}
              onClick={() => setActiveTab('metrics')}
            >
              Metrics
            </button>
            <button
              className={`tab ${activeTab === 'ai' ? 'active' : ''}`}
              onClick={() => setActiveTab('ai')}
            >
              AI Analysis
            </button>
            <button
              className={`tab ${activeTab === 'coverage' ? 'active' : ''}`}
              onClick={() => setActiveTab('coverage')}
            >
              Coverage
            </button>
          </div>
        )}
      </div>

      {/* Results Section */}
      <div className="analysis-results">
        {/* Error Display */}
        {error && (
          <div className="analysis-error">
            {error}
            <button onClick={clearError} className="error-dismiss">×</button>
          </div>
        )}

        {/* Loading State */}
        {loading && (
          <div className="analysis-loading">
            <span>Analyzing</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        )}

        {/* Results Display */}
        {hasResults && !loading && (
          <div className="results-content">
            {activeTab === 'overview' && (
              <div className="overview-tab">
                <div className="file-structure-section">
                  <h3>File Structure</h3>
                  <div className="file-tree">
                    {renderFileStructure(fileStructure)}
                  </div>
                </div>
                
                {selectedFile && (
                  <div className="selected-file-info">
                    <h3>Selected File: {selectedFile}</h3>
                    <div className="file-actions">
                      <button
                        onClick={handleAnalyzeDependencies}
                        disabled={loading}
                        className="action-button"
                      >
                        Analyze Dependencies
                      </button>
                      <button
                        onClick={handleGetMetrics}
                        disabled={loading}
                        className="action-button"
                      >
                        Get Metrics
                      </button>
                    </div>
                    
                    {selectedFileMetrics && (
                      <div className="quick-metrics">
                        <div className="metric">
                          <span>LOC: {selectedFileMetrics.lines_of_code}</span>
                        </div>
                        <div className="metric">
                          <span>Complexity: {selectedFileMetrics.complexity_score}</span>
                        </div>
                        <div className="metric">
                          <span>Maintainability: {selectedFileMetrics.maintainability_index}</span>
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}

            {activeTab === 'dependencies' && (
              <div className="dependencies-tab">
                <h3>Dependencies Analysis</h3>
                {isFileSelected ? (
                  <div>
                    <h4>Dependencies for: {selectedFile}</h4>
                    {renderDependencies(selectedFileDependencies)}
                  </div>
                ) : (
                  <div>
                    <h4>All Dependencies</h4>
                    {renderDependencies(dependencies)}
                  </div>
                )}
              </div>
            )}

            {activeTab === 'metrics' && (
              <div className="metrics-tab">
                <h3>Code Metrics</h3>
                {renderMetrics(metrics)}
              </div>
            )}

            {activeTab === 'ai' && (
              <div className="ai-tab">
                <h3>AI-Enhanced Analysis</h3>
                {isFileSelected ? (
                  <div>
                    <div className="ai-controls">
                      <label htmlFor="analysis-type">Analysis Type:</label>
                      <select
                        id="analysis-type"
                        value={analysisType}
                        onChange={(e) => setAnalysisType(e.target.value as any)}
                      >
                        <option value="refactoring">Refactoring</option>
                        <option value="optimization">Optimization</option>
                        <option value="testing">Testing</option>
                        <option value="security">Security</option>
                      </select>
                      <button
                        onClick={handleAiAnalysis}
                        disabled={loading}
                        className="action-button"
                      >
                        Run AI Analysis
                      </button>
                      <button
                        onClick={handleGenerateDocumentation}
                        disabled={loading}
                        className="action-button"
                      >
                        Generate Docs
                      </button>
                    </div>

                    {aiAnalysis && (
                      <div className="ai-results">
                        <h4>Analysis Results</h4>
                        <div className="ai-analysis">{aiAnalysis.analysis}</div>
                        <h4>Suggestions</h4>
                        <ul className="ai-suggestions">
                          {aiAnalysis.suggestions.map((suggestion, index) => (
                            <li key={index}>{suggestion}</li>
                          ))}
                        </ul>
                        {aiAnalysis.code_examples && (
                          <div>
                            <h4>Code Examples</h4>
                            <div className="code-examples">
                              {aiAnalysis.code_examples.map((example, index) => (
                                <pre key={index} className="code-example">{example}</pre>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    )}

                    {documentation && (
                      <div className="documentation-results">
                        <h4>Generated Documentation</h4>
                        <div className="generated-docs">{documentation}</div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="empty-state">
                    Select a file to perform AI analysis
                  </div>
                )}
              </div>
            )}

            {activeTab === 'coverage' && (
              <div className="coverage-tab">
                <h3>Test Coverage Analysis</h3>
                <button
                  onClick={handleAnalyzeCoverage}
                  disabled={loading}
                  className="action-button"
                >
                  Analyze Coverage
                </button>
                
                {testCoverage && (
                  <div className="coverage-results">
                    <div className="overall-coverage">
                      <h4>Overall Coverage: {testCoverage.overall_coverage}%</h4>
                      <div className={`coverage-bar coverage-${testCoverage.overall_coverage < 50 ? 'low' : testCoverage.overall_coverage < 80 ? 'medium' : 'high'}`}>
                        <div 
                          className="coverage-fill"
                          style={{ width: `${testCoverage.overall_coverage}%` }}
                        />
                      </div>
                    </div>
                    
                    <div className="file-coverage">
                      <h4>File Coverage</h4>
                      {Object.entries(testCoverage.file_coverage).map(([filePath, coverage]) => (
                        <div key={filePath} className="file-coverage-item">
                          <span className="file-path">{filePath}</span>
                          <span className={`coverage-value coverage-${coverage < 50 ? 'low' : coverage < 80 ? 'medium' : 'high'}`}>
                            {coverage}%
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {/* Empty State */}
        {!hasResults && !loading && !error && (
          <div className="empty-state">
            <p>Enter a repository path and click "Analyze Repository" to begin</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default RepositoryAnalysisPanel;
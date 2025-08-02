import React, { useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './DependencyGraph.css';

// Type definitions to match our Rust backend
interface DependencyNode {
  file_path: string;
  language: string;
  symbols: Symbol[];
  imports: Import[];
  exports: string[];
}

interface Symbol {
  name: string;
  kind: string;
  location: Range;
  documentation?: string;
  is_exported: boolean;
}

interface Import {
  source: string;
  symbols: string[];
  is_all: boolean;
  location: Range;
}

interface Range {
  start: Position;
  end: Position;
}

interface Position {
  line: number;
  character: number;
}

interface DependencyEdge {
  from: string;
  to: string;
  symbols: string[];
  strength: 'Weak' | 'Medium' | 'Strong';
}

interface DependencyGraph {
  nodes: Record<string, DependencyNode>;
  edges: DependencyEdge[];
  summary: string;
}

interface DependencyAnalysisRequest {
  repo_path: string;
  file_patterns?: string[];
  exclude_patterns?: string[];
  max_files?: number;
  include_content?: boolean;
}

interface GraphNode {
  id: string;
  label: string;
  type: string;
  details: DependencyNode;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label: string;
  strength: string;
  symbols: string[];
}

interface DependencyGraphProps {
  repoPath: string;
}

const DependencyGraph: React.FC<DependencyGraphProps> = ({ repoPath }) => {
  const [filePatterns, setFilePatterns] = useState<string>('*.ts,*.tsx,*.js,*.jsx');
  const [excludePatterns, setExcludePatterns] = useState<string>('node_modules,dist,build');
  const [maxFiles, setMaxFiles] = useState<number>(50);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [graph, setGraph] = useState<DependencyGraph | null>(null);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  const handleAnalyzeClick = async () => {
    if (!repoPath) {
      setError('Repository path is required');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const request: DependencyAnalysisRequest = {
        repo_path: repoPath,
        file_patterns: filePatterns.split(',').map(p => p.trim()),
        exclude_patterns: excludePatterns.split(',').map(p => p.trim()),
        max_files: maxFiles,
        include_content: false,
      };

      const result = await invoke<DependencyGraph>('analyze_dependencies', { request });
      setGraph(result);
      
      // Transform data for visualization
      const graphNodes: GraphNode[] = Object.entries(result.nodes).map(([id, node]) => ({
        id,
        label: getFileName(id),
        type: node.language,
        details: node,
      }));

      const graphEdges: GraphEdge[] = result.edges.map((edge, i) => ({
        id: `e${i}`,
        source: edge.from,
        target: edge.to,
        label: edge.strength,
        strength: edge.strength,
        symbols: edge.symbols,
      }));

      setNodes(graphNodes);
      setEdges(graphEdges);
    } catch (err) {
      console.error('Error analyzing dependencies:', err);
      setError(`Error analyzing dependencies: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const getFileName = (path: string) => {
    return path.split('/').pop() || path;
  };

  const handleNodeClick = (nodeId: string) => {
    setSelectedNode(nodeId === selectedNode ? null : nodeId);
  };

  const getNodeColor = (type: string) => {
    switch (type.toLowerCase()) {
      case 'javascript':
      case 'js':
        return '#f7df1e';
      case 'typescript':
      case 'ts':
        return '#3178c6';
      case 'jsx':
        return '#61dafb';
      case 'tsx':
        return '#00acd7';
      case 'python':
      case 'py':
        return '#3776ab';
      case 'rust':
      case 'rs':
        return '#dea584';
      default:
        return '#cccccc';
    }
  };

  // Utility function - used in a future implementation
  // const getEdgeColor = (strength: string) => {
  //   switch (strength) {
  //     case 'strong':
  //       return '#4CAF50';
  //     case 'medium':
  //       return '#2196F3';
  //     case 'weak':
  //       return '#9E9E9E';
  //     default:
  //       return '#9E9E9E';
  //   }
  // };

  return (
    <div className="dependency-graph-panel">
      <h2>Code Dependency Analysis</h2>

      <div className="input-section">
        <div className="input-group">
          <label htmlFor="file-patterns">File Patterns (comma-separated):</label>
          <input
            id="file-patterns"
            type="text"
            value={filePatterns}
            onChange={(e) => setFilePatterns(e.target.value)}
            placeholder="e.g., *.ts,*.tsx,*.js"
          />
        </div>

        <div className="input-group">
          <label htmlFor="exclude-patterns">Exclude Patterns (comma-separated):</label>
          <input
            id="exclude-patterns"
            type="text"
            value={excludePatterns}
            onChange={(e) => setExcludePatterns(e.target.value)}
            placeholder="e.g., node_modules,dist"
          />
        </div>

        <div className="input-group">
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

        <button
          className="analyze-button"
          onClick={handleAnalyzeClick}
          disabled={loading}
        >
          {loading ? 'Analyzing...' : 'Analyze Dependencies'}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {graph && (
        <div className="graph-results">
          <div className="graph-summary">
            <h3>Dependency Analysis</h3>
            <p>{graph.summary}</p>
          </div>

          <div className="graph-visualization" ref={canvasRef}>
            {/* In a real implementation, this would use a visualization library like D3.js or Cytoscape.js */}
            {/* For this mock-up, we'll display a simplified representation */}
            <div className="mock-graph">
              <div className="graph-info">
                <p>This is a mock visualization. In a real implementation, this would render an interactive dependency graph using D3.js or similar.</p>
              </div>
              <div className="node-list">
                <h4>Files ({nodes.length})</h4>
                <ul>
                  {nodes.map((node) => (
                    <li 
                      key={node.id} 
                      className={`graph-node ${selectedNode === node.id ? 'selected' : ''}`}
                      onClick={() => handleNodeClick(node.id)}
                    >
                      <span 
                        className="node-color" 
                        style={{ backgroundColor: getNodeColor(node.type) }}
                      ></span>
                      {node.label}
                      <span className="node-type">{node.type}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>

          {selectedNode && (
            <div className="node-details">
              <h3>File Details: {getFileName(selectedNode)}</h3>
              <div className="detail-section">
                <h4>Path</h4>
                <p className="file-path">{selectedNode}</p>
              </div>
              
              <div className="detail-section">
                <h4>Language</h4>
                <p>{graph.nodes[selectedNode]?.language || 'Unknown'}</p>
              </div>
              
              <div className="detail-section">
                <h4>Exports ({graph.nodes[selectedNode]?.exports.length || 0})</h4>
                {graph.nodes[selectedNode]?.exports.length ? (
                  <ul className="exports-list">
                    {graph.nodes[selectedNode]?.exports.map((exp, i) => (
                      <li key={`exp-${i}`}>{exp}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No exports</p>
                )}
              </div>
              
              <div className="detail-section">
                <h4>Imports ({graph.nodes[selectedNode]?.imports.length || 0})</h4>
                {graph.nodes[selectedNode]?.imports.length ? (
                  <ul className="imports-list">
                    {graph.nodes[selectedNode]?.imports.map((imp, i) => (
                      <li key={`imp-${i}`}>
                        <strong>{imp.source}</strong>
                        {imp.is_all ? (
                          <span> (import all)</span>
                        ) : (
                          <span>: {imp.symbols.join(', ')}</span>
                        )}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No imports</p>
                )}
              </div>

              <div className="detail-section">
                <h4>Dependencies</h4>
                <ul className="dependency-list">
                  {edges
                    .filter(edge => edge.source === selectedNode)
                    .map((edge, i) => (
                      <li key={`dep-${i}`} className={`dependency-item ${edge.strength.toLowerCase()}`}>
                        <span>{getFileName(edge.target)}</span>
                        <span className="dependency-strength">{edge.strength}</span>
                      </li>
                    ))}
                </ul>
              </div>
              
              <div className="detail-section">
                <h4>Dependents</h4>
                <ul className="dependency-list">
                  {edges
                    .filter(edge => edge.target === selectedNode)
                    .map((edge, i) => (
                      <li key={`depby-${i}`} className={`dependency-item ${edge.strength.toLowerCase()}`}>
                        <span>{getFileName(edge.source)}</span>
                        <span className="dependency-strength">{edge.strength}</span>
                      </li>
                    ))}
                </ul>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default DependencyGraph;

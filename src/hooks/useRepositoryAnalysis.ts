import { useState, useCallback } from 'react';
import { 
  repositoryAnalyzer, 
  aiEnhancedAnalysis,
  generateDocumentation,
  analyzeTestCoverage,
  type AnalysisResult,
  type DependencyRelation,
  type CodeMetrics,
  type FileNode
} from '../services/repository-analyzer';

export interface UseRepositoryAnalysisOptions {
  onError?: (error: Error) => void;
  autoLoadStructure?: boolean;
}

export function useRepositoryAnalysis(options: UseRepositoryAnalysisOptions = {}) {
  const { onError, autoLoadStructure = false } = options;

  // Core state
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPath, setCurrentPath] = useState<string>('');
  
  // Analysis results state
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null);
  const [fileStructure, setFileStructure] = useState<FileNode | null>(null);
  const [dependencies, setDependencies] = useState<DependencyRelation[]>([]);
  const [metrics, setMetrics] = useState<CodeMetrics[]>([]);
  const [selectedFile, setSelectedFile] = useState<string>('');
  
  // AI-enhanced analysis state
  const [aiAnalysis, setAiAnalysis] = useState<{
    analysis: string;
    suggestions: string[];
    code_examples?: string[];
  } | null>(null);
  const [documentation, setDocumentation] = useState<string>('');
  const [testCoverage, setTestCoverage] = useState<{
    overall_coverage: number;
    file_coverage: { [filePath: string]: number };
    uncovered_lines: { [filePath: string]: number[] };
  } | null>(null);

  const handleError = useCallback((err: unknown, defaultMessage: string) => {
    const errorMessage = err instanceof Error ? err.message : defaultMessage;
    setError(errorMessage);
    if (onError) {
      onError(err instanceof Error ? err : new Error(defaultMessage));
    }
  }, [onError]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Core analysis functions
  const analyzeRepository = useCallback(async (rootPath: string) => {
    if (!rootPath.trim()) {
      handleError(new Error('Root path cannot be empty'), 'Root path is required');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      setCurrentPath(rootPath);

      const result = await repositoryAnalyzer.analyzeRepository(rootPath);
      setAnalysisResult(result);
      setFileStructure(result.file_structure);
      setDependencies(result.dependencies);
      setMetrics(result.metrics);

    } catch (err) {
      handleError(err, 'Failed to analyze repository');
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const analyzeDependencies = useCallback(async (filePath: string) => {
    if (!filePath.trim()) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const deps = await repositoryAnalyzer.analyzeDependencies(filePath);
      setDependencies(deps);

    } catch (err) {
      handleError(err, 'Failed to analyze dependencies');
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const getCodeMetrics = useCallback(async (filePath: string) => {
    if (!filePath.trim()) {
      return null;
    }

    try {
      setLoading(true);
      setError(null);

      const fileMetrics = await repositoryAnalyzer.getCodeMetrics(filePath);
      
      // Update metrics array with the new data
      setMetrics(prev => {
        const filtered = prev.filter(m => m.file_path !== filePath);
        return [...filtered, fileMetrics];
      });

      return fileMetrics;

    } catch (err) {
      handleError(err, 'Failed to get code metrics');
      return null;
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const findRelatedFiles = useCallback(async (filePath: string) => {
    if (!filePath.trim()) {
      return [];
    }

    try {
      setLoading(true);
      setError(null);

      const relatedFiles = await repositoryAnalyzer.findRelatedFiles(filePath);
      return relatedFiles;

    } catch (err) {
      handleError(err, 'Failed to find related files');
      return [];
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const suggestRefactoring = useCallback(async (filePath: string) => {
    if (!filePath.trim()) {
      return [];
    }

    try {
      setLoading(true);
      setError(null);

      const suggestions = await repositoryAnalyzer.suggestRefactoring(filePath);
      return suggestions;

    } catch (err) {
      handleError(err, 'Failed to get refactoring suggestions');
      return [];
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  // AI-enhanced analysis functions
  const performAiAnalysis = useCallback(async (
    filePath: string,
    analysisType: 'refactoring' | 'optimization' | 'testing' | 'security'
  ) => {
    if (!filePath.trim()) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const result = await aiEnhancedAnalysis(filePath, analysisType);
      setAiAnalysis(result);

    } catch (err) {
      handleError(err, 'Failed to perform AI analysis');
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const generateFileDocumentation = useCallback(async (filePath: string) => {
    if (!filePath.trim()) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const docs = await generateDocumentation(filePath);
      setDocumentation(docs);

    } catch (err) {
      handleError(err, 'Failed to generate documentation');
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  const analyzeProjectTestCoverage = useCallback(async (projectPath: string) => {
    if (!projectPath.trim()) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const coverage = await analyzeTestCoverage(projectPath);
      setTestCoverage(coverage);

    } catch (err) {
      handleError(err, 'Failed to analyze test coverage');
    } finally {
      setLoading(false);
    }
  }, [handleError]);

  // Utility functions
  const selectFile = useCallback((filePath: string) => {
    setSelectedFile(filePath);
    clearError();
  }, [clearError]);

  const resetAnalysis = useCallback(() => {
    setAnalysisResult(null);
    setFileStructure(null);
    setDependencies([]);
    setMetrics([]);
    setSelectedFile('');
    setAiAnalysis(null);
    setDocumentation('');
    setTestCoverage(null);
    setCurrentPath('');
    clearError();
  }, [clearError]);

  // Computed values
  const selectedFileMetrics = selectedFile ? 
    metrics.find(m => m.file_path === selectedFile) : null;
  
  const selectedFileDependencies = selectedFile ?
    dependencies.filter(d => d.from === selectedFile || d.to === selectedFile) : [];

  const hasResults = analysisResult !== null;
  const hasFileStructure = fileStructure !== null;
  const hasMetrics = metrics.length > 0;
  const hasDependencies = dependencies.length > 0;

  return {
    // State
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
    
    // Actions
    analyzeRepository,
    analyzeDependencies,
    getCodeMetrics,
    findRelatedFiles,
    suggestRefactoring,
    performAiAnalysis,
    generateFileDocumentation,
    analyzeProjectTestCoverage,
    selectFile,
    resetAnalysis,
    clearError,
    
    // Computed
    selectedFileMetrics,
    selectedFileDependencies,
    hasResults,
    hasFileStructure,
    hasMetrics,
    hasDependencies,
    
    // Convenience
    canAnalyze: currentPath.trim().length > 0 && !loading,
    isFileSelected: selectedFile.trim().length > 0
  };
}
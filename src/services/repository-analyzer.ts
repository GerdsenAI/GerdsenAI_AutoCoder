import { invoke } from '@tauri-apps/api/core';

export interface FileNode {
  path: string;
  name: string;
  type: 'file' | 'directory';
  size?: number;
  language?: string;
  children?: FileNode[];
}

export interface DependencyRelation {
  from: string;
  to: string;
  type: 'import' | 'require' | 'include' | 'reference';
  line_number?: number;
  import_name?: string;
}

export interface CodeMetrics {
  file_path: string;
  lines_of_code: number;
  complexity_score: number;
  maintainability_index: number;
  test_coverage?: number;
  dependencies_count: number;
  dependents_count: number;
}

export interface AnalysisResult {
  file_structure: FileNode;
  dependencies: DependencyRelation[];
  metrics: CodeMetrics[];
  hotspots: string[];
  recommendations: string[];
}

export interface RepositoryAnalyzer {
  analyzeRepository(rootPath: string): Promise<AnalysisResult>;
  analyzeDependencies(filePath: string): Promise<DependencyRelation[]>;
  getCodeMetrics(filePath: string): Promise<CodeMetrics>;
  findRelatedFiles(filePath: string): Promise<string[]>;
  suggestRefactoring(filePath: string): Promise<string[]>;
}

class RepositoryAnalyzerService implements RepositoryAnalyzer {
  
  async analyzeRepository(rootPath: string): Promise<AnalysisResult> {
    try {
      const result = await invoke<AnalysisResult>('analyze_repository', {
        rootPath
      });
      return result;
    } catch (error) {
      console.error('Repository analysis failed:', error);
      throw new Error(`Failed to analyze repository: ${error}`);
    }
  }

  async analyzeDependencies(filePath: string): Promise<DependencyRelation[]> {
    try {
      const dependencies = await invoke<DependencyRelation[]>('analyze_file_dependencies', {
        filePath
      });
      return dependencies;
    } catch (error) {
      console.error('Dependency analysis failed:', error);
      return [];
    }
  }

  async getCodeMetrics(filePath: string): Promise<CodeMetrics> {
    try {
      const metrics = await invoke<CodeMetrics>('get_code_metrics', {
        filePath
      });
      return metrics;
    } catch (error) {
      console.error('Metrics analysis failed:', error);
      throw new Error(`Failed to get code metrics: ${error}`);
    }
  }

  async findRelatedFiles(filePath: string): Promise<string[]> {
    try {
      const relatedFiles = await invoke<string[]>('find_related_files', {
        filePath
      });
      return relatedFiles;
    } catch (error) {
      console.error('Related files search failed:', error);
      return [];
    }
  }

  async suggestRefactoring(filePath: string): Promise<string[]> {
    try {
      const suggestions = await invoke<string[]>('suggest_refactoring', {
        filePath
      });
      return suggestions;
    } catch (error) {
      console.error('Refactoring suggestions failed:', error);
      return [];
    }
  }

  // Additional methods for comprehensive analysis
  async getFileStructure(rootPath: string): Promise<FileNode> {
    try {
      const structure = await invoke<FileNode>('get_file_structure', {
        rootPath
      });
      return structure;
    } catch (error) {
      console.error('File structure analysis failed:', error);
      throw new Error(`Failed to get file structure: ${error}`);
    }
  }

  async analyzeComplexity(filePath: string): Promise<{ complexity: number; hotspots: string[] }> {
    try {
      const analysis = await invoke<{ complexity: number; hotspots: string[] }>('analyze_complexity', {
        filePath
      });
      return analysis;
    } catch (error) {
      console.error('Complexity analysis failed:', error);
      return { complexity: 0, hotspots: [] };
    }
  }

  async detectCodeSmells(filePath: string): Promise<string[]> {
    try {
      const codeSmells = await invoke<string[]>('detect_code_smells', {
        filePath
      });
      return codeSmells;
    } catch (error) {
      console.error('Code smell detection failed:', error);
      return [];
    }
  }

  async generateImpactAnalysis(filePath: string): Promise<{
    affected_files: string[];
    risk_level: 'low' | 'medium' | 'high';
    recommendations: string[];
  }> {
    try {
      const impact = await invoke<{
        affected_files: string[];
        risk_level: 'low' | 'medium' | 'high';
        recommendations: string[];
      }>('generate_impact_analysis', {
        filePath
      });
      return impact;
    } catch (error) {
      console.error('Impact analysis failed:', error);
      return {
        affected_files: [],
        risk_level: 'low',
        recommendations: []
      };
    }
  }
}

// Singleton instance
export const repositoryAnalyzer = new RepositoryAnalyzerService();

// AI-Enhanced Analysis Functions
export async function aiEnhancedAnalysis(
  filePath: string,
  analysisType: 'refactoring' | 'optimization' | 'testing' | 'security'
): Promise<{
  analysis: string;
  suggestions: string[];
  code_examples?: string[];
}> {
  try {
    const result = await invoke<{
      analysis: string;
      suggestions: string[];
      code_examples?: string[];
    }>('ai_enhanced_analysis', {
      filePath,
      analysisType
    });
    return result;
  } catch (error) {
    console.error('AI-enhanced analysis failed:', error);
    return {
      analysis: 'Analysis failed',
      suggestions: [],
      code_examples: []
    };
  }
}

export async function generateDocumentation(filePath: string): Promise<string> {
  try {
    const documentation = await invoke<string>('generate_documentation', {
      filePath
    });
    return documentation;
  } catch (error) {
    console.error('Documentation generation failed:', error);
    return '';
  }
}

export async function analyzeTestCoverage(projectPath: string): Promise<{
  overall_coverage: number;
  file_coverage: { [filePath: string]: number };
  uncovered_lines: { [filePath: string]: number[] };
}> {
  try {
    const coverage = await invoke<{
      overall_coverage: number;
      file_coverage: { [filePath: string]: number };
      uncovered_lines: { [filePath: string]: number[] };
    }>('analyze_test_coverage', {
      projectPath
    });
    return coverage;
  } catch (error) {
    console.error('Test coverage analysis failed:', error);
    return {
      overall_coverage: 0,
      file_coverage: {},
      uncovered_lines: {}
    };
  }
}
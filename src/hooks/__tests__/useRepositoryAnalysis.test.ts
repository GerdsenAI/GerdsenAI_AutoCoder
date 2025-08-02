import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useRepositoryAnalysis } from '../useRepositoryAnalysis';
import { invoke } from '@tauri-apps/api/core';

// Socratic testing principle: Test what users experience, not implementation details
// This test validates that the hook provides the interface users need

vi.mock('@tauri-apps/api/core');

describe('useRepositoryAnalysis - User Experience Tests', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Setup default successful responses
    mockInvoke.mockImplementation(async (command: string) => {
      switch (command) {
        case 'analyze_repository':
          return {
            file_structure: {
              path: '/test/project',
              name: 'project',
              type: 'directory',
              children: [
                {
                  path: '/test/project/src/index.ts',
                  name: 'index.ts',
                  type: 'file',
                  language: 'typescript'
                }
              ]
            },
            dependencies: [
              {
                from: '/test/project/src/index.ts',
                to: '/test/project/src/utils.ts',
                type: 'import',
                line_number: 1
              }
            ],
            metrics: [
              {
                file_path: '/test/project/src/index.ts',
                lines_of_code: 50,
                complexity_score: 3,
                maintainability_index: 75,
                dependencies_count: 1,
                dependents_count: 0
              }
            ],
            hotspots: [],
            recommendations: []
          };
        case 'analyze_file_dependencies':
          return [
            {
              from: '/test/project/src/index.ts',
              to: '/test/project/src/utils.ts',
              type: 'import',
              line_number: 1
            }
          ];
        case 'get_code_metrics':
          return {
            file_path: '/test/project/src/index.ts',
            lines_of_code: 50,
            complexity_score: 3,
            maintainability_index: 75,
            dependencies_count: 1,
            dependents_count: 0
          };
        case 'ai_enhanced_analysis':
          return {
            analysis: 'This file could benefit from refactoring',
            suggestions: ['Extract common functions', 'Add error handling'],
            code_examples: ['function example() { return true; }']
          };
        case 'generate_documentation':
          return 'Generated documentation for the file';
        case 'analyze_test_coverage':
          return {
            overall_coverage: 85,
            file_coverage: {
              '/test/project/src/index.ts': 90
            },
            uncovered_lines: {
              '/test/project/src/index.ts': [15, 25]
            }
          };
        default:
          return null;
      }
    });
  });

  describe('Essential hook interface is available', () => {
    it('provides core state and actions users need', () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Users can access essential state
      expect(typeof result.current.loading).toBe('boolean');
      expect(result.current.error).toBeNull();
      expect(result.current.currentPath).toBe('');
      expect(result.current.selectedFile).toBe('');
      
      // Users can access analysis results
      expect(result.current.analysisResult).toBeNull();
      expect(result.current.fileStructure).toBeNull();
      expect(Array.isArray(result.current.dependencies)).toBe(true);
      expect(Array.isArray(result.current.metrics)).toBe(true);
      
      // Users can access core actions
      expect(typeof result.current.analyzeRepository).toBe('function');
      expect(typeof result.current.selectFile).toBe('function');
      expect(typeof result.current.clearError).toBe('function');
      
      // Users can access computed values
      expect(typeof result.current.hasResults).toBe('boolean');
      expect(typeof result.current.canAnalyze).toBe('boolean');
      expect(typeof result.current.isFileSelected).toBe('boolean');
    });

    it('provides AI-enhanced analysis capabilities', () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Users can access AI analysis features
      expect(result.current.aiAnalysis).toBeNull();
      expect(result.current.documentation).toBe('');
      expect(result.current.testCoverage).toBeNull();
      
      // Users can perform AI operations
      expect(typeof result.current.performAiAnalysis).toBe('function');
      expect(typeof result.current.generateFileDocumentation).toBe('function');
      expect(typeof result.current.analyzeProjectTestCoverage).toBe('function');
    });
  });

  describe('User workflow functionality works as expected', () => {
    it('allows users to analyze a repository successfully', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Initial state should indicate no analysis has been performed
      expect(result.current.hasResults).toBe(false);
      expect(result.current.canAnalyze).toBe(false);

      // User performs repository analysis
      await act(async () => {
        await result.current.analyzeRepository('/test/project');
      });

      // User should see results after analysis
      await waitFor(() => {
        expect(result.current.hasResults).toBe(true);
        expect(result.current.currentPath).toBe('/test/project');
        expect(result.current.analysisResult).not.toBeNull();
        expect(result.current.fileStructure).not.toBeNull();
        expect(result.current.dependencies.length).toBeGreaterThan(0);
        expect(result.current.metrics.length).toBeGreaterThan(0);
      });

      // Backend should have been called appropriately
      expect(mockInvoke).toHaveBeenCalledWith('analyze_repository', {
        rootPath: '/test/project'
      });
    });

    it('allows users to select and analyze individual files', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // User selects a file
      act(() => {
        result.current.selectFile('/test/project/src/index.ts');
      });

      expect(result.current.selectedFile).toBe('/test/project/src/index.ts');
      expect(result.current.isFileSelected).toBe(true);

      // User analyzes file dependencies
      await act(async () => {
        await result.current.analyzeDependencies('/test/project/src/index.ts');
      });

      // User should see dependency results
      await waitFor(() => {
        expect(result.current.dependencies.length).toBeGreaterThan(0);
      });

      expect(mockInvoke).toHaveBeenCalledWith('analyze_file_dependencies', {
        filePath: '/test/project/src/index.ts'
      });
    });

    it('enables users to perform AI-enhanced analysis', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // User selects a file
      act(() => {
        result.current.selectFile('/test/project/src/index.ts');
      });

      // User performs AI analysis
      await act(async () => {
        await result.current.performAiAnalysis('/test/project/src/index.ts', 'refactoring');
      });

      // User should see AI analysis results
      await waitFor(() => {
        expect(result.current.aiAnalysis).not.toBeNull();
        expect(result.current.aiAnalysis?.analysis).toBeTruthy();
        expect(result.current.aiAnalysis?.suggestions.length).toBeGreaterThan(0);
      });

      expect(mockInvoke).toHaveBeenCalledWith('ai_enhanced_analysis', {
        filePath: '/test/project/src/index.ts',
        analysisType: 'refactoring'
      });
    });
  });

  describe('Error handling provides appropriate user feedback', () => {
    it('handles repository analysis errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Repository not found'));
      
      const { result } = renderHook(() => useRepositoryAnalysis());

      await act(async () => {
        await result.current.analyzeRepository('/invalid/path');
      });

      // User should see error state (includes the service layer error format)
      expect(result.current.error).toBe('Failed to analyze repository: Error: Repository not found');
      expect(result.current.hasResults).toBe(false);
      expect(result.current.loading).toBe(false);

      // User can clear the error
      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });

    it('handles empty path inputs appropriately', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // User tries to analyze empty path
      await act(async () => {
        await result.current.analyzeRepository('');
      });

      // Should provide appropriate feedback
      expect(result.current.error).toBeTruthy();
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('handles service failures without breaking user experience', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Mock AI analysis to fail after the hook is created
      mockInvoke.mockRejectedValueOnce(new Error('AI service unavailable'));

      act(() => {
        result.current.selectFile('/test/project/src/index.ts');
      });

      await act(async () => {
        await result.current.performAiAnalysis('/test/project/src/index.ts', 'refactoring');
      });

      // AI service handles errors gracefully by returning default results
      // User should see some analysis result, hook should remain functional
      expect(result.current.error).toBeNull(); // AI service doesn't throw to hook
      expect(result.current.aiAnalysis).toEqual({
        analysis: 'Analysis failed',
        suggestions: [],
        code_examples: []
      });
      expect(result.current.selectedFile).toBe('/test/project/src/index.ts');
    });
  });

  describe('Hook state management works correctly', () => {
    it('manages loading states appropriately', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Initial state
      expect(result.current.loading).toBe(false);
      expect(result.current.canAnalyze).toBe(false); // No path set yet

      // After analysis
      await act(async () => {
        await result.current.analyzeRepository('/test/project');
      });

      // Should complete loading and be ready for more analysis
      expect(result.current.loading).toBe(false);
      expect(result.current.canAnalyze).toBe(true);
    });

    it('allows users to reset analysis state', async () => {
      const { result } = renderHook(() => useRepositoryAnalysis());

      // Set some state by doing an analysis first
      await act(async () => {
        await result.current.analyzeRepository('/test/project');
      });
      
      act(() => {
        result.current.selectFile('/test/file.ts');
      });

      expect(result.current.selectedFile).toBe('/test/file.ts');
      expect(result.current.currentPath).toBe('/test/project');

      // User resets analysis
      act(() => {
        result.current.resetAnalysis();
      });

      // State should be cleared
      expect(result.current.selectedFile).toBe('');
      expect(result.current.analysisResult).toBeNull();
      expect(result.current.currentPath).toBe('');
      expect(result.current.error).toBeNull();
    });
  });

  describe('Hook responds to user configuration', () => {
    it('accepts error callback configuration', async () => {
      const onError = vi.fn();
      
      const { result } = renderHook(() => useRepositoryAnalysis({ onError }));

      mockInvoke.mockRejectedValueOnce(new Error('Test error'));

      await act(async () => {
        await result.current.analyzeRepository('/test/project');
      });

      // User's error callback should be called
      expect(onError).toHaveBeenCalledWith(expect.any(Error));
      
      // Hook should still provide error state to user
      expect(result.current.error).toBeTruthy();
    });
  });
});
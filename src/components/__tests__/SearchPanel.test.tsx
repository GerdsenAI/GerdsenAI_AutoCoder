import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SearchPanel } from '../SearchPanel';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core');

// Mock SearchHealthIndicator component
vi.mock('../SearchHealthIndicator', () => ({
  default: () => <div>Search Health</div>
}));

describe('SearchPanel', () => {
  const mockInvoke = vi.mocked(invoke);
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
    // Default mock implementations
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'get_available_engines') {
        return Promise.resolve(['github', 'stackoverflow', 'google', 'duckduckgo']);
      }
      if (cmd === 'search_web') {
        return Promise.resolve({
          results: [
            {
              title: 'Test Result 1',
              url: 'https://example.com/1',
              content: 'This is a test search result',
              engine: 'google',
              score: 0.95
            }
          ]
        });
      }
      return Promise.resolve();
    });
  });

  describe('Rendering', () => {
    it('should render the search panel', async () => {
      render(<SearchPanel />);
      
      expect(screen.getByPlaceholderText('Search the web...')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /search/i })).toBeInTheDocument();
      expect(screen.getByText('Search Health')).toBeInTheDocument();
    });

    it('should load available engines on mount', async () => {
      render(<SearchPanel />);
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_available_engines');
      });
    });
  });

  describe('Search Functionality', () => {
    it('should perform search when form is submitted', async () => {
      render(<SearchPanel />);
      
      const input = screen.getByPlaceholderText('Search the web...');
      await user.type(input, 'test query');
      
      const searchButton = screen.getByRole('button', { name: /search/i });
      await user.click(searchButton);
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('search_web', {
          query: 'test query',
          engines: expect.any(Array)
        });
      });
    });

    it('should display search results', async () => {
      render(<SearchPanel />);
      
      const input = screen.getByPlaceholderText('Search the web...');
      await user.type(input, 'test query');
      
      const searchButton = screen.getByRole('button', { name: /search/i });
      await user.click(searchButton);
      
      await waitFor(() => {
        expect(screen.getByText('Test Result 1')).toBeInTheDocument();
        expect(screen.getByText('This is a test search result')).toBeInTheDocument();
      });
    });

    it('should show loading state while searching', async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'search_web') {
          return new Promise(resolve => setTimeout(resolve, 100));
        }
        return Promise.resolve(['github', 'google']);
      });

      render(<SearchPanel />);
      
      const input = screen.getByPlaceholderText('Search the web...');
      await user.type(input, 'test query');
      
      const searchButton = screen.getByRole('button', { name: /search/i });
      await user.click(searchButton);
      
      expect(screen.getByText('Searching...')).toBeInTheDocument();
    });
  });

  describe('Engine Selection', () => {
    it('should display engine checkboxes', async () => {
      render(<SearchPanel />);
      
      await waitFor(() => {
        expect(screen.getByLabelText('github')).toBeInTheDocument();
        expect(screen.getByLabelText('stackoverflow')).toBeInTheDocument();
        expect(screen.getByLabelText('google')).toBeInTheDocument();
      });
    });

    it('should toggle engine selection', async () => {
      render(<SearchPanel />);
      
      await waitFor(() => {
        const githubCheckbox = screen.getByLabelText('github');
        expect(githubCheckbox).toBeChecked();
      });
      
      const duckduckgoCheckbox = screen.getByLabelText('duckduckgo');
      expect(duckduckgoCheckbox).not.toBeChecked();
      
      await user.click(duckduckgoCheckbox);
      expect(duckduckgoCheckbox).toBeChecked();
    });
  });

  describe('Result Interaction', () => {
    it('should call onResultSelect when result is clicked', async () => {
      const onResultSelect = vi.fn();
      render(<SearchPanel onResultSelect={onResultSelect} />);
      
      // Perform search
      const input = screen.getByPlaceholderText('Search the web...');
      await user.type(input, 'test');
      await user.click(screen.getByRole('button', { name: /search/i }));
      
      await waitFor(() => {
        expect(screen.getByText('Test Result 1')).toBeInTheDocument();
      });
      
      // Click on result
      await user.click(screen.getByText('Test Result 1'));
      
      expect(onResultSelect).toHaveBeenCalledWith({
        title: 'Test Result 1',
        url: 'https://example.com/1',
        content: 'This is a test search result',
        engine: 'google',
        score: 0.95
      });
    });
  });

  describe('Error Handling', () => {
    it('should display error message on search failure', async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === 'search_web') {
          return Promise.reject(new Error('Search failed'));
        }
        return Promise.resolve(['github']);
      });

      render(<SearchPanel />);
      
      const input = screen.getByPlaceholderText('Search the web...');
      await user.type(input, 'test');
      await user.click(screen.getByRole('button', { name: /search/i }));
      
      await waitFor(() => {
        expect(screen.getByText(/error/i)).toBeInTheDocument();
      });
    });
  });
});
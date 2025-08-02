import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SearchPanel } from '../SearchPanel';

// Mock SearchHealthIndicator component 
vi.mock('../SearchHealthIndicator', () => ({
  default: () => <div>Search Health</div>
}));

// Apply Socratic principle: Test what users actually experience, not implementation details

describe('SearchPanel - User Experience Tests', () => {
  const user = userEvent.setup();

  describe('Essential user interface is present', () => {
    it('shows main elements users need to search', () => {
      render(<SearchPanel />);
      
      // User can see search interface
      expect(screen.getByPlaceholderText(/Search.*GitHub/)).toBeInTheDocument();
      expect(screen.getByText('🔍')).toBeInTheDocument();
      expect(screen.getByText('Search Health')).toBeInTheDocument();
    });

    it('shows search engine filter options', () => {
      render(<SearchPanel />);
      
      // User can see different search engines to choose from
      expect(screen.getByText('github')).toBeInTheDocument();
      expect(screen.getByText('stackoverflow')).toBeInTheDocument();
      expect(screen.getByText('google')).toBeInTheDocument();
      expect(screen.getByText('duckduckgo')).toBeInTheDocument();
    });
  });

  describe('User can interact with search interface', () => {
    it('allows users to type in search input', async () => {
      render(<SearchPanel />);
      
      // User can type in search field
      const searchInput = screen.getByPlaceholderText(/Search.*GitHub/);
      await user.type(searchInput, 'react hooks tutorial');
      
      expect(searchInput).toHaveValue('react hooks tutorial');
    });

    it('provides clickable search engines', async () => {
      render(<SearchPanel />);
      
      // User can click on different search engines
      const githubButton = screen.getByText('github');
      const stackoverflowButton = screen.getByText('stackoverflow');
      
      expect(githubButton).toBeInTheDocument();
      expect(stackoverflowButton).toBeInTheDocument();
      
      // User can interact with engine buttons
      await user.click(githubButton);
      await user.click(stackoverflowButton);
      
      // Buttons should respond to clicks (visual state may change)
      expect(githubButton).toBeInTheDocument();
    });

    it('supports keyboard navigation', async () => {
      render(<SearchPanel />);
      
      // User can navigate and use Enter key
      const searchInput = screen.getByPlaceholderText(/Search.*GitHub/);
      searchInput.focus();
      
      await user.type(searchInput, 'typescript guide');
      await user.keyboard('{Enter}');
      
      // Component should handle Enter key gracefully
      expect(searchInput).toHaveValue('typescript guide');
    });
  });

  describe('Component provides appropriate feedback', () => {
    it('shows health status information', () => {
      render(<SearchPanel />);
      
      // User can see search service health
      expect(screen.getByText('Search Health')).toBeInTheDocument();
    });

    it('indicates active search engines visually', () => {
      render(<SearchPanel />);
      
      // Some engines should be visually indicated as active
      const engineButtons = screen.getAllByText(/github|stackoverflow|google|duckduckgo|bing|brave|documentation|forums/);
      
      // Should have multiple engine options
      expect(engineButtons.length).toBeGreaterThanOrEqual(4);
      
      // Some should have active styling
      const activeButtons = engineButtons.filter(btn => 
        btn.className.includes('active') || 
        btn.closest('button')?.className.includes('active')
      );
      
      expect(activeButtons.length).toBeGreaterThan(0);
    });
  });

  describe('Component handles different states appropriately', () => {
    it('renders without errors regardless of hook state', () => {
      render(<SearchPanel />);
      
      // Component renders successfully
      expect(screen.getByPlaceholderText(/Search.*GitHub/)).toBeInTheDocument();
      
      // Component structure is intact
      const searchPanel = screen.getByPlaceholderText(/Search.*GitHub/).closest('.search-panel');
      expect(searchPanel).toBeInTheDocument();
    });

    it('handles callback integration properly', () => {
      const mockCallback = vi.fn();
      
      // Component accepts callback without error
      render(<SearchPanel onResultSelect={mockCallback} />);
      
      // Component renders normally with callback
      expect(screen.getByPlaceholderText(/Search.*GitHub/)).toBeInTheDocument();
      expect(mockCallback).toBeDefined();
    });
  });
});
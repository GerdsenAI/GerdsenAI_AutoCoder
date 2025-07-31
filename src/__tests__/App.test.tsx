import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from '../App';

// Mock child components to avoid complex setup
vi.mock('../components/ChatInterface', () => ({
  default: () => <div data-testid="chat-interface">Chat Interface</div>,
}));

vi.mock('../components/ModelSelector', () => ({
  default: () => <div data-testid="model-selector">Model Selector</div>,
}));

vi.mock('../components/RAGPanel', () => ({
  default: () => <div data-testid="rag-panel">RAG Panel</div>,
}));

vi.mock('../components/SearchPanel', () => ({
  default: () => <div data-testid="search-panel">Search Panel</div>,
}));

vi.mock('../components/HistoryPanel', () => ({
  default: () => <div data-testid="history-panel">History Panel</div>,
}));

describe('App', () => {
  it('renders the main application structure', () => {
    render(<App />);
    
    // Check for main navigation elements
    expect(screen.getByText('Chat')).toBeInTheDocument();
    expect(screen.getByText('Search')).toBeInTheDocument();
    expect(screen.getByText('RAG')).toBeInTheDocument();
    expect(screen.getByText('History')).toBeInTheDocument();
    expect(screen.getByText('Code Analysis')).toBeInTheDocument();
  });

  it('renders the header with logo and theme toggle', () => {
    render(<App />);
    
    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('🌙')).toBeInTheDocument();
  });

  it('renders the model selector', () => {
    render(<App />);
    
    expect(screen.getByTestId('model-selector')).toBeInTheDocument();
  });

  it('shows chat interface by default', () => {
    render(<App />);
    
    // Chat tab should be active by default
    const chatTab = screen.getByText('Chat').closest('.nav-tab');
    expect(chatTab).toHaveClass('active');
  });
});
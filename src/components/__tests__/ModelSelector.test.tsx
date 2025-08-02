import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { ModelSelector } from '../ModelSelector';
import { invoke } from '@tauri-apps/api/core';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('ModelSelector', () => {
  const mockOnModelSelect = vi.fn();
  const mockInvoke = vi.mocked(invoke);
  
  beforeEach(() => {
    mockOnModelSelect.mockClear();
    mockInvoke.mockClear();
  });

  it('renders with selected model name', async () => {
    mockInvoke.mockResolvedValueOnce(true); // check_ollama_connection
    mockInvoke.mockResolvedValueOnce([]); // list_models
    
    render(
      <ModelSelector 
        selectedModel="llama3" 
        onModelSelect={mockOnModelSelect}
      />
    );
    
    expect(screen.getByText('llama3')).toBeInTheDocument();
  });

  it('shows connection status', async () => {
    mockInvoke.mockResolvedValueOnce(true); // check_ollama_connection
    mockInvoke.mockResolvedValueOnce([]); // list_models
    
    render(
      <ModelSelector 
        selectedModel="llama3" 
        onModelSelect={mockOnModelSelect}
      />
    );
    
    // Initially shows "Checking..."
    expect(screen.getByText('Checking...')).toBeInTheDocument();
    
    // Wait for connection check to complete
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('check_ollama_connection');
    });
  });

  it('renders dropdown arrow', () => {
    mockInvoke.mockResolvedValueOnce(true);
    mockInvoke.mockResolvedValueOnce([]);
    
    render(
      <ModelSelector 
        selectedModel="llama3" 
        onModelSelect={mockOnModelSelect}
      />
    );
    
    expect(screen.getByText('▼')).toBeInTheDocument();
  });
});
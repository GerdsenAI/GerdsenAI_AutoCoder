import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChatInterface } from '../ChatInterface';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ChatSession, ChatMessage } from '../../types';

// Mock external dependencies
vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event');
vi.mock('../../utils/validation', () => ({
  validateMessage: vi.fn((msg: string) => ({ 
    isValid: true, 
    sanitized: msg 
  })),
  validateModelName: vi.fn((model: string) => ({ 
    isValid: true, 
    sanitized: model 
  }))
}));

// Mock syntax highlighter to avoid complex setup
vi.mock('react-syntax-highlighter', () => ({
  Prism: ({ children }: { children: string }) => <pre>{children}</pre>
}));

vi.mock('react-syntax-highlighter/dist/esm/styles/prism', () => ({
  coldarkDark: {}
}));

// Mock context manager hook
vi.mock('../../hooks/useContextManager', () => ({
  useContextManager: vi.fn(() => ({
    budget: {
      total: 128000,
      used: 1000,
      available: 127000,
      allocations: {
        conversation: 50000,
        code_files: 40000,
        documentation: 20000,
        available: 18000
      }
    },
    files: [
      {
        path: '/test/file.ts',
        token_count: 500,
        relevance_score: 0.8,
        is_pinned: true,
        file_type: 'typescript'
      }
    ],
    settings: {
      model: 'llama3.1:8b',
      reservedTokens: 2048,
      contextStrategy: 'balanced' as const,
      autoIncludeDependencies: true,
      maxTokens: 128000
    },
    loading: false,
    error: null,
    refreshBudget: vi.fn(),
    pinFile: vi.fn(),
    unpinFile: vi.fn(),
    updateSettings: vi.fn(),
    buildContext: vi.fn().mockResolvedValue('test context'),
    calculateFileRelevance: vi.fn().mockResolvedValue(0.8)
  }))
}));

// Mock child components
vi.mock('../TokenBudgetBar', () => ({
  TokenBudgetBar: ({ budget }: any) => (
    <div data-testid="token-budget-bar">
      Total: {budget.total}, Available: {budget.available}
    </div>
  )
}));

vi.mock('../ContextFileList', () => ({
  ContextFileList: ({ includedFiles, onFilePin }: any) => (
    <div data-testid="context-file-list">
      {includedFiles.map((file: any) => (
        <div key={file.path} data-testid="context-file">
          {file.path}
          <button onClick={() => onFilePin(file.path)}>Pin</button>
        </div>
      ))}
    </div>
  )
}));

vi.mock('../ContextControls', () => ({
  ContextControls: ({ settings, onSettingsChange }: any) => (
    <div data-testid="context-controls">
      <select 
        data-testid="context-strategy-select"
        value={settings.contextStrategy}
        onChange={(e) => onSettingsChange({ contextStrategy: e.target.value })}
      >
        <option value="balanced">Balanced</option>
        <option value="code">Code</option>
      </select>
    </div>
  )
}));

describe('ChatInterface', () => {
  const mockInvoke = vi.mocked(invoke);
  const mockListen = vi.mocked(listen);
  const mockOnSendMessage = vi.fn();
  
  const mockSession: ChatSession = {
    id: 'test-session-1',
    title: 'Test Session',
    messages: [],
    model: 'llama3.1:8b',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    tags: []
  };

  beforeEach(() => {
    mockInvoke.mockClear();
    mockListen.mockClear();
    mockOnSendMessage.mockClear();
    
    // Default mock implementations
    mockInvoke.mockResolvedValue(['default', 'custom']);
    mockListen.mockResolvedValue(() => {});
    
    // Mock DOM APIs that aren't available in jsdom
    const mockWriteText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(window, 'navigator', {
      value: {
        ...window.navigator,
        clipboard: {
          writeText: mockWriteText
        }
      },
      writable: true
    });
    
    // Store reference for later use in tests
    window.mockWriteText = mockWriteText;
    mockWriteText.mockClear();

    // Mock scrollIntoView
    Element.prototype.scrollIntoView = vi.fn();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe('Component Rendering', () => {
    it('renders chat interface with empty state', () => {
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      expect(screen.getByText('How can I help you today?')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...')).toBeInTheDocument();
      expect(screen.getByLabelText('Send message')).toBeInTheDocument();
    });

    it('renders existing messages from session', () => {
      const sessionWithMessages: ChatSession = {
        ...mockSession,
        messages: [
          {
            role: 'user',
            content: 'Hello, how are you?',
            timestamp: '2024-01-01T12:00:00Z'
          },
          {
            role: 'assistant', 
            content: 'I am doing well, thank you!',
            timestamp: '2024-01-01T12:00:01Z'
          }
        ]
      };

      render(
        <ChatInterface
          session={sessionWithMessages}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );

      expect(screen.getByText('Hello, how are you?')).toBeInTheDocument();
      expect(screen.getByText('I am doing well, thank you!')).toBeInTheDocument();
    });

    it('renders control buttons', () => {
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      expect(screen.getByLabelText('Toggle RAG')).toBeInTheDocument();
      expect(screen.getByLabelText('Manage Context Window')).toBeInTheDocument();
      expect(screen.getByLabelText('Clear Chat')).toBeInTheDocument();
    });
  });

  describe('Message Sending', () => {
    it('sends a message successfully', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      const sendButton = screen.getByLabelText('Send message');
      
      await user.type(input, 'Test message');
      await user.click(sendButton);
      
      expect(mockOnSendMessage).toHaveBeenCalledWith({
        role: 'user',
        content: 'Test message',
        timestamp: expect.any(String)
      });
      
      expect(mockInvoke).toHaveBeenCalledWith('generate_stream_with_ollama', {
        model: 'llama3.1:8b',
        prompt: 'Test message',
        useRag: false,
        sessionId: 'test-session-1',
        collection: 'default'
      });
    });

    it('sends message with Enter key', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      
      await user.type(input, 'Test message{Enter}');
      
      expect(mockOnSendMessage).toHaveBeenCalled();
    });

    it('does not send message with Shift+Enter', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      
      await user.type(input, 'Test message{Shift>}{Enter}{/Shift}');
      
      expect(mockOnSendMessage).not.toHaveBeenCalled();
    });

    it('prevents sending empty messages', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const sendButton = screen.getByLabelText('Send message');
      expect(sendButton).toBeDisabled();
      
      await user.click(sendButton);
      expect(mockOnSendMessage).not.toHaveBeenCalled();
    });

    it('prevents sending messages while loading', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      const sendButton = screen.getByLabelText('Send message');
      
      // Start sending a message
      await user.type(input, 'First message');
      await user.click(sendButton);
      
      // Try to send another message while first is processing
      await user.type(input, 'Second message');
      expect(sendButton).toBeDisabled();
    });
  });

  describe('Streaming Response Handling', () => {
    it('handles streaming tokens correctly', async () => {
      const user = userEvent.setup();
      let streamListener: ((event: any) => void) | null = null;
      
      mockListen.mockImplementation((eventName: string, callback: any) => {
        if (eventName === 'ollama-stream') {
          streamListener = callback;
        }
        return Promise.resolve(() => {});
      });
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      await user.type(input, 'Test message');
      await user.click(screen.getByLabelText('Send message'));
      
      // Simulate streaming tokens
      await act(async () => {
        streamListener?.({ payload: { token: 'Hello', done: false } });
        streamListener?.({ payload: { token: ' world', done: false } });
        streamListener?.({ payload: { token: '!', done: true } });
      });
      
      await waitFor(() => {
        expect(screen.getByText('Hello world!')).toBeInTheDocument();
      });
      
      expect(mockOnSendMessage).toHaveBeenCalledWith({
        role: 'assistant',
        content: 'Hello world!',
        timestamp: expect.any(String)
      });
    });
  });

  describe('RAG Integration', () => {
    it('loads collections on mount', async () => {
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('list_chroma_collections');
      });
    });

    it('toggles RAG functionality', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const ragButton = screen.getByLabelText('Toggle RAG');
      await user.click(ragButton);
      
      expect(ragButton).toHaveClass('active');
      expect(screen.getByDisplayValue('default')).toBeInTheDocument();
    });

    it('sends messages with RAG enabled', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      // Enable RAG
      await user.click(screen.getByLabelText('Toggle RAG'));
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      await user.type(input, 'Test with RAG');
      await user.click(screen.getByLabelText('Send message'));
      
      expect(mockInvoke).toHaveBeenCalledWith('generate_stream_with_ollama', {
        model: 'llama3.1:8b',
        prompt: 'Test with RAG',
        useRag: true,
        sessionId: 'test-session-1',
        collection: 'default'
      });
    });

    it('shows RAG context indicator', async () => {
      let ragListener: ((event: any) => void) | null = null;
      
      mockListen.mockImplementation((eventName: string, callback: any) => {
        if (eventName === 'rag-context') {
          ragListener = callback;
        }
        return Promise.resolve(() => {});
      });
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      // Simulate RAG context event
      await act(async () => {
        ragListener?.({
          payload: {
            session_id: 'test-session-1',
            documents_used: 3,
            collection: 'custom'
          }
        });
      });
      
      await waitFor(() => {
        expect(screen.getByText('Using 3 documents from custom')).toBeInTheDocument();
      });
    });
  });

  describe('Context Window Management', () => {
    it('toggles context panel', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const contextButton = screen.getByLabelText('Manage Context Window');
      await user.click(contextButton);
      
      expect(screen.getByText('Context Window Management')).toBeInTheDocument();
      expect(screen.getByTestId('token-budget-bar')).toBeInTheDocument();
      expect(screen.getByTestId('context-file-list')).toBeInTheDocument();
      expect(screen.getByTestId('context-controls')).toBeInTheDocument();
    });

    it('shows pinned file count badge', () => {
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      // Should show badge with count of pinned files (1 from mock)
      expect(screen.getByText('1')).toBeInTheDocument();
    });

    it('closes context panel', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      // Open panel
      await user.click(screen.getByLabelText('Manage Context Window'));
      expect(screen.getByText('Context Window Management')).toBeInTheDocument();
      
      // Close panel
      await user.click(screen.getByLabelText('Close Context Panel'));
      
      await waitFor(() => {
        expect(screen.queryByText('Context Window Management')).not.toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('handles message generation errors', async () => {
      const user = userEvent.setup();
      
      // Mock the first invoke call (list_chroma_collections) to succeed
      mockInvoke
        .mockResolvedValueOnce(['default', 'custom']) // list_chroma_collections
        .mockRejectedValueOnce(new Error('Network failure')); // generate_stream_with_ollama
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      await user.type(input, 'Test message');
      await user.click(screen.getByLabelText('Send message'));
      
      await waitFor(() => {
        expect(screen.getByText(/Sorry, I encountered an error/)).toBeInTheDocument();
      }, { timeout: 3000 });
      
      expect(mockOnSendMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          role: 'assistant',
          content: expect.stringContaining('Network failure')
        })
      );
    });

    it('handles collection loading errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('ChromaDB unavailable'));
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('list_chroma_collections');
      });
      
      // Should still render with default collection after error
      const user = userEvent.setup();
      await user.click(screen.getByLabelText('Toggle RAG'));
      
      await waitFor(() => {
        expect(screen.getByDisplayValue('default')).toBeInTheDocument();
      });
    });
  });

  describe('Code Block Rendering', () => {
    it('renders code blocks with copy functionality', async () => {
      const sessionWithCode: ChatSession = {
        ...mockSession,
        messages: [
          {
            role: 'assistant',
            content: 'Here is some code:\n```typescript\nconst x = 5;\nconsole.log(x);\n```',
            timestamp: '2024-01-01T12:00:00Z'
          }
        ]
      };

      render(
        <ChatInterface
          session={sessionWithCode}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );

      // Check that code is rendered in the code block container
      const codeBlock = document.querySelector('.code-block pre');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock?.textContent).toContain('const x = 5;');
      expect(screen.getByText('typescript')).toBeInTheDocument();
      
      // Check that copy button exists
      const copyButton = screen.getByText('Copy');
      expect(copyButton).toBeInTheDocument();
      expect(copyButton).toHaveAttribute('aria-label', 'Copy code to clipboard');
      
      // Verify the code block structure is properly rendered
      expect(document.querySelector('.code-block-header')).toBeInTheDocument();
      expect(document.querySelector('.code-block')).toBeInTheDocument();
    });
  });

  describe('Chat Management', () => {
    it('clears chat messages', async () => {
      const user = userEvent.setup();
      const sessionWithMessages: ChatSession = {
        ...mockSession,
        messages: [
          { role: 'user', content: 'Hello', timestamp: '2024-01-01T12:00:00Z' },
          { role: 'assistant', content: 'Hi there!', timestamp: '2024-01-01T12:00:01Z' }
        ]
      };

      render(
        <ChatInterface
          session={sessionWithMessages}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );

      expect(screen.getByText('Hello')).toBeInTheDocument();
      expect(screen.getByText('Hi there!')).toBeInTheDocument();

      await user.click(screen.getByLabelText('Clear Chat'));

      await waitFor(() => {
        expect(screen.queryByText('Hello')).not.toBeInTheDocument();
        expect(screen.queryByText('Hi there!')).not.toBeInTheDocument();
        expect(screen.getByText('How can I help you today?')).toBeInTheDocument();
      });
    });

    it('disables clear button when no messages', () => {
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      expect(screen.getByLabelText('Clear Chat')).toBeDisabled();
    });
  });

  describe('Suggested Prompts', () => {
    it('sets input value when suggestion is clicked', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const suggestion = screen.getByText('Explain quantum entanglement');
      await user.click(suggestion);
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...') as HTMLTextAreaElement;
      expect(input.value).toBe('Explain the concept of quantum entanglement.');
    });
  });

  describe('Loading States', () => {
    it('shows loading indicator while generating response', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      await user.type(input, 'Test message');
      await user.click(screen.getByLabelText('Send message'));
      
      expect(screen.getByText('GerdsenAI Socrates is thinking')).toBeInTheDocument();
      
      // Check for loading dots container instead of individual dots
      const loadingDots = document.querySelector('.loading-dots');
      expect(loadingDots).toBeInTheDocument();
    });

    it('disables controls while loading', async () => {
      const user = userEvent.setup();
      
      render(
        <ChatInterface 
          session={mockSession}
          model="llama3.1:8b"
          onSendMessage={mockOnSendMessage}
        />
      );
      
      const input = screen.getByPlaceholderText('Ask me to debug, explain code, or generate solutions...');
      await user.type(input, 'Test message');
      await user.click(screen.getByLabelText('Send message'));
      
      expect(screen.getByLabelText('Toggle RAG')).toBeDisabled();
      expect(screen.getByLabelText('Manage Context Window')).toBeDisabled();
      expect(input).toBeDisabled();
    });
  });
});
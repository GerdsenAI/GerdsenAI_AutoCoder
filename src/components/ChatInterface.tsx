import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
// Fix: Use the correct import path for version 15.x
import { coldarkDark } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { ChatMessage, ChatSession, ISODateString } from '../types';
import { validateMessage, validateModelName } from '../utils/validation';
import { useContextManager } from '../hooks/useContextManager';
import { TokenBudgetBar } from './TokenBudgetBar';
import { ContextFileList } from './ContextFileList';
import { ContextControls } from './ContextControls';
import './ChatInterface.css';

// Utility function removed to fix build error

interface StreamEvent {
  token: string;
  done: boolean;
  context?: number[];
}

interface RAGContextEvent {
  session_id: string;
  documents_used: number;
  collection: string;
}

interface AnalysisMode {
  mode: 'standard' | 'socratic' | 'systematic';
  maxRounds?: number;
  saveToRAG?: boolean;
  timeLimit?: number; // seconds
}

interface DeepAnalysisResult {
  solution: string;
  reasoning: QuestionAnswerChain[];
  confidence: number;
  savedToRAG: boolean;
}

interface QuestionAnswerChain {
  question: string;
  answer: string;
  round: number;
  timestamp: string;
}

export interface ChatInterfaceProps {
  session: ChatSession;
  model: string;
  onSendMessage: (message: ChatMessage) => void;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({
  session,
  model,
  onSendMessage,
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>(session.messages || []);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const streamingMessageRef = useRef(''); // Use ref for streaming message
  const [showCopySuccess, setShowCopySuccess] = useState<boolean>(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const [ragEnabled, setRagEnabled] = useState(false);
  const [selectedCollection, setSelectedCollection] = useState('default');
  const [collections, setCollections] = useState<string[]>([]);
  const [ragContextInfo, setRagContextInfo] = useState<RAGContextEvent | null>(null);
  const ragListenerRef = useRef<(() => void) | null>(null);
  
  // Context Window Management
  const [showContextPanel, setShowContextPanel] = useState(false);
  const contextManager = useContextManager();
  
  // Deep Analysis Mode
  const [analysisMode, setAnalysisMode] = useState<'standard' | 'socratic' | 'systematic'>('standard');
  const [maxRounds, setMaxRounds] = useState(5);
  const [saveToRAG, setSaveToRAG] = useState(true);
  const [showAnalysisSettings, setShowAnalysisSettings] = useState(false);
  
  // MCP Tools Integration
  const [mcpTools, setMcpTools] = useState<Array<{serverId: string, tool: any}>>([]);
  const [showMcpTools, setShowMcpTools] = useState(false);
  const [selectedMcpTool, setSelectedMcpTool] = useState<{serverId: string, toolName: string} | null>(null);

  // Update messages when session changes
  useEffect(() => {
    setMessages(session.messages || []);
  }, [session]);

  // Load available collections and MCP tools when component mounts
  useEffect(() => {
    loadCollections();
    loadMcpTools();
  }, []);

  // Set up RAG context event listener
  useEffect(() => {
    const setupRAGListener = async () => {
      if (ragListenerRef.current) {
        ragListenerRef.current();
      }
      
      ragListenerRef.current = await listen<RAGContextEvent>('rag-context', (event) => {
        if (event.payload.session_id === session.id) {
          setRagContextInfo(event.payload);
          // Clear after 5 seconds
          setTimeout(() => setRagContextInfo(null), 5000);
        }
      });
    };

    setupRAGListener();

    return () => {
      if (ragListenerRef.current) {
        ragListenerRef.current();
      }
    };
  }, [session.id]);

  const loadCollections = async () => {
    try {
      const result = await invoke<string[]>('list_chroma_collections');
      setCollections(result);
    } catch (error) {
      console.error('Failed to load collections:', error);
      setCollections(['default']);
    }
  };

  const loadMcpTools = async () => {
    try {
      const tools = await invoke<Array<[string, any]>>('list_mcp_tools');
      const formattedTools = tools.map(([serverId, tool]) => ({ serverId, tool }));
      setMcpTools(formattedTools);
    } catch (error) {
      console.error('Failed to load MCP tools:', error);
      setMcpTools([]);
    }
  };

  const callMcpTool = async (serverId: string, toolName: string, args: any) => {
    try {
      const result = await invoke('call_mcp_tool', {
        serverId,
        toolName,
        arguments: args
      });
      
      // Add the tool result as a system message
      const toolMessage: ChatMessage = {
        role: 'assistant',
        content: `🔧 **MCP Tool Result** (${toolName} from ${serverId})\n\n${JSON.stringify(result, null, 2)}`,
        timestamp: new Date().toISOString()
      };
      
      const updatedMessages = [...messages, toolMessage];
      setMessages(updatedMessages);
      onSendMessage(toolMessage);
      
      return result;
    } catch (error) {
      console.error('Failed to call MCP tool:', error);
      
      // Add error message
      const errorMessage: ChatMessage = {
        role: 'assistant',
        content: `❌ **MCP Tool Error** (${toolName} from ${serverId})\n\nError: ${error}`,
        timestamp: new Date().toISOString()
      };
      
      const updatedMessages = [...messages, errorMessage];
      setMessages(updatedMessages);
      onSendMessage(errorMessage);
      
      throw error;
    }
  };

  // Auto-resize textarea as content grows
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [inputValue]);

  // Scroll to bottom when messages change
  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessageRef.current]); // Depend on ref.current

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  const handleSendMessage = useCallback(async () => {
    if (inputValue.trim() === '' || isLoading) return;

    // Validate message content
    const messageValidation = validateMessage(inputValue);
    if (!messageValidation.isValid) {
      console.error('Message validation failed:', messageValidation.error);
      alert(`Invalid message: ${messageValidation.error}`);
      return;
    }

    // Validate model name
    const modelValidation = validateModelName(model);
    if (!modelValidation.isValid) {
      console.error('Model validation failed:', modelValidation.error);
      alert(`Invalid model: ${modelValidation.error}`);
      return;
    }

    const userMessage: ChatMessage = {
      role: 'user',
      content: messageValidation.sanitized!,
      timestamp: new Date().toISOString()
    };

    const updatedMessages = [...messages, userMessage];
    setMessages(updatedMessages);
    setInputValue('');
    setIsLoading(true);
    streamingMessageRef.current = ''; // Reset ref

    try {
      onSendMessage(userMessage);

      unlistenRef.current = await listen<StreamEvent>('ollama-stream', (event) => {
        streamingMessageRef.current += event.payload.token;
        // Force re-render to show streaming content
        setMessages((prev) => [...prev.slice(0, prev.length - 1), { ...prev[prev.length - 1], content: streamingMessageRef.current }]);

        if (event.payload.done) {
          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: streamingMessageRef.current,
            timestamp: new Date().toISOString()
          };
          const finalMessages = [...updatedMessages, assistantMessage];
          setMessages(finalMessages);
          streamingMessageRef.current = '';
          onSendMessage(assistantMessage);
          setIsLoading(false);
          if (unlistenRef.current) {
            unlistenRef.current();
            unlistenRef.current = null;
          }
        }
      });

      await invoke('generate_stream_with_ollama', {
        model: model,
        prompt: userMessage.content,
        useRag: ragEnabled,
        sessionId: session.id,
        collection: selectedCollection,
        analysisMode: analysisMode,
        maxRounds: maxRounds,
        saveToRAG: saveToRAG
      });

    } catch (error) {
      console.error('Error generating response:', error);
      const errorMessage: ChatMessage = {
        role: 'assistant',
        content: `Sorry, I encountered an error while generating a response: ${error}. Please check your connection to Ollama.`,
        timestamp: new Date().toISOString()
      };
      const errorMessages = [...updatedMessages, errorMessage];
      setMessages(errorMessages);
      onSendMessage(errorMessage);
      setIsLoading(false);
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    }
  }, [inputValue, isLoading, messages, session, model, ragEnabled, onSendMessage]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInputValue(e.target.value);
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  }, [handleSendMessage]);

  const handleClearChat = useCallback(() => {
    setMessages([]);
  }, []);

  const handleCopyCode = useCallback(async (code: string) => {
    try {
      await navigator.clipboard.writeText(code);
      setShowCopySuccess(true);
      setTimeout(() => setShowCopySuccess(false), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  }, []);

  const formatTime = useCallback((isoString: ISODateString) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }, []);

  const renderMessageContent = useCallback((content: string) => {
    const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    codeBlockRegex.lastIndex = 0;

    while ((match = codeBlockRegex.exec(content)) !== null) {
      if (match.index > lastIndex) {
        parts.push(
          <span key={`text-${lastIndex}`} className="message-text">
            {content.substring(lastIndex, match.index)}
          </span>
        );
      }

      const language = match[1] || 'plaintext';
      parts.push(
        <div key={`code-${match.index}`} className="code-block">
          <div className="code-block-header">
            <span>{language}</span>
            <button
              className="copy-button"
              onClick={() => handleCopyCode(match![2])}
              aria-label="Copy code to clipboard"
            >
              {showCopySuccess ? 'Copied!' : 'Copy'}
            </button>
          </div>
          <SyntaxHighlighter language={language} style={coldarkDark} customStyle={{ background: 'transparent' }}>
            {match[2]}
          </SyntaxHighlighter>
        </div>
      );

      lastIndex = match.index + match[0].length;
    }

    if (lastIndex < content.length) {
      parts.push(
        <span key={`text-${lastIndex}`} className="message-text">
          {content.substring(lastIndex)}
        </span>
      );
    }

    return parts.length > 0 ? parts : <span className="message-text">{content}</span>;
  }, [handleCopyCode, showCopySuccess]);

  const combinedMessages = useMemo(() => {
    const currentStreamingMessage: ChatMessage | null = streamingMessageRef.current ? {
      role: 'assistant',
      content: streamingMessageRef.current,
      timestamp: new Date().toISOString()
    } : null;

    return currentStreamingMessage ? [...messages, currentStreamingMessage] : messages;
  }, [messages, streamingMessageRef.current]);

  return (
    <div className="chat-interface">
      <div className="messages-area">
        {combinedMessages.length === 0 && !isLoading && (
          <div className="empty-chat-state">
            <img src="/assets/gerdsenai/s-logo.png" alt="GerdsenAI Socrates" className="cse-icon" />
            <p>How can I help you today?</p>
            <div className="suggested-prompts">
              <button onClick={() => setInputValue('Explain the concept of quantum entanglement.')}>Explain quantum entanglement</button>
              <button onClick={() => setInputValue('Write a Python function to reverse a string.')}>Write a Python function to reverse a string</button>
              <button onClick={() => setInputValue('Debug this JavaScript code: function sum(a,b){return a+b}')}>Debug this JavaScript code</button>
            </div>
          </div>
        )}

        {combinedMessages.map((message, index) => (
          <div
            key={index}
            className={`message ${message.role === 'user' ? 'user-message' : 'assistant-message'}`}
          >
            <div className={`message-avatar ${message.role === 'user' ? 'user-avatar' : 'ai-avatar'}`}>
              {message.role === 'user' ? 'U' : 'AI'}
            </div>
            <div className="message-content">
              <div className="message-header">
                <span className="message-author">
                  {message.role === 'user' ? 'You' : session.model}
                </span>
                <span className="message-time">{formatTime(message.timestamp)}</span>
              </div>
              <div className="message-body">{renderMessageContent(message.content)}</div>
            </div>
          </div>
        ))}

        {isLoading && streamingMessageRef.current === '' && (
          <div className="loading-indicator">
            <span>GerdsenAI Socrates is thinking</span>
            <span className="loading-dots">
              <span>.</span>
              <span>.</span>
              <span>.</span>
            </span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="input-area">
        {ragContextInfo && (
          <div className="rag-context-indicator">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="4" rx="1"/>
              <rect x="3" y="10" width="18" height="4" rx="1"/>
              <rect x="3" y="17" width="18" height="4" rx="1"/>
            </svg>
            <span>Using {ragContextInfo.documents_used} documents from {ragContextInfo.collection}</span>
          </div>
        )}
        
        {/* Context Window Management Panel */}
        {showContextPanel && (
          <div className="context-management-panel">
            <div className="context-panel-header">
              <h3>Context Window Management</h3>
              <button 
                className="close-panel-button"
                onClick={() => setShowContextPanel(false)}
                aria-label="Close Context Panel"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18"/>
                  <path d="M6 6l12 12"/>
                </svg>
              </button>
            </div>
            
            {contextManager.error && (
              <div className="context-error">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
                  <line x1="12" y1="9" x2="12" y2="13"/>
                  <line x1="12" y1="17" x2="12.01" y2="17"/>
                </svg>
                <span>{contextManager.error}</span>
              </div>
            )}
            
            <div className="context-panel-content">
              {/* Token Budget Visualization */}
              {contextManager.budget && (
                <div className="context-section">
                  <TokenBudgetBar 
                    budget={contextManager.budget} 
                    className="context-budget-bar"
                  />
                </div>
              )}
              
              {/* Context Configuration */}
              <div className="context-section">
                <ContextControls
                  settings={contextManager.settings}
                  onSettingsChange={contextManager.updateSettings}
                />
              </div>
              
              {/* File Management */}
              <div className="context-section">
                <ContextFileList
                  includedFiles={contextManager.files.filter(f => f.is_pinned)}
                  suggestedFiles={contextManager.files.filter(f => !f.is_pinned)}
                  availableTokens={contextManager.budget?.available || 0}
                  onFilePin={contextManager.pinFile}
                  onFileUnpin={contextManager.unpinFile}
                  onFileInclude={contextManager.pinFile}
                  onFileRemove={contextManager.unpinFile}
                />
              </div>
            </div>
          </div>
        )}
        
        {/* MCP Tools Panel */}
        {showMcpTools && (
          <div className="mcp-tools-panel">
            <div className="mcp-panel-header">
              <h3>MCP Tools</h3>
              <button 
                className="close-panel-button"
                onClick={() => setShowMcpTools(false)}
                aria-label="Close MCP Tools Panel"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18"/>
                  <path d="M6 6l12 12"/>
                </svg>
              </button>
            </div>
            
            <div className="mcp-panel-content">
              {mcpTools.length === 0 ? (
                <div className="no-tools">
                  <p>No MCP tools available. Configure MCP servers in Settings to add tools.</p>
                </div>
              ) : (
                <div className="tools-list">
                  {mcpTools.map(({ serverId, tool }, index) => (
                    <div key={`${serverId}-${tool.name}-${index}`} className="tool-item">
                      <div className="tool-info">
                        <h4>{tool.name}</h4>
                        <p className="tool-server">from {serverId}</p>
                        {tool.description && (
                          <p className="tool-description">{tool.description}</p>
                        )}
                      </div>
                      <button
                        className="use-tool-button"
                        onClick={() => {
                          // For now, call tool with empty args
                          // In a full implementation, you'd show a form for tool arguments
                          callMcpTool(serverId, tool.name, {});
                        }}
                        disabled={isLoading}
                      >
                        Use Tool
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
        
        <div className="input-controls">
          {/* Analysis Mode Selector */}
          <div className="analysis-mode-container">
            <button
              className={`control-button ${analysisMode !== 'standard' ? 'active' : ''}`}
              title="Deep Analysis Mode"
              aria-label="Deep Analysis Mode"
              disabled={isLoading}
              onClick={() => setShowAnalysisSettings(!showAnalysisSettings)}
            >
              <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M12 1v6m0 6v6"/>
                <path d="m9 9 3-3 3 3"/>
                <path d="m9 15 3 3 3-3"/>
              </svg>
              {analysisMode !== 'standard' && <span className="badge">{analysisMode.toUpperCase()}</span>}
            </button>
            
            {showAnalysisSettings && (
              <div className="analysis-settings-panel">
                <div className="analysis-mode-selector">
                  <label>Analysis Mode:</label>
                  <select
                    value={analysisMode}
                    onChange={(e) => setAnalysisMode(e.target.value as 'standard' | 'socratic' | 'systematic')}
                    disabled={isLoading}
                  >
                    <option value="standard">Standard - Direct Answer</option>
                    <option value="socratic">Socratic - Guided Questions</option>
                    <option value="systematic">Systematic - PDCA/OODA Loop</option>
                  </select>
                </div>
                
                {analysisMode !== 'standard' && (
                  <>
                    <div className="analysis-rounds">
                      <label>Max Question Rounds:</label>
                      <input
                        type="number"
                        min="1"
                        max="10"
                        value={maxRounds}
                        onChange={(e) => setMaxRounds(parseInt(e.target.value) || 5)}
                        disabled={isLoading}
                      />
                    </div>
                    
                    <div className="analysis-rag">
                      <label>
                        <input
                          type="checkbox"
                          checked={saveToRAG}
                          onChange={(e) => setSaveToRAG(e.target.checked)}
                          disabled={isLoading}
                        />
                        Save reasoning patterns to RAG
                      </label>
                    </div>
                  </>
                )}
              </div>
            )}
          </div>
          
          <button
            className={`control-button ${ragEnabled ? 'active' : ''}`}
            title="Toggle RAG"
            aria-label="Toggle RAG"
            disabled={isLoading}
            onClick={() => setRagEnabled(!ragEnabled)}
          >
            <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="4" rx="1"/>
              <rect x="3" y="10" width="18" height="4" rx="1"/>
              <rect x="3" y="17" width="18" height="4" rx="1"/>
            </svg>
            {ragEnabled && <span className="badge">RAG</span>}
          </button>
          
          {ragEnabled && collections.length > 0 && (
            <select
              className="collection-selector"
              value={selectedCollection}
              onChange={(e) => setSelectedCollection(e.target.value)}
              disabled={isLoading}
              title="Select RAG collection"
            >
              {collections.map((collection) => (
                <option key={collection} value={collection}>
                  {collection}
                </option>
              ))}
            </select>
          )}
          <button
            className={`control-button ${showContextPanel ? 'active' : ''}`}
            title="Manage Context Window"
            aria-label="Manage Context Window"
            disabled={isLoading}
            onClick={() => setShowContextPanel(!showContextPanel)}
          >
            <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <path d="M14 2v6h6"/>
              <path d="M16 13H8"/>
              <path d="M16 17H8"/>
              <path d="M10 9H8"/>
            </svg>
            {contextManager.files.filter(f => f.is_pinned).length > 0 && (
              <span className="badge">{contextManager.files.filter(f => f.is_pinned).length}</span>
            )}
            {showContextPanel && <span className="badge">CONTEXT</span>}
          </button>
          <button
            className={`control-button ${showMcpTools ? 'active' : ''}`}
            title="MCP Tools"
            aria-label="MCP Tools"
            disabled={isLoading}
            onClick={() => setShowMcpTools(!showMcpTools)}
          >
            <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M12 22a10 10 0 110-20 10 10 0 010 20z"/>
              <path d="M8 12l2 2 4-4"/>
              <path d="M21 12c0-1-2-3-2-3s-2 2-2 3 2 3 2 3 2-2 2-3"/>
              <path d="M3 12c0 1 2 3 2 3s2-2 2-3-2-3-2-3-2 2-2 3"/>
            </svg>
            {mcpTools.length > 0 && <span className="badge">{mcpTools.length}</span>}
            {showMcpTools && <span className="badge">TOOLS</span>}
          </button>
          <button
            className="control-button"
            title="Clear Chat"
            aria-label="Clear Chat"
            onClick={handleClearChat}
            disabled={isLoading || messages.length === 0}
          >
            <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M3 6h18"/>
              <path d="M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
              <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/>
              <path d="M10 11v6"/>
              <path d="M14 11v6"/>
            </svg>
          </button>
        </div>

        <div className="input-wrapper">
          <textarea
            ref={textareaRef}
            className="chat-input"
            value={inputValue}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder="Ask me to debug, explain code, or generate solutions..."
            rows={1}
            disabled={isLoading}
            maxLength={2000} // Example max length
          />
          <button
            className="send-button"
            onClick={handleSendMessage}
            disabled={inputValue.trim() === '' || isLoading}
            aria-label="Send message"
          >
            <svg className="icon" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M2 21l21-9L2 3v7l15 2-15 2v7z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
};

export default ChatInterface;

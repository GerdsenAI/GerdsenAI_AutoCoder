import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import ChatInterface from './components/ChatInterface';
import ModelSelector from './components/ModelSelector';
import RAGPanel from './components/RAGPanel';
import SearchPanel from './components/SearchPanel';
import HistoryPanel from './components/HistoryPanel';
import { ChatSession, ChatMessage } from './types';
import { OperationMonitor } from './components/OperationMonitor';
import { OperationTestButton } from './components/OperationTestButton';

function App() {
  const [activeTab, setActiveTab] = useState<'chat' | 'search' | 'rag' | 'history'>('chat');
  const [selectedModel, setSelectedModel] = useState<string>('llama3');
  const [chatSessions, setChatSessions] = useState<ChatSession[]>([]);
  const [currentSession, setCurrentSession] = useState<ChatSession | null>(null);
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');

  // Initialize app on mount
  useEffect(() => {
    const loadSessions = async () => {
      try {
        const sessions = await invoke<ChatSession[]>('list_chat_sessions');
        setChatSessions(sessions);
        
        // Create a new session if none exist
        if (sessions.length === 0) {
          createNewSession();
        } else {
          // Set the most recent session as current
          setCurrentSession(sessions[0]);
        }
      } catch (error) {
        console.error('Failed to load chat sessions:', error);
        createNewSession();
      }
    };

    loadSessions();
    
    // Set theme based on system preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    setTheme(prefersDark ? 'dark' : 'light');
    document.body.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
  }, []);

  const createNewSession = useCallback(async () => {
    try {
      const newSession = await invoke<ChatSession>('create_chat_session', { 
        title: 'New Chat',
        model: selectedModel 
      });
      setChatSessions(prev => [newSession, ...prev]);
      setCurrentSession(newSession);
      setActiveTab('chat');
    } catch (error) {
      console.error('Failed to create new session:', error);
    }
  }, [selectedModel]);

  const updateSession = useCallback(async (session: ChatSession) => {
    try {
      await invoke('update_chat_session', { session });
      
      // Reload sessions to get the updated data
      const sessions = await invoke<ChatSession[]>('list_chat_sessions');
      setChatSessions(sessions);
      
      if (currentSession?.id === session.id) {
        const updatedSession = sessions.find(s => s.id === session.id);
        if (updatedSession) {
          setCurrentSession(updatedSession);
        }
      }
    } catch (error) {
      console.error('Failed to update session:', error);
    }
  }, [currentSession]);

  const deleteSession = useCallback(async (sessionId: string) => {
    try {
      await invoke('delete_chat_session', { id: sessionId });
      
      setChatSessions(prev => prev.filter(s => s.id !== sessionId));
      
      if (currentSession?.id === sessionId) {
        if (chatSessions.length > 1) {
          const newCurrentSession = chatSessions.find(s => s.id !== sessionId);
          setCurrentSession(newCurrentSession || null);
        } else {
          createNewSession();
        }
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  }, [currentSession, chatSessions, createNewSession]);

  const addMessageToCurrentSession = useCallback(async (message: ChatMessage) => {
    if (!currentSession) return;
    
    try {
      const updatedSession = await invoke<ChatSession>('add_chat_message', {
        sessionId: currentSession.id,
        role: message.role,
        content: message.content
      });
      
      setChatSessions(prev => 
        prev.map(s => s.id === updatedSession.id ? updatedSession : s)
      );
      setCurrentSession(updatedSession);
      
      // Update the title based on the first user message if it's still "New Chat"
      if (currentSession.title === 'New Chat' && message.role === 'user') {
        const newTitle = message.content.substring(0, 50) + (message.content.length > 50 ? '...' : '');
        const sessionWithTitle = { ...updatedSession, title: newTitle };
        await invoke('update_chat_session', { session: sessionWithTitle });
        
        const sessions = await invoke<ChatSession[]>('list_chat_sessions');
        setChatSessions(sessions);
        const updated = sessions.find(s => s.id === sessionWithTitle.id);
        if (updated) {
          setCurrentSession(updated);
        }
      }
    } catch (error) {
      console.error('Failed to add message to session:', error);
    }
  }, [currentSession]);

  const toggleTheme = useCallback(() => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
    document.body.setAttribute('data-theme', newTheme);
  }, [theme]);

  const handleModelSelect = useCallback((model: string) => {
    setSelectedModel(model);
  }, []);

  return (
    <div className="app-container">
      <div className="header">
        <div className="logo-container">
          <img src="/assets/cse-icon-logo.png" alt="CSE Icon" className="logo-image" />
        </div>
        <div className="connection-status">
          <div className="status-indicator status-connected"></div>
          Connected
        </div>
        <button className="theme-toggle" onClick={toggleTheme}>
          {theme === 'light' ? '🌙' : '☀️'}
        </button>
      </div>
      
      <ModelSelector 
        selectedModel={selectedModel} 
        onModelSelect={handleModelSelect} 
      />
      
      <div className="nav-tabs">
        <div 
          className={`nav-tab ${activeTab === 'chat' ? 'active' : ''}`}
          onClick={() => setActiveTab('chat')}
        >
          Chat
        </div>
        <div 
          className={`nav-tab ${activeTab === 'search' ? 'active' : ''}`}
          onClick={() => setActiveTab('search')}
        >
          Search
        </div>
        <div 
          className={`nav-tab ${activeTab === 'rag' ? 'active' : ''}`}
          onClick={() => setActiveTab('rag')}
        >
          RAG
        </div>
        <div 
          className={`nav-tab ${activeTab === 'history' ? 'active' : ''}`}
          onClick={() => setActiveTab('history')}
        >
          History
        </div>
      </div>
      
      <div className="content-area">
        {activeTab === 'chat' && currentSession && (
          <>
            <ChatInterface 
              session={currentSession}
              onSendMessage={addMessageToCurrentSession}
              model={selectedModel}
            />
            <OperationTestButton />
            <OperationMonitor />
          </>
        )}
        
        {activeTab === 'search' && (
          <SearchPanel />
        )}
        
        {activeTab === 'rag' && (
          <RAGPanel />
        )}
        
        {activeTab === 'history' && (
          <HistoryPanel 
            sessions={chatSessions}
            onSelectSession={(session: ChatSession | null) => {
              setCurrentSession(session);
              if (session) {
                setActiveTab('chat');
              }
            }}
            onDeleteSession={deleteSession}
            onCreateNewSession={createNewSession}
          />
        )}
      </div>
      
      <div className="action-bar">
        <button className="action-button" title="Favorite">★</button>
        <button className="action-button" title="Search">🔍</button>
        <button className="action-button" title="New Chat" onClick={createNewSession}>+</button>
        <button className="action-button" title="Delete">🗑️</button>
        <button className="action-button" title="Share">↗️</button>
      </div>
    </div>
  );
}

export default App;

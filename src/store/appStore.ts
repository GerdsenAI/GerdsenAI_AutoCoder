import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { ChatSession } from '../types';

interface AppState {
  // UI State
  activeTab: 'chat' | 'search' | 'rag' | 'history' | 'code' | 'settings';
  codeAnalysisTab: 'dependency' | 'impact' | 'refactor';
  theme: 'light' | 'dark';

  // Chat State
  chatSessions: ChatSession[];
  currentSession: ChatSession | null;
  selectedModel: string;

  // Actions
  setActiveTab: (tab: AppState['activeTab']) => void;
  setCodeAnalysisTab: (tab: AppState['codeAnalysisTab']) => void;
  setTheme: (theme: AppState['theme']) => void;
  setChatSessions: (sessions: ChatSession[]) => void;
  setCurrentSession: (session: ChatSession | null) => void;
  setSelectedModel: (model: string) => void;
  addChatSession: (session: ChatSession) => void;
  updateChatSession: (id: string, session: ChatSession) => void;
  removeChatSession: (id: string) => void;
}

export const useAppStore = create<AppState>()(
  devtools(
    persist(
      (set) => ({
        // Initial State
        activeTab: 'chat',
        codeAnalysisTab: 'dependency',
        theme: 'dark',
        chatSessions: [],
        currentSession: null,
        selectedModel: 'llama3',

        // Actions
        setActiveTab: (tab) => set({ activeTab: tab }),
        setCodeAnalysisTab: (tab) => set({ codeAnalysisTab: tab }),
        setTheme: (theme) => {
          set({ theme });
          document.body.setAttribute('data-theme', theme);
        },
        setChatSessions: (sessions) => set({ chatSessions: sessions }),
        setCurrentSession: (session) => set({ currentSession: session }),
        setSelectedModel: (model) => set({ selectedModel: model }),
        addChatSession: (session) =>
          set((state) => ({
            chatSessions: [session, ...state.chatSessions],
          })),
        updateChatSession: (id, session) =>
          set((state) => ({
            chatSessions: state.chatSessions.map((s) =>
              s.id === id ? session : s
            ),
          })),
        removeChatSession: (id) =>
          set((state) => ({
            chatSessions: state.chatSessions.filter((s) => s.id !== id),
          })),
      }),
      {
        name: 'gerdsenai-app-storage',
        partialize: (state) => ({
          theme: state.theme,
          selectedModel: state.selectedModel,
        }),
      }
    ),
    { name: 'GerdsenAI Store' }
  )
);

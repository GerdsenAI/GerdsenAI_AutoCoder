import { invoke } from '@tauri-apps/api/core';
import { ChatSession } from '../types';

export interface ApiOptions {
  onError?: (error: Error) => void;
}

class Api {
  private options: ApiOptions;

  constructor(options: ApiOptions = {}) {
    this.options = options;
  }

  // Window management
  async createWindow(config: any): Promise<string> {
    try {
      return await invoke<string>('create_window', { config });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async closeWindow(windowLabel: string): Promise<void> {
    try {
      await invoke('close_window', { windowLabel });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async dockWindow(windowLabel: string, position: any): Promise<void> {
    try {
      await invoke('dock_window', { windowLabel, position });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async undockWindow(windowLabel: string): Promise<void> {
    try {
      await invoke('undock_window', { windowLabel });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  // Chat history
  async listChatSessions(): Promise<ChatSession[]> {
    try {
      return await invoke<ChatSession[]>('list_chat_sessions');
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async getChatSession(id: string): Promise<ChatSession> {
    try {
      return await invoke<ChatSession>('get_chat_session', { id });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async createChatSession(title: string, model: string): Promise<ChatSession> {
    try {
      return await invoke<ChatSession>('create_chat_session', { title, model });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async updateChatSession(session: ChatSession): Promise<void> {
    try {
      await invoke('update_chat_session', { session });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async deleteChatSession(id: string): Promise<void> {
    try {
      await invoke('delete_chat_session', { id });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async addChatMessage(sessionId: string, role: string, content: string): Promise<any> {
    try {
      return await invoke<any>('add_chat_message', { sessionId, role, content });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  // Documentation scraping
  async scrapeDocumentation(request: any): Promise<any> {
    try {
      return await invoke<any>('scrape_documentation', { request });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async batchScrapeDocumentation(request: any): Promise<any> {
    try {
      return await invoke<any>('batch_scrape_documentation', { request });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async searchDocumentation(request: any): Promise<any[]> {
    try {
      return await invoke<any[]>('search_documentation', { request });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async scrapeFromSearch(query: string, collectionName: string, documentType: string, maxResults: number): Promise<any> {
    try {
      return await invoke<any>('scrape_from_search', { query, collectionName, documentType, maxResults });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  // File operations
  async watchRepository(path: string): Promise<string> {
    try {
      return await invoke<string>('watch_repository', { path });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async unwatchRepository(watcherId: string): Promise<void> {
    try {
      await invoke('unwatch_repository', { watcherId });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async listFiles(path: string, patterns?: string[]): Promise<string[]> {
    try {
      return await invoke<string[]>('list_files', { path, patterns });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async readFile(path: string): Promise<string> {
    try {
      return await invoke<string>('read_file', { path });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async writeFile(path: string, content: string): Promise<void> {
    try {
      await invoke('write_file', { path, content });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  // Settings
  async getSettings(): Promise<any> {
    try {
      return await invoke<any>('get_settings');
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }

  async updateSettings(settings: any): Promise<void> {
    try {
      await invoke('update_settings', { settings });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      throw error;
    }
  }
}

export default Api;

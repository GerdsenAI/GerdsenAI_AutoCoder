import { invoke } from '@tauri-apps/api/core';

export interface LSPClientOptions {
  serverPath?: string;
  workspacePath?: string;
  onError?: (error: Error) => void;
}

export interface LSPDiagnostic {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  message: string;
  severity: 'Error' | 'Warning' | 'Information' | 'Hint';
  source: string;
  code?: string;
}

export interface LSPCompletionItem {
  label: string;
  kind: string;
  detail?: string;
  documentation?: string;
  insertText?: string;
}

export interface LSPHoverResult {
  contents: string;
  range?: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
}

export interface LSPCodeAction {
  title: string;
  kind: string;
  edit?: any;
  command?: {
    title: string;
    command: string;
    arguments?: any[];
  };
}

class LSPClient {
  private options: LSPClientOptions;
  private isConnected: boolean = false;

  constructor(options: LSPClientOptions = {}) {
    this.options = options;
  }

  async initialize(): Promise<boolean> {
    try {
      this.isConnected = await invoke<boolean>('initialize_lsp_server', {
        serverPath: this.options.serverPath,
        workspacePath: this.options.workspacePath
      });
      return this.isConnected;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return false;
    }
  }

  async shutdown(): Promise<boolean> {
    try {
      await invoke<boolean>('shutdown_lsp_server');
      this.isConnected = false;
      return true;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return false;
    }
  }

  async openDocument(uri: string, text: string, languageId: string): Promise<boolean> {
    try {
      return await invoke<boolean>('lsp_open_document', {
        uri,
        text,
        languageId
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return false;
    }
  }

  async closeDocument(uri: string): Promise<boolean> {
    try {
      return await invoke<boolean>('lsp_close_document', {
        uri
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return false;
    }
  }

  async updateDocument(uri: string, text: string): Promise<boolean> {
    try {
      return await invoke<boolean>('lsp_update_document', {
        uri,
        text
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return false;
    }
  }

  async getDiagnostics(uri: string): Promise<LSPDiagnostic[]> {
    try {
      return await invoke<LSPDiagnostic[]>('lsp_get_diagnostics', {
        uri
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return [];
    }
  }

  async getCompletions(uri: string, line: number, character: number): Promise<LSPCompletionItem[]> {
    try {
      return await invoke<LSPCompletionItem[]>('lsp_get_completions', {
        uri,
        line,
        character
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return [];
    }
  }

  async getHover(uri: string, line: number, character: number): Promise<LSPHoverResult | null> {
    try {
      return await invoke<LSPHoverResult | null>('lsp_get_hover', {
        uri,
        line,
        character
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }

  async getCodeActions(uri: string, line: number, character: number, diagnostics?: LSPDiagnostic[]): Promise<LSPCodeAction[]> {
    try {
      return await invoke<LSPCodeAction[]>('lsp_get_code_actions', {
        uri,
        line,
        character,
        diagnostics
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return [];
    }
  }

  async executeCommand(command: string, args?: any[]): Promise<any> {
    try {
      return await invoke<any>('lsp_execute_command', {
        command,
        args
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }

  async analyzeCode(code: string, language: string, filePath?: string): Promise<any> {
    try {
      return await invoke<any>('analyze_code', {
        request: {
          code,
          language,
          file_path: filePath
        }
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }

  async analyzeRepository(repoPath: string, filePatterns?: string[], excludePatterns?: string[], maxFiles?: number): Promise<any> {
    try {
      return await invoke<any>('analyze_repository', {
        request: {
          repo_path: repoPath,
          file_patterns: filePatterns,
          exclude_patterns: excludePatterns,
          max_files: maxFiles
        }
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }

  async fixCode(code: string, language: string, errorMessage: string, errorRange: any): Promise<any> {
    try {
      return await invoke<any>('fix_code', {
        request: {
          code,
          language,
          error_message: errorMessage,
          error_range: errorRange
        }
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }

  async generateCode(prompt: string, language: string, context?: string): Promise<any> {
    try {
      return await invoke<any>('generate_code', {
        request: {
          prompt,
          language,
          context
        }
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError?.(error);
      return null;
    }
  }
}

export default LSPClient;

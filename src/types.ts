// A utility type for ISO 8601 formatted date strings
export type ISODateString = string;

// Clear role enforcement for better autocomplete and safety
export type ChatRole = 'user' | 'assistant' | 'system';

// === Message === (matches backend ChatMessage)
export interface ChatMessage {
  role: string;
  content: string;
  timestamp: string;
}

// === Code Snippet === (matches backend)
export interface CodeSnippet {
  code: string;
  language: string;
  file_path?: string;
  start_line?: number;
  end_line?: number;
}

// === Context Metadata === (matches backend ChatContext)
export interface ChatContext {
  code_snippets: CodeSnippet[];
  file_paths: string[];
  repository_path?: string;
  additional_context: Record<string, string>;
}

// === Chat Session === (matches backend ChatSession)
export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  model: string;
  created_at: string;
  updated_at: string;
  tags: string[];
  context?: ChatContext;
}

// === Frontend-only types for UI state ===
export interface ChatAttachment {
  type: 'code' | 'image' | 'file';
  name: string;
  content: string; // base64 or raw text
}

export interface UIMessage extends ChatMessage {
  id?: string;
  attachments?: ChatAttachment[];
}

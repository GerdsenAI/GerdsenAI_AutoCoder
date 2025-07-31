/**
 * Input validation utilities for Auto-Coder Companion
 * Prevents XSS, injection attacks, and malformed data
 */

// Maximum lengths for various inputs
const MAX_MESSAGE_LENGTH = 10000;
const MAX_MODEL_NAME_LENGTH = 100;
const MAX_URL_LENGTH = 2048;
const MAX_FILE_PATH_LENGTH = 500;

// Regex patterns for validation
const SAFE_MODEL_NAME_PATTERN = /^[a-zA-Z0-9_\-.:]+$/;
const SAFE_URL_PATTERN = /^https?:\/\/[a-zA-Z0-9\-._~:/?#[\]@!$&'()*+,;=%]+$/;
const SAFE_FILE_PATH_PATTERN = /^[a-zA-Z0-9\-._/\\:]+$/;

export interface ValidationResult {
  isValid: boolean;
  error?: string;
  sanitized?: string;
}

/**
 * Sanitize HTML content to prevent XSS attacks
 */
export function sanitizeHtml(input: string): string {
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');
}

/**
 * Validate and sanitize chat message content
 */
export function validateMessage(message: string): ValidationResult {
  if (!message || typeof message !== 'string') {
    return { isValid: false, error: 'Message is required and must be a string' };
  }

  if (message.length > MAX_MESSAGE_LENGTH) {
    return { 
      isValid: false, 
      error: `Message too long. Maximum ${MAX_MESSAGE_LENGTH} characters` 
    };
  }

  // Remove potentially dangerous characters but preserve formatting
  const sanitized = message.trim();
  
  if (sanitized.length === 0) {
    return { isValid: false, error: 'Message cannot be empty' };
  }

  return { 
    isValid: true, 
    sanitized: sanitizeHtml(sanitized)
  };
}

/**
 * Validate model name for Ollama integration
 */
export function validateModelName(modelName: string): ValidationResult {
  if (!modelName || typeof modelName !== 'string') {
    return { isValid: false, error: 'Model name is required' };
  }

  if (modelName.length > MAX_MODEL_NAME_LENGTH) {
    return { 
      isValid: false, 
      error: `Model name too long. Maximum ${MAX_MODEL_NAME_LENGTH} characters` 
    };
  }

  if (!SAFE_MODEL_NAME_PATTERN.test(modelName)) {
    return { 
      isValid: false, 
      error: 'Model name contains invalid characters. Only alphanumeric, dash, underscore, dot, and colon allowed' 
    };
  }

  return { isValid: true, sanitized: modelName.trim() };
}

/**
 * Validate URL for external API calls
 */
export function validateUrl(url: string): ValidationResult {
  if (!url || typeof url !== 'string') {
    return { isValid: false, error: 'URL is required' };
  }

  if (url.length > MAX_URL_LENGTH) {
    return { 
      isValid: false, 
      error: `URL too long. Maximum ${MAX_URL_LENGTH} characters` 
    };
  }

  if (!SAFE_URL_PATTERN.test(url)) {
    return { 
      isValid: false, 
      error: 'Invalid URL format. Only HTTP/HTTPS URLs allowed' 
    };
  }

  // Additional security checks
  const urlObj = new URL(url);
  
  // Block dangerous protocols
  if (!['http:', 'https:'].includes(urlObj.protocol)) {
    return { isValid: false, error: 'Only HTTP and HTTPS protocols are allowed' };
  }

  // Block localhost in production (except for Ollama)
  if (urlObj.hostname === 'localhost' || urlObj.hostname === '127.0.0.1') {
    // Allow localhost only for specific ports (Ollama typically uses 11434)
    const allowedPorts = ['11434', '8080', '3000', '5173', '1420'];
    if (!allowedPorts.includes(urlObj.port)) {
      return { isValid: false, error: 'Localhost connections only allowed on specific ports' };
    }
  }

  return { isValid: true, sanitized: url.trim() };
}

/**
 * Validate file path for file operations
 */
export function validateFilePath(filePath: string): ValidationResult {
  if (!filePath || typeof filePath !== 'string') {
    return { isValid: false, error: 'File path is required' };
  }

  if (filePath.length > MAX_FILE_PATH_LENGTH) {
    return { 
      isValid: false, 
      error: `File path too long. Maximum ${MAX_FILE_PATH_LENGTH} characters` 
    };
  }

  // Basic path traversal prevention
  if (filePath.includes('..') || filePath.includes('~')) {
    return { isValid: false, error: 'Path traversal attempts are not allowed' };
  }

  // Check for valid file path characters
  if (!SAFE_FILE_PATH_PATTERN.test(filePath)) {
    return { 
      isValid: false, 
      error: 'File path contains invalid characters' 
    };
  }

  return { isValid: true, sanitized: filePath.trim() };
}

/**
 * Validate JSON data to prevent injection attacks
 */
export function validateJsonData(data: unknown): ValidationResult {
  try {
    const jsonString = JSON.stringify(data);
    
    if (jsonString.length > 100000) { // 100KB limit
      return { isValid: false, error: 'JSON data too large' };
    }

    // Check for potentially dangerous patterns
    const dangerousPatterns = [
      /__proto__/,
      /constructor/,
      /prototype/,
      /eval\(/,
      /function\(/,
      /<script/i,
      /javascript:/i
    ];

    for (const pattern of dangerousPatterns) {
      if (pattern.test(jsonString)) {
        return { isValid: false, error: 'JSON contains potentially dangerous content' };
      }
    }

    return { isValid: true, sanitized: jsonString };
  } catch (error) {
    return { isValid: false, error: 'Invalid JSON format' };
  }
}

/**
 * General purpose input sanitizer
 */
export function sanitizeInput(input: string, maxLength = 1000): string {
  if (!input || typeof input !== 'string') {
    return '';
  }

  return sanitizeHtml(input.trim().slice(0, maxLength));
}
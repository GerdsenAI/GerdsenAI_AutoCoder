import { describe, it, expect } from 'vitest';
import { validateMessage, validateModelName, sanitizeHtml } from '../validation';

describe('validation utilities', () => {
  describe('validateMessage', () => {
    it('returns valid result for valid message', () => {
      const result = validateMessage('Hello, world!');
      expect(result.isValid).toBe(true);
      expect(result.sanitized).toBe('Hello, world!');
    });

    it('returns invalid result for empty message', () => {
      const result = validateMessage('');
      expect(result.isValid).toBe(false);
      expect(result.error).toBe('Message is required and must be a string');
    });

    it('returns invalid result for whitespace-only message', () => {
      const result = validateMessage('   ');
      expect(result.isValid).toBe(false);
      expect(result.error).toBe('Message cannot be empty');
    });

    it('returns invalid result for null/undefined message', () => {
      const nullResult = validateMessage(null as any);
      expect(nullResult.isValid).toBe(false);
      expect(nullResult.error).toBe('Message is required and must be a string');

      const undefinedResult = validateMessage(undefined as any);
      expect(undefinedResult.isValid).toBe(false);
      expect(undefinedResult.error).toBe('Message is required and must be a string');
    });

    it('returns invalid result for very long messages', () => {
      const longMessage = 'a'.repeat(10001);
      const result = validateMessage(longMessage);
      expect(result.isValid).toBe(false);
      expect(result.error).toContain('Message too long');
    });
  });

  describe('validateModelName', () => {
    it('returns valid result for valid model name', () => {
      const result1 = validateModelName('llama3');
      expect(result1.isValid).toBe(true);
      expect(result1.sanitized).toBe('llama3');

      const result2 = validateModelName('llama3.1:8b');
      expect(result2.isValid).toBe(true);
      expect(result2.sanitized).toBe('llama3.1:8b');
    });

    it('returns invalid result for empty model name', () => {
      const result = validateModelName('');
      expect(result.isValid).toBe(false);
      expect(result.error).toBe('Model name is required');
    });

    it('returns invalid result for whitespace-only model name', () => {
      const result = validateModelName('   ');
      expect(result.isValid).toBe(false);
      expect(result.error).toContain('invalid characters');
    });

    it('returns invalid result for null/undefined model name', () => {
      const nullResult = validateModelName(null as any);
      expect(nullResult.isValid).toBe(false);
      expect(nullResult.error).toBe('Model name is required');

      const undefinedResult = validateModelName(undefined as any);
      expect(undefinedResult.isValid).toBe(false);
      expect(undefinedResult.error).toBe('Model name is required');
    });
  });

  describe('sanitizeHtml', () => {
    it('escapes HTML characters', () => {
      const input = '<script>alert("xss")</script>';
      const result = sanitizeHtml(input);
      expect(result).toBe('&lt;script&gt;alert(&quot;xss&quot;)&lt;&#x2F;script&gt;');
    });

    it('escapes special characters', () => {
      const input = '& < > " \' /';
      const result = sanitizeHtml(input);
      expect(result).toBe('&amp; &lt; &gt; &quot; &#x27; &#x2F;');
    });
  });
});
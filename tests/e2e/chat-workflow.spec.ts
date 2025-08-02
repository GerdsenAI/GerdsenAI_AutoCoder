import { test, expect, Page } from '@playwright/test';
import { mockOllamaResponse, mockSearchResults } from './helpers/mocks';

test.describe('Chat Workflow', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto('/');
    
    // Wait for app to load
    await page.waitForSelector('[data-testid="chat-interface"]', { timeout: 10000 });
  });

  test('should complete a basic chat interaction', async () => {
    // Setup mock for Ollama response
    await mockOllamaResponse(page, {
      model: 'gpt-4',
      response: 'Hello! I can help you with programming questions.'
    });

    // Type a message
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Hello, can you help me with TypeScript?');
    
    // Send message
    await input.press('Enter');
    
    // Wait for response
    await expect(page.locator('[data-testid="assistant-message"]')).toContainText(
      'Hello! I can help you with programming questions.',
      { timeout: 15000 }
    );
    
    // Verify message appears in history
    await expect(page.locator('[data-testid="user-message"]')).toContainText(
      'Hello, can you help me with TypeScript?'
    );
  });

  test('should handle streaming responses', async () => {
    // Setup streaming mock
    await mockOllamaResponse(page, {
      model: 'gpt-4',
      response: 'This is a longer response that will be streamed word by word to test the streaming functionality.',
      stream: true
    });

    // Send message
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Explain async/await in detail');
    await input.press('Enter');
    
    // Verify streaming indicator appears
    await expect(page.locator('[data-testid="streaming-indicator"]')).toBeVisible();
    
    // Wait for streaming to complete
    await expect(page.locator('[data-testid="streaming-indicator"]')).not.toBeVisible({ timeout: 20000 });
    
    // Verify complete response
    await expect(page.locator('[data-testid="assistant-message"]').last()).toContainText(
      'streaming functionality'
    );
  });

  test('should switch between AI models', async () => {
    // Open model selector
    await page.click('[data-testid="model-selector-button"]');
    
    // Select Claude model
    await page.click('[data-testid="model-option-claude-3"]');
    
    // Verify model changed
    await expect(page.locator('[data-testid="current-model"]')).toContainText('claude-3');
    
    // Send a message with new model
    await mockOllamaResponse(page, {
      model: 'claude-3',
      response: 'Response from Claude model'
    });
    
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Test with Claude');
    await input.press('Enter');
    
    // Verify response from correct model
    await expect(page.locator('[data-testid="assistant-message"]').last()).toContainText(
      'Response from Claude model'
    );
  });

  test('should integrate RAG documents into chat', async () => {
    // Switch to RAG tab
    await page.click('[data-testid="tab-rag"]');
    
    // Upload a document
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles('./tests/fixtures/documents/sample.txt');
    
    // Wait for upload to complete
    await expect(page.locator('[data-testid="document-list-item"]')).toHaveCount(1);
    
    // Go back to chat
    await page.click('[data-testid="tab-chat"]');
    
    // Add document to context
    await page.click('[data-testid="add-context-button"]');
    await page.click('[data-testid="document-option-sample.txt"]');
    
    // Verify document added to context
    await expect(page.locator('[data-testid="context-document"]')).toContainText('sample.txt');
    
    // Send message that uses document context
    await mockOllamaResponse(page, {
      model: 'gpt-4',
      response: 'Based on the document you provided, here is the answer...',
      context: ['sample.txt']
    });
    
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('What does the document say about testing?');
    await input.press('Enter');
    
    // Verify response uses document context
    await expect(page.locator('[data-testid="assistant-message"]').last()).toContainText(
      'Based on the document'
    );
  });

  test('should perform web search and add results to chat', async () => {
    // Switch to search tab
    await page.click('[data-testid="tab-search"]');
    
    // Mock search results
    await mockSearchResults(page, [
      {
        title: 'TypeScript Best Practices',
        url: 'https://example.com/ts-practices',
        content: 'Here are the best practices for TypeScript development...'
      }
    ]);
    
    // Perform search
    const searchInput = page.locator('[data-testid="search-input"]');
    await searchInput.fill('TypeScript best practices');
    await searchInput.press('Enter');
    
    // Wait for results
    await expect(page.locator('[data-testid="search-result"]')).toHaveCount(1);
    
    // Add result to chat
    await page.click('[data-testid="add-to-chat-button"]');
    
    // Go back to chat
    await page.click('[data-testid="tab-chat"]');
    
    // Verify search result added to context
    await expect(page.locator('[data-testid="context-search-result"]')).toContainText(
      'TypeScript Best Practices'
    );
  });

  test('should handle errors gracefully', async () => {
    // Mock error response
    await page.route('**/api/chat', route => {
      route.fulfill({
        status: 500,
        body: JSON.stringify({ error: 'Service temporarily unavailable' })
      });
    });
    
    // Try to send message
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('This should fail');
    await input.press('Enter');
    
    // Verify error message appears
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Service temporarily unavailable'
    );
    
    // Verify retry button appears
    await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();
  });

  test('should manage chat sessions', async () => {
    // Create first chat session
    await page.locator('[data-testid="chat-input"]').fill('First conversation');
    await page.keyboard.press('Enter');
    
    // Create new session
    await page.click('[data-testid="new-chat-button"]');
    
    // Verify clean slate
    await expect(page.locator('[data-testid="chat-messages"]')).toBeEmpty();
    
    // Start second conversation
    await page.locator('[data-testid="chat-input"]').fill('Second conversation');
    await page.keyboard.press('Enter');
    
    // Open history
    await page.click('[data-testid="tab-history"]');
    
    // Verify both sessions exist
    await expect(page.locator('[data-testid="session-item"]')).toHaveCount(2);
    
    // Load first session
    await page.click('[data-testid="session-item"]').first();
    
    // Verify first conversation loaded
    await expect(page.locator('[data-testid="user-message"]')).toContainText(
      'First conversation'
    );
  });

  test('should export chat conversation', async () => {
    // Have a conversation
    await mockOllamaResponse(page, {
      model: 'gpt-4',
      response: 'Here is my response to your question.'
    });
    
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Can you explain promises?');
    await input.press('Enter');
    
    await expect(page.locator('[data-testid="assistant-message"]')).toBeVisible();
    
    // Export conversation
    await page.click('[data-testid="export-button"]');
    await page.click('[data-testid="export-format-markdown"]');
    
    // Verify download initiated
    const download = await page.waitForEvent('download');
    expect(download.suggestedFilename()).toContain('.md');
  });

  test('should handle context window limits', async () => {
    // Fill context with large content
    const largeMessage = 'x'.repeat(100000); // Large message
    
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill(largeMessage);
    
    // Verify warning appears
    await expect(page.locator('[data-testid="context-warning"]')).toContainText(
      'Message exceeds context window'
    );
    
    // Verify send button is disabled
    await expect(page.locator('[data-testid="send-button"]')).toBeDisabled();
  });

  test('should use deep analysis mode', async () => {
    // Enable Socratic mode
    await page.click('[data-testid="analysis-mode-button"]');
    await page.click('[data-testid="mode-socratic"]');
    
    // Mock Socratic response
    await mockOllamaResponse(page, {
      model: 'gpt-4',
      response: 'Let me ask you some clarifying questions: What specific aspect of TypeScript are you struggling with?',
      mode: 'socratic'
    });
    
    // Send message
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('I need help with TypeScript');
    await input.press('Enter');
    
    // Verify Socratic response
    await expect(page.locator('[data-testid="assistant-message"]').last()).toContainText(
      'clarifying questions'
    );
    
    // Verify analysis indicator
    await expect(page.locator('[data-testid="analysis-mode-indicator"]')).toContainText(
      'Socratic'
    );
  });
});
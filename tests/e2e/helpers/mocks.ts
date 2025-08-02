import { Page } from '@playwright/test';

export async function mockOllamaResponse(page: Page, options: {
  model: string;
  response: string;
  stream?: boolean;
  context?: string[];
}) {
  await page.route('**/api/chat', route => {
    if (options.stream) {
      // Simulate streaming response
      route.fulfill({
        status: 200,
        contentType: 'text/event-stream',
        body: `data: ${JSON.stringify({ response: options.response })}\n\n`
      });
    } else {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          model: options.model,
          response: options.response,
          context: options.context || []
        })
      });
    }
  });
}

export async function mockSearchResults(page: Page, results: Array<{
  title: string;
  url: string;
  content: string;
}>) {
  await page.route('**/api/search', route => {
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        results: results.map(r => ({
          ...r,
          engine: 'mock',
          score: 0.9
        }))
      })
    });
  });
}
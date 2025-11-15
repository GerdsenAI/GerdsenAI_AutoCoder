import { test, expect } from '@playwright/test';

test.describe('GerdsenAI Socrates', () => {
  test('should load the application', async ({ page }) => {
    await page.goto('/');

    // Wait for the app to load
    await expect(page).toHaveTitle(/GerdsenAI/);

    // Check if main elements are present
    await expect(page.locator('body')).toBeVisible();
  });

  test('should display chat interface by default', async ({ page }) => {
    await page.goto('/');

    // Wait for chat interface to be visible
    const chatInterface = page.locator('[data-testid="chat-interface"]').first();
    await expect(chatInterface).toBeVisible({ timeout: 10000 });
  });

  test('should switch between tabs', async ({ page }) => {
    await page.goto('/');

    // Click on different tabs if they exist
    const tabs = ['chat', 'search', 'rag', 'history', 'code', 'settings'];

    for (const tab of tabs) {
      const tabButton = page.locator(`button:has-text("${tab}")`).first();
      if (await tabButton.isVisible()) {
        await tabButton.click();
        // Wait a bit for the tab to change
        await page.waitForTimeout(500);
      }
    }
  });

  test('should toggle theme', async ({ page }) => {
    await page.goto('/');

    // Look for theme toggle button
    const themeToggle = page.locator('button:has-text("Theme")').first();
    if (await themeToggle.isVisible()) {
      await themeToggle.click();

      // Check if theme changed
      const body = page.locator('body');
      const dataTheme = await body.getAttribute('data-theme');
      expect(dataTheme).toBeTruthy();
    }
  });

  test('should handle model selection', async ({ page }) => {
    await page.goto('/');

    // Look for model selector
    const modelSelector = page.locator('select').first();
    if (await modelSelector.isVisible()) {
      const options = await modelSelector.locator('option').count();
      expect(options).toBeGreaterThan(0);
    }
  });
});

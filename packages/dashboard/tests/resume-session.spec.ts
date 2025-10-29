// Test to diagnose resume session navigation issue in Quick Mode
import { test, expect, Page } from '@playwright/test';

test.describe('Quick Mode Resume Session', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // Enable console message logging
    page.on('console', msg => {
      console.log(`[${msg.type()}] ${msg.text()}`);
    });
    // Log all network requests
    page.on('response', response => {
      console.log(`[Network] ${response.status()} ${response.url()}`);
    });
  });

  test('should load editor view when resuming a completed session', async () => {
    // Navigate to dashboard
    await page.goto('http://localhost:5173');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    // Look for the "New Quick Mode" button or similar trigger
    // Since we don't know the exact UI, we'll look for elements that might trigger it
    const quickModeButton = page.locator('text=/Quick Mode|Generate PRD/i').first();

    if (await quickModeButton.isVisible()) {
      await quickModeButton.click();
    } else {
      console.log('Quick Mode button not found, trying to manually open dialog');
    }

    // Wait for the dialog to appear
    const dialogTitle = page.locator('text=Quick Mode');
    await expect(dialogTitle).toBeVisible({ timeout: 5000 });

    // Log what we see in the dialog
    const dialogContent = page.locator('[role="dialog"]');
    const dialogText = await dialogContent.textContent();
    console.log('[Dialog Content]', dialogText?.substring(0, 200));

    // Check if we see input screen or editor screen
    const inputField = page.locator('input[placeholder*="description"], input[placeholder*="idea"]').first();
    const editorButton = page.locator('text=Generate PRD');

    const hasInput = await inputField.isVisible({ timeout: 2000 }).catch(() => false);
    const hasEditor = await editorButton.isVisible({ timeout: 2000 }).catch(() => false);

    console.log('[Dialog State] Input visible:', hasInput, 'Editor visible:', hasEditor);

    // Take screenshot for debugging
    await page.screenshot({ path: 'test-resume-session.png' });
  });

  test('should handle previewPRD API calls correctly', async () => {
    // Navigate to dashboard
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    // Monitor API calls
    const apiCalls: Array<{ url: string; status?: number; responseBody?: any }> = [];

    page.on('response', async response => {
      if (response.url().includes('/api/ideate') || response.url().includes('/preview')) {
        try {
          const body = await response.json().catch(() => null);
          apiCalls.push({
            url: response.url(),
            status: response.status(),
            responseBody: body
          });
          console.log(`[API] ${response.status()} ${response.url()}`);
          if (body) {
            console.log('[API Response]', JSON.stringify(body).substring(0, 500));
          }
        } catch (e) {
          apiCalls.push({
            url: response.url(),
            status: response.status()
          });
        }
      }
    });

    // Try to trigger Quick Mode
    const quickModeButton = page.locator('text=/Quick Mode|Generate PRD/i').first();
    if (await quickModeButton.isVisible({ timeout: 3000 }).catch(() => false)) {
      await quickModeButton.click();
      await page.waitForTimeout(2000);
    }

    // Log all API calls we captured
    console.log('[API Calls Summary]', JSON.stringify(apiCalls.slice(0, 5), null, 2));
  });

  test('should check useIdeateSession hook behavior', async () => {
    // Navigate to dashboard
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');

    // Evaluate JavaScript in browser context to check React state
    const reactState = await page.evaluate(() => {
      // Try to access React DevTools or window.__REACT_DEVTOOLS_GLOBAL_HOOK__
      return {
        hasReactDevTools: typeof (window as any).__REACT_DEVTOOLS_GLOBAL_HOOK__ !== 'undefined',
        userAgent: navigator.userAgent
      };
    });

    console.log('[React State Check]', reactState);

    // Take screenshot
    await page.screenshot({ path: 'test-react-state.png' });
  });
});

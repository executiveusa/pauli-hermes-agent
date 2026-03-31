// Archon X Dashboard — E2E smoke tests
// Tests the live Vercel static site (no backend required)

const { test, expect } = require('@playwright/test');

test.describe('Landing Page', () => {
  test('loads with 200 and has title', async ({ page }) => {
    const response = await page.goto('/');
    expect(response.status()).toBe(200);
    await expect(page).toHaveTitle(/Hermes|Archon/i);
  });

  test('has link to dashboard', async ({ page }) => {
    await page.goto('/');
    const dashLink = page.locator('a[href*="dashboard"]').first();
    await expect(dashLink).toBeVisible();
  });
});

test.describe('Dashboard', () => {
  test('loads at /dashboard', async ({ page }) => {
    const response = await page.goto('/dashboard');
    expect(response.status()).toBe(200);
  });

  test('has agent switcher', async ({ page }) => {
    await page.goto('/dashboard');
    // Wait for JS to init
    await page.waitForTimeout(1000);
    const agentSwitch = page.locator('#agentSwitch');
    await expect(agentSwitch).toBeVisible();
  });

  test('has chat input', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(500);
    const chatInput = page.locator('#chatInput');
    await expect(chatInput).toBeVisible();
  });

  test('shows offline status when no API configured', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(2000);
    // When accessing from non-local, apiUrl should be empty → shows offline
    const statusText = page.locator('#hStatusText');
    await expect(statusText).toBeVisible();
    const text = await statusText.textContent();
    // Accept either "Offline" or "No API" — both are correct for static-only
    expect(text).toMatch(/offline|no api|connecting/i);
  });

  test('sidebar navigation works', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(500);
    // Click Settings nav item
    await page.click('[data-panel="settings"]');
    await expect(page.locator('#p-settings')).toHaveClass(/active/);
  });

  test('settings panel has API URL input', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(500);
    await page.click('[data-panel="settings"]');
    const urlInput = page.locator('#cfgUrl');
    await expect(urlInput).toBeVisible();
    await expect(urlInput).toHaveAttribute('placeholder', /localhost:8642/);
  });

  test('settings can be saved', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(500);
    await page.click('[data-panel="settings"]');
    await page.fill('#cfgUrl', 'http://test-api:8642');
    await page.click('#cfgSave');
    // Should show toast
    await expect(page.locator('#toast')).toBeVisible();
    await expect(page.locator('#toast')).toContainText(/saved/i);
  });

  test('command palette opens with Ctrl+K', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(500);
    await page.keyboard.press('Control+k');
    await expect(page.locator('#cmdBackdrop')).toBeVisible();
  });

  test('repos.json is accessible', async ({ page }) => {
    const response = await page.goto('/repos.json');
    expect(response.status()).toBe(200);
    const ct = response.headers()['content-type'];
    expect(ct).toContain('json');
  });
});

// Playwright E2E — Archon X Dashboard smoke tests
// Runs against the live Vercel site (or local dist/ via static serve)

const { defineConfig, devices } = require('@playwright/test');

const BASE_URL = process.env.TEST_URL || 'https://pauli-hermes-agent.vercel.app';

module.exports = defineConfig({
  testDir: './tests/e2e',
  timeout: 30000,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: BASE_URL,
    headless: true,
    screenshot: 'only-on-failure',
    video: 'off',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});

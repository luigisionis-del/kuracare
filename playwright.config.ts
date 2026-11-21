import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: 'tests/e2e',
  timeout: 60_000,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: { baseURL: 'http://localhost:4183', trace: 'retain-on-failure' },
  // E2E runs against the production build (CSP + service worker included).
  // Own port and never reuse: a preview left running for the phone would serve a stale build.
  webServer: { command: 'npm run build && npx vite preview --port 4183 --strictPort', url: 'http://localhost:4183', reuseExistingServer: false, timeout: 120_000 },
  projects: [{ name: 'pixel', use: { ...devices['Pixel 7'] } }],
});

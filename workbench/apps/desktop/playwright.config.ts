import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright drives the Vite dev server directly (not a packaged Tauri binary):
 * real Tauri E2E needs OS-level windowing unavailable in CI/this environment.
 * The Tauri IPC bridge (`window.__TAURI_INTERNALS__.invoke`) is mocked per-test
 * via `page.addInitScript` — the recommended pattern for exercising a Tauri
 * frontend without the native shell.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:5199",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    // Fixed port so `baseURL` is deterministic; `strictPort` fails fast if busy.
    command: "bun run vite --port 5199 --strictPort",
    url: "http://localhost:5199",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});

import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      allow: [new URL("../../..", import.meta.url).pathname],
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
    // Vitest owns unit/component tests under `src/`. Playwright specs live in
    // `e2e/` and use `@playwright/test` (incompatible with the Vitest runner),
    // so they must be excluded from the Vitest glob.
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["node_modules", "dist", "e2e/**"],
  },
});

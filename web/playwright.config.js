import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  timeout: 30_000,
  use: {
    baseURL: "http://127.0.0.1:8080",
    headless: true,
  },
  webServer: {
    command: "npx serve -l 8080 --no-port-switching",
    cwd: __dirname,
    port: 8080,
    reuseExistingServer: true,
  },
});

import { defineConfig, devices } from "@playwright/test";

const PORT = 4173;
const CI = process.env["CI"] !== undefined;

// Runs against the production build served by `vite preview`, in both colour
// schemes so the accessibility scan covers light and dark tokens. The Wasm
// module must already be built (npm run build:wasm).
export default defineConfig({
  testDir: "tests/e2e",
  forbidOnly: CI,
  reporter: CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL: `http://localhost:${String(PORT)}`,
    trace: "retain-on-failure",
  },
  projects: [
    { name: "light", use: { ...devices["Desktop Chrome"], colorScheme: "light" } },
    { name: "dark", use: { ...devices["Desktop Chrome"], colorScheme: "dark" } },
  ],
  webServer: {
    command: `npm run build && npm run preview -- --port ${String(PORT)} --strictPort`,
    url: `http://localhost:${String(PORT)}`,
    reuseExistingServer: !CI,
  },
});

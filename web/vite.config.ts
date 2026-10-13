import { defineConfig } from "vitest/config";

// GitHub Pages serves the site from /<repository>/; the deploy workflow sets
// BASE_PATH accordingly. Local development and previews use the root.
export default defineConfig({
  base: process.env["BASE_PATH"] ?? "/",
  build: {
    target: "es2022",
  },
  test: {
    include: ["tests/unit/**/*.test.ts"],
    environment: "node",
  },
});

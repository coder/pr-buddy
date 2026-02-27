import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    // Force Svelte client bundle (not server) when running in jsdom
    conditions: ["browser"],
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
    globals: true,
    // Mock Tauri APIs so component imports don't explode in Node
    alias: {
      "@tauri-apps/api/core": new URL(
        "src/__mocks__/tauri-api.ts",
        import.meta.url,
      ).pathname,
      "@tauri-apps/api/event": new URL(
        "src/__mocks__/tauri-event.ts",
        import.meta.url,
      ).pathname,
      "@tauri-apps/plugin-opener": new URL(
        "src/__mocks__/tauri-opener.ts",
        import.meta.url,
      ).pathname,
      "@tauri-apps/plugin-notification": new URL(
        "src/__mocks__/tauri-notification.ts",
        import.meta.url,
      ).pathname,
      "@tauri-apps/plugin-autostart": new URL(
        "src/__mocks__/tauri-autostart.ts",
        import.meta.url,
      ).pathname,
    },
  },
});

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  optimizeDeps: {
    // Pre-bundle lucide icons so Vite resolves the "svelte" export condition in dev
    include: [
      "@lucide/svelte/icons/bell",
      "@lucide/svelte/icons/rocket",
      "@lucide/svelte/icons/x-circle",
      "@lucide/svelte/icons/rotate-ccw",
      "@lucide/svelte/icons/eye",
      "@lucide/svelte/icons/circle-dot",
      "@lucide/svelte/icons/loader",
      "@lucide/svelte/icons/check-circle",
      "@lucide/svelte/icons/file-edit",
      "@lucide/svelte/icons/git-merge",
      "@lucide/svelte/icons/inbox",
      "@lucide/svelte/icons/party-popper",
      "@lucide/svelte/icons/refresh-cw",
      "@lucide/svelte/icons/log-out",
    ],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
  },
  envPrefix: ["VITE_", "TAURI_"],
});

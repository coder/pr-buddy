// Stub for @tauri-apps/api/window
export function getCurrentWindow() {
  return {
    minimize: () => Promise.resolve(),
    hide: () => Promise.resolve(),
  };
}

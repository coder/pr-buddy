// Stub for @tauri-apps/plugin-autostart
let _enabled = false;
export async function enable(): Promise<void> {
  _enabled = true;
}
export async function disable(): Promise<void> {
  _enabled = false;
}
export async function isEnabled(): Promise<boolean> {
  return _enabled;
}

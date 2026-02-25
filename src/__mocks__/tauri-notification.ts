// Stub for @tauri-apps/plugin-notification
export async function isPermissionGranted(): Promise<boolean> {
  return true;
}
export async function requestPermission(): Promise<string> {
  return "granted";
}
export async function sendNotification(_options: unknown): Promise<void> {}

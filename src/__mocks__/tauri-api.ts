// Stub for @tauri-apps/api/core — returns sensible defaults so components mount
let invokeHandler: ((cmd: string, args?: unknown) => unknown) | null = null;

export async function invoke(cmd: string, args?: unknown): Promise<unknown> {
  if (invokeHandler) {
    return invokeHandler(cmd, args);
  }

  return null;
}

export function __setInvokeHandler(
  handler: (cmd: string, args?: unknown) => unknown,
): void {
  invokeHandler = handler;
}

export function __resetInvokeMock(): void {
  invokeHandler = null;
}

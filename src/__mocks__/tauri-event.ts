// Stub for @tauri-apps/api/event
const listeners = new Map<string, Set<(...args: unknown[]) => void>>();

function notifyListeners(event: string, payload?: unknown): void {
  const eventListeners = listeners.get(event);

  if (!eventListeners) {
    return;
  }

  const tauriEvent = {
    event,
    payload,
    id: 0,
    windowLabel: "main",
  };

  for (const handler of eventListeners) {
    handler(tauriEvent);
  }
}

export async function listen(
  event: string,
  handler: (...args: unknown[]) => void,
): Promise<() => void> {
  const eventListeners = listeners.get(event) ?? new Set();
  eventListeners.add(handler);
  listeners.set(event, eventListeners);

  return () => {
    const registeredHandlers = listeners.get(event);

    if (!registeredHandlers) {
      return;
    }

    registeredHandlers.delete(handler);

    if (registeredHandlers.size === 0) {
      listeners.delete(event);
    }
  };
}

export async function emit(event: string, payload?: unknown): Promise<void> {
  notifyListeners(event, payload);
}

export function __triggerEvent(eventName: string, payload?: unknown): void {
  notifyListeners(eventName, payload);
}

export function __resetListeners(): void {
  listeners.clear();
}

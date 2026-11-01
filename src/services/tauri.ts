import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function callCommand<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  return await invoke<T>(name, args);
}

// Returns a promise of the function that removes the listener
export function listenEvent<T>(name: string, handler: (payload: T) => void): Promise<() => void> {
  return listen<T>(name, (event) => handler(event.payload));
}

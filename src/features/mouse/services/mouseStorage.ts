import { RemapState } from "../types";

const STORAGE_KEY = "devicedeck.mouse.remap";

export function defaultRemapState(): RemapState {
  return { enabled: false, actions: {} };
}

export function loadRemapState(): RemapState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultRemapState();
    const parsed = JSON.parse(raw) as RemapState;
    return { enabled: Boolean(parsed.enabled), actions: parsed.actions ?? {} };
  } catch {
    return defaultRemapState();
  }
}

export function saveRemapState(state: RemapState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (e) {
    console.error("Failed to save mouse settings", e);
  }
}
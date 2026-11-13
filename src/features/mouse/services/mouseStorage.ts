import { getStorageItemWithMigration } from "../../../services/storage";
import { RemapState } from "../types";

const STORAGE_KEY = "mousely-hub.mouse.remap";
const LEGACY_STORAGE_KEY = "devicedeck.mouse.remap";

export function defaultRemapState(): RemapState {
  return { enabled: false, actions: {} };
}

export function loadRemapState(): RemapState {
  try {
    const raw = getStorageItemWithMigration(STORAGE_KEY, LEGACY_STORAGE_KEY);
    if (!raw) return defaultRemapState();
    const parsed = JSON.parse(raw) as RemapState;
    return { enabled: Boolean(parsed.enabled), actions: parsed.actions ?? {} };
  } catch {
    return defaultRemapState();
  }
}

/** Sent on window when the remap state was changed from outside the UI (backend value). */
export const REMAP_CHANGED_EVENT = "mousely-hub-remap-changed";

export function saveRemapState(state: RemapState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (e) {
    console.error("Failed to save mouse settings", e);
  }
}
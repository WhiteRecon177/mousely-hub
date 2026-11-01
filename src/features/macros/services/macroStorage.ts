import type { SavedMacro } from "../types";

const STORAGE_KEY = "devicedeck.macros";

export function loadMacros(): SavedMacro[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as SavedMacro[]) : [];
  } catch (err) {
    console.error("Failed to load macros", err);
    return [];
  }
}

export function saveMacros(list: SavedMacro[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  } catch (err) {
    console.error("Failed to save macros", err);
  }
}

import { useSyncExternalStore } from "react";

export type ThemeId = "indigo" | "sunset" | "crimson" | "ice";

/** Names are the same in every language (they are the names of the looks). */
export const THEMES: { id: ThemeId; name: string }[] = [
  { id: "indigo", name: "Indigo" },
  { id: "sunset", name: "Sunset" },
  { id: "crimson", name: "Crimson" },
  { id: "ice", name: "Ice" },
];

const STORAGE_KEY = "devicedeck.theme";
const DEFAULT_THEME: ThemeId = "indigo";

function isTheme(value: unknown): value is ThemeId {
  return THEMES.some((theme) => theme.id === value);
}

function load(): ThemeId {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (isTheme(saved)) return saved;
  } catch {
    // storage not available: use the default
  }
  return DEFAULT_THEME;
}

let current: ThemeId = load();
const listeners = new Set<() => void>();

function apply(): void {
  document.documentElement.dataset.theme = current;
  listeners.forEach((listener) => listener());
}

export function getTheme(): ThemeId {
  return current;
}

export function setTheme(theme: ThemeId): void {
  if (theme === current) return;
  current = theme;
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    // storage not available
  }
  apply();
}

// Applied as soon as this file is loaded, before the first screen is drawn (no colour flash).
// The tray menu is another window: it follows the theme chosen in the main one.
if (typeof window !== "undefined") {
  document.documentElement.dataset.theme = current;
  window.addEventListener("storage", (event) => {
    if (event.key === STORAGE_KEY && isTheme(event.newValue) && event.newValue !== current) {
      current = event.newValue;
      apply();
    }
  });
}

export function useTheme(): ThemeId {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    getTheme,
  );
}

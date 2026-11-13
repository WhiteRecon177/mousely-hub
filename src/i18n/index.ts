import { useSyncExternalStore } from "react";
import { getStorageItemWithMigration } from "../services/storage";
import { en, type Key } from "./locales/en";
import { it } from "./locales/it";
import { es } from "./locales/es";
import { de } from "./locales/de";
import { fr } from "./locales/fr";
import { pt } from "./locales/pt";
import { ru } from "./locales/ru";
import { zh } from "./locales/zh";

export type { Key };
export type Lang = "en" | "it" | "es" | "de" | "fr" | "pt" | "ru" | "zh";

/** Names are written in their own language, so everybody can find theirs. */
export const LANGUAGES: { code: Lang; name: string }[] = [
  { code: "en", name: "English" },
  { code: "it", name: "Italiano" },
  { code: "es", name: "Español" },
  { code: "de", name: "Deutsch" },
  { code: "fr", name: "Français" },
  { code: "pt", name: "Português (Brasil)" },
  { code: "ru", name: "Русский" },
  { code: "zh", name: "中文（简体）" },
];

const DICTS: Record<Lang, Record<Key, string>> = { en, it, es, de, fr, pt, ru, zh };
const STORAGE_KEY = "mousely-hub.language";
const LEGACY_STORAGE_KEY = "devicedeck.language";

function isLang(value: unknown): value is Lang {
  return typeof value === "string" && value in DICTS;
}

/** Saved choice, else the language of the system when we have it, else English. */
function detect(): Lang {
  const saved = getStorageItemWithMigration(STORAGE_KEY, LEGACY_STORAGE_KEY);
  if (isLang(saved)) return saved;
  const system = (typeof navigator !== "undefined" ? navigator.language : "en").slice(0, 2).toLowerCase();
  return isLang(system) ? system : "en";
}

let current: Lang = detect();
const listeners = new Set<() => void>();

function notify(): void {
  document.documentElement.lang = current;
  listeners.forEach((listener) => listener());
}

export function getLanguage(): Lang {
  return current;
}

export function setLanguage(lang: Lang): void {
  if (lang === current) return;
  current = lang;
  try {
    localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    // storage not available
  }
  notify();
}

// The tray menu is another window of the app: it follows the language chosen in the main one.
if (typeof window !== "undefined") {
  window.addEventListener("storage", (event) => {
    if (event.key === STORAGE_KEY && isLang(event.newValue) && event.newValue !== current) {
      current = event.newValue;
      notify();
    }
  });
  document.documentElement.lang = current;
}

export type Vars = Record<string, string | number>;

/** Text for a key in the current language; {name} markers are replaced by the values. */
export function translate(key: Key, vars?: Vars): string {
  const text = DICTS[current][key] ?? en[key];
  if (!vars) return text;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in vars ? String(vars[name]) : match,
  );
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Use in a component: it draws again when the language changes. */
export function useT(): typeof translate {
  useSyncExternalStore(subscribe, getLanguage);
  return translate;
}

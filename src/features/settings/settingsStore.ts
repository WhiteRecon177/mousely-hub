import { getStorageItemWithMigration } from "../../services/storage";

const KEY = "mousely-hub.settings";
const LEGACY_KEY = "devicedeck.settings";

export type Settings = {
  /** Closing the window keeps the app running in the tray. */
  closeToTray: boolean;
  /** Looks for a new version at start (the only thing the app asks the internet). */
  autoUpdate: boolean;
};

export const DEFAULT_SETTINGS: Settings = { closeToTray: true, autoUpdate: true };

export function loadSettings(): Settings {
  try {
    const raw = getStorageItemWithMigration(KEY, LEGACY_KEY);
    if (!raw) return DEFAULT_SETTINGS;
    const parsed: unknown = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      const { closeToTray, autoUpdate } = parsed as { closeToTray?: unknown; autoUpdate?: unknown };
      return {
        closeToTray: typeof closeToTray === "boolean" ? closeToTray : DEFAULT_SETTINGS.closeToTray,
        autoUpdate: typeof autoUpdate === "boolean" ? autoUpdate : DEFAULT_SETTINGS.autoUpdate,
      };
    }
  } catch {
    // storage not available: use the defaults
  }
  return DEFAULT_SETTINGS;
}

export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    // storage not available
  }
}

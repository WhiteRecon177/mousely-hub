import type { CameraSnapshot, WebcamProfile } from "../types";

const PROFILES_KEY = "devicedeck.webcam.profiles";
const AUTOSAVE_KEY = "devicedeck.webcam.autosave";
const AUTOSAVE_ENABLED_KEY = "devicedeck.webcam.autosaveEnabled";

function readJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function writeJson(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (err) {
    console.error("Cannot save settings:", err);
  }
}

export function loadProfiles(): WebcamProfile[] {
  return readJson<WebcamProfile[]>(PROFILES_KEY, []);
}

export function saveProfiles(profiles: WebcamProfile[]): void {
  writeJson(PROFILES_KEY, profiles);
}

// Last settings of each camera, stored by camera name
export function loadAutosave(cameraLabel: string): CameraSnapshot | null {
  const all = readJson<Record<string, CameraSnapshot>>(AUTOSAVE_KEY, {});
  return all[cameraLabel] ?? null;
}

export function saveAutosave(cameraLabel: string, snapshot: CameraSnapshot): void {
  const all = readJson<Record<string, CameraSnapshot>>(AUTOSAVE_KEY, {});
  all[cameraLabel] = snapshot;
  writeJson(AUTOSAVE_KEY, all);
}

export function isAutosaveEnabled(): boolean {
  return readJson<boolean>(AUTOSAVE_ENABLED_KEY, true);
}

export function setAutosaveEnabled(value: boolean): void {
  writeJson(AUTOSAVE_ENABLED_KEY, value);
}

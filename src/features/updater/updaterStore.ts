import { useSyncExternalStore } from "react";
import { loadSettings } from "../settings/settingsStore";

/**
 * Update flow: check -> available -> downloading -> installing.
 * The state lives outside React so the toast, the full screen and the settings button share it.
 */
export type UpdateState =
  | { phase: "idle" }
  | { phase: "checking"; manual: boolean }
  | { phase: "uptodate" }
  | { phase: "available"; version: string; notes: string; dismissed: boolean }
  | { phase: "downloading"; version: string; downloaded: number; total: number }
  | { phase: "installing"; version: string }
  | { phase: "error"; message: string };

// The plugin objects are only used here, so a browser preview (no Tauri) does not break.
type Pending = { version: string; body?: string; downloadAndInstall: (cb: (e: DownloadEvent) => void) => Promise<void> };
type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

let state: UpdateState = { phase: "idle" };
let pending: Pending | null = null;
const listeners = new Set<() => void>();

function set(next: UpdateState): void {
  state = next;
  listeners.forEach((listener) => listener());
}

export function getUpdateState(): UpdateState {
  return state;
}

export function useUpdateState(): UpdateState {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    getUpdateState,
  );
}

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Looks for a new version. `manual` shows "you are up to date" and errors; the automatic check stays silent. */
export async function checkForUpdate(manual = false): Promise<void> {
  if (state.phase === "checking" || state.phase === "downloading" || state.phase === "installing") return;
  set({ phase: "checking", manual });
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) {
      pending = null;
      set(manual ? { phase: "uptodate" } : { phase: "idle" });
      return;
    }
    pending = update as unknown as Pending;
    set({ phase: "available", version: update.version, notes: update.body ?? "", dismissed: false });
  } catch (e) {
    set(manual ? { phase: "error", message: errorText(e) } : { phase: "idle" });
  }
}

/** Hides the toast but keeps the update (the settings button still offers it). */
export function dismissUpdate(): void {
  if (state.phase === "available") set({ ...state, dismissed: true });
}

export function closeUpdateMessage(): void {
  if (state.phase === "uptodate" || state.phase === "error") set({ phase: "idle" });
}

export async function installUpdate(): Promise<void> {
  const update = pending;
  if (!update) return;
  const version = update.version;
  let downloaded = 0;
  let total = 0;
  set({ phase: "downloading", version, downloaded: 0, total: 0 });
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
        set({ phase: "downloading", version, downloaded: 0, total });
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        set({ phase: "downloading", version, downloaded, total });
      } else {
        set({ phase: "installing", version });
      }
    });
    set({ phase: "installing", version });
    // On Windows the installer closes the app by itself; elsewhere we restart it.
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch (e) {
    set({ phase: "error", message: errorText(e) });
  }
}

const FIRST_CHECK_MS = 4000;
const EVERY_MS = 6 * 60 * 60 * 1000;
let started = false;

/** Checks a few seconds after start and then every few hours. Not in `npm run tauri dev`. */
export function startAutoCheck(): void {
  if (started || import.meta.env.DEV) return;
  started = true;
  // The user can turn the automatic check off in the settings: it is read each time.
  const auto = () => {
    if (loadSettings().autoUpdate) void checkForUpdate(false);
  };
  window.setTimeout(auto, FIRST_CHECK_MS);
  window.setInterval(auto, EVERY_MS);
}

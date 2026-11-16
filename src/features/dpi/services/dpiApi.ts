import { invoke } from "@tauri-apps/api/core";
import { MouseInfo } from "../types";

export interface ProbeResult {
  info: MouseInfo;
  devices: MouseInfo[];
  activeId: string | null;
  diagnostics: string[];
}

// DPI and polling rate writing. The mouse is read back after each write.
export const DPI_WRITE_ENABLED = true;
export const POLLING_WRITE_ENABLED = true;

// The last successful reading is kept, so coming back to the Mouse tab is instant.
let cache: { result: ProbeResult; at: number } | null = null;
let inflightCached: Promise<ProbeResult> | null = null;
let inflightFresh: Promise<ProbeResult> | null = null;

function storeResult(result: ProbeResult, clearWhenDisconnected: boolean): void {
  if (result.info.connected) {
    cache = { result, at: Date.now() };
  } else if (clearWhenDisconnected) {
    cache = null;
  }
}

function patchCachedMouse(patch: Partial<MouseInfo>): void {
  if (!cache) return;
  const currentId = cache.result.info.id;
  const info = { ...cache.result.info, ...patch };
  const devices = cache.result.devices.map((dev) =>
    dev.id === currentId ? { ...dev, ...patch } : dev
  );
  cache = {
    at: cache.at,
    result: { ...cache.result, info, devices },
  };
}

export const dpiApi = {
  cached(): ProbeResult | null {
    return cache ? cache.result : null;
  },
  cacheAge(): number {
    return cache ? Date.now() - cache.at : Number.POSITIVE_INFINITY;
  },
  // The backend read the mouse again by itself (cable, receiver...): keep that reading.
  setFromEvent(result: ProbeResult): void {
    storeResult(result, true);
  },
  probe(force = false): Promise<ProbeResult> {
    if (!force && inflightCached) return inflightCached;
    if (force && inflightFresh) return inflightFresh;

    const pending = invoke<ProbeResult>("get_mouse_info", { force })
      .then((result) => {
        storeResult(result, force);
        return result;
      })
      .finally(() => {
        if (force) inflightFresh = null;
        else inflightCached = null;
      });

    if (force) inflightFresh = pending;
    else inflightCached = pending;
    return pending;
  },
  async selectActive(id: string): Promise<ProbeResult> {
    const result = await invoke<ProbeResult>("select_active_mouse", { id });
    storeResult(result, true);
    return result;
  },
  async setDpi(value: number): Promise<number> {
    const applied = await invoke<number>("set_mouse_dpi", { value });
    const readable = cache?.result.info.capabilities.dpiRead ?? true;
    patchCachedMouse({
      dpi: applied,
      dpiSource: applied > 0 ? (readable ? "hardware" : "saved") : "unset",
    });
    return applied;
  },
  async setPollingRate(hz: number): Promise<void> {
    await invoke<number>("set_polling_rate", { hz });
    const readable = cache?.result.info.capabilities.pollingRead ?? true;
    patchCachedMouse({
      pollingRate: hz,
      pollingSource: hz > 0 ? (readable ? "hardware" : "saved") : "unset",
    });
  },
};
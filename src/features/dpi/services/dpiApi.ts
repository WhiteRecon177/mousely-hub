import { invoke } from "@tauri-apps/api/core";
import { MouseInfo } from "../types";

export interface ProbeResult {
  info: MouseInfo;
  diagnostics: string[];
}

// DPI and polling rate writing. The mouse is read back after each write.
export const DPI_WRITE_ENABLED = true;
export const POLLING_WRITE_ENABLED = true;

// The last successful reading is kept, so coming back to the Mouse tab is instant.
let cache: { result: ProbeResult; at: number } | null = null;
let inflight: Promise<ProbeResult> | null = null;

export const dpiApi = {
  cached(): ProbeResult | null {
    return cache ? cache.result : null;
  },
  cacheAge(): number {
    return cache ? Date.now() - cache.at : Number.POSITIVE_INFINITY;
  },
  // The backend read the mouse again by itself (cable, receiver...): keep that reading.
  setFromEvent(result: ProbeResult): void {
    cache = result.info.connected ? { result, at: Date.now() } : null;
  },
  probe(): Promise<ProbeResult> {
    if (inflight) return inflight;
    inflight = invoke<ProbeResult>("get_mouse_info")
      .then((result) => {
        if (result.info.connected) cache = { result, at: Date.now() };
        return result;
      })
      .finally(() => {
        inflight = null;
      });
    return inflight;
  },
  async setDpi(value: number): Promise<number> {
    const applied = await invoke<number>("set_mouse_dpi", { value });
    if (cache) {
      cache = {
        at: cache.at,
        result: { ...cache.result, info: { ...cache.result.info, dpi: applied } },
      };
    }
    return applied;
  },
  async setPollingRate(hz: number): Promise<void> {
    await invoke<number>("set_polling_rate", { hz });
    if (cache) {
      cache = {
        at: cache.at,
        result: { ...cache.result, info: { ...cache.result.info, pollingRate: hz } },
      };
    }
  },
};
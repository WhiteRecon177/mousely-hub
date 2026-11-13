import { useEffect } from "react";
import { mouseApi } from "../services/mouseApi";
import { REMAP_CHANGED_EVENT, loadRemapState, saveRemapState } from "../services/mouseStorage";
import { syncRemap } from "../services/remapSync";

// Mounted once at app level: keeps remapping alive on every tab.
export function useRemapRuntime(): void {
  useEffect(() => {
    let alive = true;
    const offs: Array<() => void> = [];
    const add = (p: Promise<() => void>) => {
      p.then((off) => {
        if (alive) offs.push(off);
        else off();
      }).catch(() => {});
    };

    // The backend may have turned remapping off (F10) while no window existed: take its value
    // first, then send the saved mappings and macros.
    const start = async () => {
      try {
        const backend = await mouseApi.getState();
        const local = loadRemapState();
        if (backend.enabled !== local.enabled) {
          saveRemapState({ ...local, enabled: backend.enabled });
          window.dispatchEvent(new Event(REMAP_CHANGED_EVENT));
        }
      } catch (e) {
        console.error("Could not read the remap state", e);
      }
      await syncRemap(loadRemapState());
    };
    start().catch((e) => console.error("Remap sync failed", e));

    add(
      mouseApi.onDisabled(() => {
        const state = loadRemapState();
        saveRemapState({ ...state, enabled: false });
      })
    );

    return () => {
      alive = false;
      offs.forEach((off) => off());
    };
  }, []);
}
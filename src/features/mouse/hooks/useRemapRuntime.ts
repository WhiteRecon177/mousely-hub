import { useEffect } from "react";
import { mouseApi } from "../services/mouseApi";
import { loadRemapState, saveRemapState } from "../services/mouseStorage";
import { syncRemap } from "../services/remapSync";
import { loadMacros } from "../../macros/services/macroStorage";
import { macroApi } from "../../macros/services/macroApi";

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

    syncRemap(loadRemapState()).catch((e) => console.error("Remap sync failed", e));

    add(
      mouseApi.onTrigger((p) => {
        const macro = loadMacros().find((m) => m.id === p.id);
        if (!macro) return;
        macroApi.playMacro(macro.actions, 1).catch((e) => console.error(e));
      })
    );

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
import { ButtonId, RemapState } from "../types";
import { mouseApi } from "./mouseApi";
import { loadMacros } from "../../macros/services/macroStorage";
import type { MacroAction } from "../../macros/types";

const EDITABLE: ButtonId[] = ["middle", "back", "forward"];

export async function syncRemap(state: RemapState): Promise<void> {
  const list = EDITABLE.map((button) => ({
    button,
    action: state.actions[button] ?? { type: "Normal" as const },
  }));
  // Only the macros assigned to a button are sent.
  const saved = loadMacros();
  const macros: Record<string, MacroAction[]> = {};
  for (const item of list) {
    if (item.action.type !== "Macro") continue;
    const id = item.action.id;
    const found = saved.find((m) => m.id === id);
    if (found) macros[found.id] = found.actions;
  }
  await mouseApi.setMappings(list, macros);
  await mouseApi.setEnabled(state.enabled);
}
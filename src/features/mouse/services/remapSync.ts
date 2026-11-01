import { ButtonId, RemapState } from "../types";
import { mouseApi } from "./mouseApi";

const EDITABLE: ButtonId[] = ["middle", "back", "forward"];

export async function syncRemap(state: RemapState): Promise<void> {
  const list = EDITABLE.map((button) => ({
    button,
    action: state.actions[button] ?? { type: "Normal" as const },
  }));
  await mouseApi.setMappings(list);
  await mouseApi.setEnabled(state.enabled);
}
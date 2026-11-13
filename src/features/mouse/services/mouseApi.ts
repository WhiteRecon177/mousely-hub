import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ButtonId, RemapAction } from "../types";
import type { MacroAction } from "../../macros/types";

export const mouseApi = {
  // The macros used by the buttons go along: the backend plays them without a window.
  setMappings(
    list: { button: ButtonId; action: RemapAction }[],
    macros: Record<string, MacroAction[]>,
  ): Promise<void> {
    return invoke<void>("set_mouse_mappings", { mappings: list, macros });
  },
  getState(): Promise<{ enabled: boolean }> {
    return invoke<{ enabled: boolean }>("get_remap_state");
  },
  setEnabled(enabled: boolean): Promise<void> {
    return invoke<void>("set_remap_enabled", { enabled });
  },
  startDetect(): Promise<void> {
    return invoke<void>("start_button_detect");
  },
  cancelDetect(): Promise<void> {
    return invoke<void>("cancel_button_detect");
  },
  onDetected(handler: (button: string) => void): Promise<() => void> {
    return listen<string>("mouse-button-detected", (e) => handler(e.payload));
  },
  onDisabled(handler: () => void): Promise<() => void> {
    return listen<boolean>("mouse-remap-disabled", () => handler());
  },
};
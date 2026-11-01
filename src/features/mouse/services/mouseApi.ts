import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ButtonId, RemapAction } from "../types";

export interface TriggerPayload {
  button: string;
  id: string;
}

export const mouseApi = {
  setMappings(list: { button: ButtonId; action: RemapAction }[]): Promise<void> {
    return invoke<void>("set_mouse_mappings", { mappings: list });
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
  onTrigger(handler: (p: TriggerPayload) => void): Promise<() => void> {
    return listen<TriggerPayload>("mouse-remap-trigger", (e) => handler(e.payload));
  },
  onDisabled(handler: () => void): Promise<() => void> {
    return listen<boolean>("mouse-remap-disabled", () => handler());
  },
};
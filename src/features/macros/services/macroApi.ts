import { callCommand, listenEvent } from "../../../services/tauri";
import type { MacroAction } from "../types";

export const macroApi = {
  async startRecording(): Promise<void> {
    await callCommand<void>("start_recording");
  },

  async stopRecording(): Promise<MacroAction[]> {
    return await callCommand<MacroAction[]>("stop_recording");
  },

  async playMacro(actions: MacroAction[], repeat: number): Promise<void> {
    await callCommand<void>("play_macro", { actions, repeat });
  },

  async stopPlayback(): Promise<void> {
    await callCommand<void>("stop_playback");
  },

  onRecordingStopped(handler: (actions: MacroAction[]) => void): Promise<() => void> {
    return listenEvent<MacroAction[]>("macro-recording-stopped", handler);
  },

  onPlaybackFinished(handler: () => void): Promise<() => void> {
    return listenEvent<null>("macro-playback-finished", () => handler());
  },
};

export type MacroKey = string | Record<string, number>;
export type MacroButton = string | Record<string, number>;

export type ActionKind =
  | { type: "KeyDown"; key: MacroKey }
  | { type: "KeyUp"; key: MacroKey }
  | { type: "MouseDown"; button: MacroButton; x: number; y: number }
  | { type: "MouseUp"; button: MacroButton; x: number; y: number };

export type MacroAction = {
  delay_ms: number;
  kind: ActionKind;
};

export type SavedMacro = {
  id: string;
  name: string;
  actions: MacroAction[];
};

export type MacroStatus = "idle" | "recording" | "playing";

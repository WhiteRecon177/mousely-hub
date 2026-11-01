export type ButtonId = "left" | "right" | "middle" | "back" | "forward";

export type RemapAction =
  | { type: "Normal" }
  | { type: "Disabled" }
  | { type: "Keys"; keys: string[] }
  | { type: "Macro"; id: string };

export interface RemapState {
  enabled: boolean;
  actions: Partial<Record<ButtonId, RemapAction>>;
}

export interface ButtonRow {
  id: ButtonId;
  label: string;
  protectedButton: boolean;
}
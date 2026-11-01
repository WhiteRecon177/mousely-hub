export interface KeyOption {
  value: string;
  label: string;
}

export const MODIFIERS: KeyOption[] = [
  { value: "ControlLeft", label: "Ctrl" },
  { value: "ShiftLeft", label: "Shift" },
  { value: "Alt", label: "Alt" },
  { value: "MetaLeft", label: "Win" },
];

function build(): KeyOption[] {
  const list: KeyOption[] = [];
  for (const c of "ABCDEFGHIJKLMNOPQRSTUVWXYZ") {
    list.push({ value: `Key${c}`, label: c });
  }
  for (let i = 0; i <= 9; i++) {
    list.push({ value: `Num${i}`, label: String(i) });
  }
  for (let i = 1; i <= 12; i++) {
    if (i === 10) continue; // F10 is the emergency kill switch
    list.push({ value: `F${i}`, label: `F${i}` });
  }
  const others: KeyOption[] = [
    { value: "Return", label: "Enter" },
    { value: "Escape", label: "Esc" },
    { value: "Tab", label: "Tab" },
    { value: "Space", label: "Space" },
    { value: "Backspace", label: "Backspace" },
    { value: "Delete", label: "Delete" },
    { value: "Home", label: "Home" },
    { value: "End", label: "End" },
    { value: "PageUp", label: "Page Up" },
    { value: "PageDown", label: "Page Down" },
    { value: "UpArrow", label: "Up" },
    { value: "DownArrow", label: "Down" },
    { value: "LeftArrow", label: "Left" },
    { value: "RightArrow", label: "Right" },
  ];
  return list.concat(others);
}

export const MAIN_KEYS: KeyOption[] = build();
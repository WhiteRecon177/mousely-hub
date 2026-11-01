import type { MacroAction } from "../types";
import { translate } from "../../../i18n";

// Key and button names are a string or an object like { Unknown: 12 }
function nameOf(value: string | Record<string, number>): string {
  if (typeof value === "string") return value;
  const first = Object.keys(value)[0];
  return first ? `${first} ${value[first]}` : "Unknown";
}

export function describeAction(action: MacroAction): string {
  const { kind } = action;
  const delay = `(+${action.delay_ms} ms)`;

  switch (kind.type) {
    case "KeyDown":
    case "KeyUp":
      return `${kind.type} ${nameOf(kind.key)} ${delay}`;
    case "MouseDown":
    case "MouseUp":
      return translate("action.at", {
        type: kind.type,
        name: nameOf(kind.button),
        x: Math.round(kind.x),
        y: Math.round(kind.y),
        delay,
      });
    default:
      return translate("action.unknown");
  }
}

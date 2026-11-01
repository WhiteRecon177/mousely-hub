import { RemapAction } from "../types";
import { SavedMacro } from "../../macros/types";
import { MAIN_KEYS, MODIFIERS } from "./keyOptions";
import { translate } from "../../../i18n";

function keyLabel(value: string): string {
  const found = MODIFIERS.concat(MAIN_KEYS).find((k) => k.value === value);
  return found ? found.label : value;
}

export function describeRemap(action: RemapAction, macros: SavedMacro[]): string {
  switch (action.type) {
    case "Normal":
      return translate("action.normal");
    case "Disabled":
      return translate("action.disabled");
    case "Keys":
      return action.keys.length === 0 ? translate("remap.keysNone") : action.keys.map(keyLabel).join(" + ");
    case "Macro": {
      const macro = macros.find((m) => m.id === action.id);
      return macro ? translate("remap.macroName", { name: macro.name }) : translate("remap.macroMissing");
    }
  }
}
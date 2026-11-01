import React from "react";
import { RemapAction } from "../types";
import { SavedMacro } from "../../macros/types";
import { MAIN_KEYS, MODIFIERS } from "../services/keyOptions";
import { useT } from "../../../i18n";

interface ActionEditorProps {
  value: RemapAction;
  macros: SavedMacro[];
  onChange: (action: RemapAction) => void;
}

const MOD_VALUES = MODIFIERS.map((m) => m.value);

export const ActionEditor: React.FC<ActionEditorProps> = ({ value, macros, onChange }) => {
  const t = useT();
  const handleType = (type: string) => {
    if (type === "Normal") onChange({ type: "Normal" });
    else if (type === "Disabled") onChange({ type: "Disabled" });
    else if (type === "Keys") onChange({ type: "Keys", keys: ["KeyA"] });
    else onChange({ type: "Macro", id: macros[0]?.id ?? "" });
  };

  return (
    <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
      <select value={value.type} onChange={(e) => handleType(e.target.value)}>
        <option value="Normal">{t("action.normalDefault")}</option>
        <option value="Keys">{t("action.keys")}</option>
        <option value="Macro">{t("action.macro")}</option>
        <option value="Disabled">{t("action.disabled")}</option>
      </select>

      {value.type === "Keys" && (
        <KeysEditor keys={value.keys} onChange={(keys) => onChange({ type: "Keys", keys })} />
      )}

      {value.type === "Macro" &&
        (macros.length === 0 ? (
          <span>{t("macro.noneSaved")}</span>
        ) : (
          <select value={value.id} onChange={(e) => onChange({ type: "Macro", id: e.target.value })}>
            {macros.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        ))}
    </div>
  );
};

interface KeysEditorProps {
  keys: string[];
  onChange: (keys: string[]) => void;
}

const KeysEditor: React.FC<KeysEditorProps> = ({ keys, onChange }) => {
  const t = useT();
  const mods = keys.filter((k) => MOD_VALUES.includes(k));
  const main = keys.find((k) => !MOD_VALUES.includes(k)) ?? "";

  const build = (nextMods: string[], nextMain: string): string[] => {
    const ordered = MOD_VALUES.filter((m) => nextMods.includes(m));
    return nextMain ? [...ordered, nextMain] : ordered;
  };

  const toggleMod = (mod: string) => {
    const next = mods.includes(mod) ? mods.filter((m) => m !== mod) : [...mods, mod];
    onChange(build(next, main));
  };

  return (
    <>
      {MODIFIERS.map((m) => (
        <label key={m.value}>
          <input type="checkbox" checked={mods.includes(m.value)} onChange={() => toggleMod(m.value)} />
          {m.label}
        </label>
      ))}
      <select value={main} onChange={(e) => onChange(build(mods, e.target.value))}>
        <option value="">{t("key.none")}</option>
        {MAIN_KEYS.map((k) => (
          <option key={k.value} value={k.value}>
            {k.label}
          </option>
        ))}
      </select>
    </>
  );
};
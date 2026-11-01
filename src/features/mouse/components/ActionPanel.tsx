import { ButtonId, RemapAction } from "../types";
import { SavedMacro } from "../../macros/types";
import { ActionEditor } from "./ActionEditor";
import { useT, type Key } from "../../../i18n";

interface ActionPanelProps {
  selected: ButtonId;
  protectedButton: boolean;
  action: RemapAction;
  macros: SavedMacro[];
  onChange: (action: RemapAction) => void;
}

const LABEL_KEYS: Record<ButtonId, Key> = {
  left: "btn.left",
  right: "btn.right",
  middle: "btn.middle",
  back: "btn.back",
  forward: "btn.forward",
};

export function ActionPanel({ selected, protectedButton, action, macros, onChange }: ActionPanelProps) {
  const t = useT();

  return (
    <section className="dd-section">
      <div className="dd-selected">
        <span className="dd-dot" />
        <span className="dd-muted">{t("action.selected")}</span> {t(LABEL_KEYS[selected])}
      </div>
      <div className="dd-title">{t("action.title")}</div>
      {protectedButton ? (
        <div className="dd-hint">{t("action.protected")}</div>
      ) : (
        <ActionEditor value={action} macros={macros} onChange={onChange} />
      )}
    </section>
  );
}
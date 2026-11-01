import { ButtonId, ButtonRow, RemapState } from "../types";
import { SavedMacro } from "../../macros/types";
import { describeRemap } from "../services/describeRemap";
import { LockIcon, MouseIcon } from "./Icons";
import { useT } from "../../../i18n";

interface ButtonListProps {
  rows: ButtonRow[];
  actions: RemapState["actions"];
  macros: SavedMacro[];
  selected: ButtonId;
  detecting: boolean;
  detected: ButtonId | null;
  onSelect: (id: ButtonId) => void;
  onDetect: () => void;
  onCancelDetect: () => void;
}

export function ButtonList({
  rows,
  actions,
  macros,
  selected,
  detecting,
  detected,
  onSelect,
  onDetect,
  onCancelDetect,
}: ButtonListProps) {
  const t = useT();

  return (
    <div className="dd-card dd-col dd-left">
      <div className="dd-title">{t("btn.yourButtons")}</div>

      <div className="dd-btn-list">
        {rows.map((row) => {
          const text = row.protectedButton
            ? t("action.normal")
            : describeRemap(actions[row.id] ?? { type: "Normal" }, macros);
          const classes = ["dd-btn-row"];
          if (row.id === selected) classes.push("dd-btn-row--selected");
          if (row.id === detected) classes.push("dd-btn-row--detected");
          if (row.protectedButton) classes.push("dd-btn-row--locked");

          return (
            <button
              key={row.id}
              type="button"
              className={classes.join(" ")}
              onClick={() => onSelect(row.id)}
            >
              <span className="dd-btn-label">
                {row.protectedButton && <LockIcon size={14} />}
                {row.label}
              </span>
              <span className="dd-badge">{text}</span>
            </button>
          );
        })}
      </div>

      <button
        type="button"
        className="dd-detect"
        onClick={detecting ? onCancelDetect : onDetect}
      >
        <MouseIcon size={18} />
        {detecting ? t("btn.detecting") : t("btn.detect")}
      </button>
    </div>
  );
}
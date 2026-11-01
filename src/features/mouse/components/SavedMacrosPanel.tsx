import { useEffect, useState } from "react";
import { SavedMacro } from "../../macros/types";
import { ChevronIcon, EditIcon, PlayIcon, TrashIcon } from "./Icons";
import { useT } from "../../../i18n";

interface SavedMacrosPanelProps {
  macros: SavedMacro[];
  highlightId: string | null;
  disabled: boolean;
  onPlay: (macro: SavedMacro) => void;
  onEdit: (macro: SavedMacro) => void;
  onDelete: (id: string) => void;
}

export function SavedMacrosPanel({
  macros,
  highlightId,
  disabled,
  onPlay,
  onEdit,
  onDelete,
}: SavedMacrosPanelProps) {
  const t = useT();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (highlightId !== null) setOpen(true);
  }, [highlightId]);

  const handleDelete = (id: string) => {
    if (window.confirm(t("macro.confirmDelete"))) onDelete(id);
  };

  return (
    <section className="dd-section">
      <button type="button" className="dd-collapse-header" onClick={() => setOpen(!open)}>
        <ChevronIcon size={16} open={open} />
        {t("macro.savedTitle", { n: macros.length })}
      </button>

      {open &&
        (macros.length === 0 ? (
          <div className="dd-empty">{t("macro.noneSaved")}</div>
        ) : (
          <div className="dd-macro-list">
            {macros.map((macro) => (
              <div
                key={macro.id}
                className={
                  macro.id === highlightId ? "dd-macro-row dd-macro-row--highlight" : "dd-macro-row"
                }
              >
                <span className="dd-macro-name">
                  {macro.name} <span className="dd-muted">· {t("macro.actionsCount", { n: macro.actions.length })}</span>
                </span>
                <button
                  type="button"
                  className="dd-mini-btn"
                  title={t("common.play")}
                  aria-label={t("common.play")}
                  disabled={disabled}
                  onClick={() => onPlay(macro)}
                >
                  <PlayIcon size={14} />
                </button>
                <button
                  type="button"
                  className="dd-mini-btn"
                  title={t("common.edit")}
                  aria-label={t("common.edit")}
                  disabled={disabled}
                  onClick={() => onEdit(macro)}
                >
                  <EditIcon size={14} />
                </button>
                <button
                  type="button"
                  className="dd-mini-btn dd-mini-btn--danger"
                  title={t("common.delete")}
                  aria-label={t("common.delete")}
                  disabled={disabled}
                  onClick={() => handleDelete(macro.id)}
                >
                  <TrashIcon size={14} />
                </button>
              </div>
            ))}
          </div>
        ))}
    </section>
  );
}
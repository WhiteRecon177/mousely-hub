import { SaveIcon, TrashIcon } from "./Icons";
import { useT } from "../../../i18n";

interface MacroNameRowProps {
  name: string;
  canSave: boolean;
  onNameChange: (name: string) => void;
  onSave: () => void;
  onClear: () => void;
}

export function MacroNameRow({ name, canSave, onNameChange, onSave, onClear }: MacroNameRowProps) {
  const t = useT();

  return (
    <div className="dd-name-row">
      <input
        type="text"
        placeholder={t("macro.namePlaceholder")}
        value={name}
        onChange={(e) => onNameChange(e.target.value)}
      />
      <button
        type="button"
        className="dd-icon-btn dd-icon-btn--primary"
        title={t("common.save")}
        aria-label={t("common.save")}
        disabled={!canSave}
        onClick={onSave}
      >
        <SaveIcon size={20} />
      </button>
      <button
        type="button"
        className="dd-icon-btn dd-icon-btn--danger"
        title={t("macro.clear")}
        aria-label={t("macro.clear")}
        onClick={onClear}
      >
        <TrashIcon size={20} />
      </button>
    </div>
  );
}
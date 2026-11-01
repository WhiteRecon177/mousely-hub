import { useState } from "react";
import type { WebcamProfile } from "../types";
import { useT } from "../../../i18n";

type ProfilesPanelProps = {
  profiles: WebcamProfile[];
  autosave: boolean;
  onAutosaveChange: (value: boolean) => void;
  onSave: (name: string) => void;
  onApply: (profile: WebcamProfile) => void;
  onDelete: (id: string) => void;
};

export function ProfilesPanel({
  profiles,
  autosave,
  onAutosaveChange,
  onSave,
  onApply,
  onDelete,
}: ProfilesPanelProps) {
  const t = useT();
  const [name, setName] = useState("");

  const handleSave = () => {
    if (!name.trim()) return;
    onSave(name);
    setName("");
  };

  return (
    <div className="dd-cam-card">
      <div className="dd-cam-title">{t("prof.title")}</div>

      <label className="dd-cam-switch">
        <input
          type="checkbox"
          checked={autosave}
          onChange={(e) => onAutosaveChange(e.target.checked)}
        />
        <span className="dd-cam-switch-track" />
        <span>{t("prof.autosave")}</span>
      </label>

      <div className="dd-cam-save-row">
        <input
          className="dd-cam-input"
          value={name}
          placeholder={t("prof.placeholder")}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSave();
          }}
        />
        <button
          type="button"
          className="dd-cam-btn"
          disabled={name.trim() === ""}
          onClick={handleSave}
        >
          {t("common.save")}
        </button>
      </div>

      {profiles.length === 0 ? (
        <div className="dd-cam-muted">{t("prof.none")}</div>
      ) : (
        <ul className="dd-cam-profiles">
          {profiles.map((profile) => (
            <li key={profile.id} className="dd-cam-profile">
              <span className="dd-cam-profile-name">{profile.name}</span>
              <button type="button" className="dd-cam-btn" onClick={() => onApply(profile)}>
                {t("common.apply")}
              </button>
              <button
                type="button"
                className="dd-cam-btn dd-cam-btn--danger"
                onClick={() => onDelete(profile.id)}
              >
                {t("common.delete")}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
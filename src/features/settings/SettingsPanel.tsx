import { getVersion } from "@tauri-apps/api/app";
import { useCallback, useEffect, useRef, useState } from "react";
import { callCommand } from "../../services/tauri";
import { LANGUAGES, getLanguage, setLanguage, useT, type Lang } from "../../i18n";
import { THEMES, setTheme, useTheme, type ThemeId } from "../../theme/theme";
import { DEFAULT_SETTINGS, loadSettings, saveSettings, type Settings } from "./settingsStore";
import { UpdateRow } from "../updater";
import "./settings.css";

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Tells the backend what the close button should do. Called at start and when it changes. */
export function applyCloseSetting(settings: Settings): void {
  void callCommand("set_close_to_tray", { enabled: settings.closeToTray }).catch(() => undefined);
}

type ToggleProps = {
  label: string;
  hint?: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (value: boolean) => void;
};

function Toggle({ label, hint, checked, disabled, onChange }: ToggleProps) {
  return (
    <label className="dd-set-row">
      <span className="dd-set-text">
        <span className="dd-set-label">{label}</span>
        {hint && <span className="dd-set-hint">{hint}</span>}
      </span>
      <input
        type="checkbox"
        className="dd-switch"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}

export function SettingsButton() {
  const t = useT();
  const theme = useTheme();
  const [open, setOpen] = useState(false);
  const [settings, setSettings] = useState<Settings>(() => loadSettings());
  const [autostart, setAutostart] = useState(false);
  const [note, setNote] = useState("");
  const [version, setVersion] = useState("");
  const rootRef = useRef<HTMLDivElement>(null);

  // Windows decides "start with Windows", so it is read from there each time the panel opens.
  useEffect(() => {
    void getVersion().then(setVersion).catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!open) return;
    setNote("");
    void callCommand<boolean>("get_autostart").then(setAutostart).catch(() => undefined);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onDown = (event: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const update = useCallback((next: Settings) => {
    setSettings(next);
    saveSettings(next);
    applyCloseSetting(next);
  }, []);

  const changeAutostart = async (enabled: boolean) => {
    try {
      setAutostart(await callCommand<boolean>("set_autostart", { enabled }));
      setNote("");
    } catch (e) {
      setNote(errorText(e));
    }
  };

  const copyInfo = async () => {
    try {
      const text = await callCommand<string>("export_hid_info");
      await navigator.clipboard.writeText(text);
      setNote(t("settings.copied"));
    } catch (e) {
      setNote(errorText(e));
    }
  };

  const reset = async () => {
    update(DEFAULT_SETTINGS);
    await changeAutostart(false);
    setNote(t("settings.resetDone"));
  };

  return (
    <div className="dd-settings" ref={rootRef}>
      <button
        type="button"
        className={open ? "dd-gear dd-gear--open" : "dd-gear"}
        aria-label={t("settings.title")}
        title={t("settings.title")}
        onClick={() => setOpen((value) => !value)}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
        </svg>
      </button>

      {open && (
        <div className="dd-set-panel" role="dialog" aria-label={t("settings.title")}>
          <Toggle label={t("settings.autostart")} hint={t("settings.autostartHint")} checked={autostart} onChange={changeAutostart} />
          <Toggle
            label={t("settings.tray")}
            hint={t("settings.trayHint")}
            checked={settings.closeToTray}
            onChange={(value) => update({ ...settings, closeToTray: value })}
          />
          <Toggle
            label={t("settings.autoUpdate")}
            hint={t("settings.autoUpdateHint")}
            checked={settings.autoUpdate}
            onChange={(value) => update({ ...settings, autoUpdate: value })}
          />

          <label className="dd-set-row dd-set-row--lang">
            <span className="dd-set-label">{t("settings.language")}</span>
            <select
              className="dd-set-select"
              value={getLanguage()}
              onChange={(event) => setLanguage(event.target.value as Lang)}
            >
              {LANGUAGES.map((lang) => (
                <option key={lang.code} value={lang.code}>
                  {lang.name}
                </option>
              ))}
            </select>
          </label>

          <label className="dd-set-row dd-set-row--lang">
            <span className="dd-set-label">{t("settings.theme")}</span>
            <select
              className="dd-set-select"
              value={theme}
              onChange={(event) => setTheme(event.target.value as ThemeId)}
            >
              {THEMES.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                </option>
              ))}
            </select>
          </label>

          <UpdateRow />

          <div className="dd-set-actions">
            <button type="button" className="dd-set-btn" onClick={copyInfo}>
              {t("settings.copyInfo")}
            </button>
            <button type="button" className="dd-set-btn dd-set-btn--ghost" onClick={reset}>
              {t("settings.reset")}
            </button>
          </div>

          {note && <div className="dd-set-note">{note}</div>}
          <div className="dd-set-foot">Mousely Hub {version} · {t("settings.footer")}</div>
        </div>
      )}
    </div>
  );
}

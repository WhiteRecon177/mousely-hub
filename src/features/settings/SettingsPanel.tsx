import { getVersion } from "@tauri-apps/api/app";
import { useCallback, useEffect, useRef, useState } from "react";
import { callCommand } from "../../services/tauri";
import { LANGUAGES, getLanguage, setLanguage, useT, type Lang } from "../../i18n";
import { THEMES, setTheme, useTheme, type ThemeId } from "../../theme/theme";
import { DEFAULT_SETTINGS, loadSettings, saveSettings, type Settings } from "./settingsStore";
import { UpdateRow } from "../updater";
import { loadRemapState, saveRemapState } from "../mouse/services/mouseStorage";
import { syncRemap } from "../mouse/services/remapSync";
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

  // Removes every saved macro and puts the buttons that used one back to normal.
  const deleteMacros = async () => {
    if (!confirm(t("settings.deleteMacrosConfirm"))) return;
    try {
      localStorage.removeItem("devicedeck.macros");
    } catch {
      // storage not available
    }
    const state = loadRemapState();
    const actions = { ...state.actions };
    for (const [button, action] of Object.entries(actions)) {
      if (action?.type === "Macro") actions[button as keyof typeof actions] = { type: "Normal" };
    }
    saveRemapState({ ...state, actions });
    try {
      await syncRemap(loadRemapState());
    } catch (e) {
      setNote(errorText(e));
      return;
    }
    // The macro lists and the button list read their data at start: load everything again.
    location.reload();
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
          <path d="M4 7h9M17 7h3M4 17h3M11 17h9" />
          <circle cx="15" cy="7" r="2" />
          <circle cx="9" cy="17" r="2" />
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
            <button type="button" className="dd-set-btn dd-set-btn--ghost" onClick={deleteMacros}>
              {t("settings.deleteMacros")}
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

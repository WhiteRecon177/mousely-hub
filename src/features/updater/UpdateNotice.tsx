import { useEffect } from "react";
import { useT } from "../../i18n";
import { dismissUpdate, installUpdate, startAutoCheck, useUpdateState } from "./updaterStore";
import "./updater.css";

function mb(bytes: number): string {
  return (bytes / 1048576).toFixed(1);
}

/** Toast "new version" + full screen with the progress. Mounted once in App. */
export function UpdateNotice() {
  const t = useT();
  const state = useUpdateState();

  useEffect(() => {
    startAutoCheck();
  }, []);

  if (state.phase === "available" && !state.dismissed) {
    return (
      <div className="dd-upd-toast" role="status">
        <div className="dd-upd-toast-icon" aria-hidden="true">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 3v12M7 10l5 5 5-5M5 21h14" />
          </svg>
        </div>
        <div className="dd-upd-toast-body">
          <div className="dd-upd-toast-title">{t("update.available", { version: state.version })}</div>
          <div className="dd-upd-toast-sub">{t("update.availableHint")}</div>
          <div className="dd-upd-toast-actions">
            <button type="button" className="dd-upd-btn" onClick={() => void installUpdate()}>
              {t("update.now")}
            </button>
            <button type="button" className="dd-upd-btn dd-upd-btn--ghost" onClick={dismissUpdate}>
              {t("update.later")}
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (state.phase === "downloading" || state.phase === "installing") {
    const known = state.phase === "downloading" && state.total > 0;
    const percent = state.phase === "installing" ? 100 : known ? Math.min(100, Math.round((state.downloaded / state.total) * 100)) : 0;
    return (
      <div className="dd-upd-screen" role="alertdialog" aria-live="polite">
        <div className="dd-upd-card">
          <div className="dd-upd-spinner" aria-hidden="true" />
          <h2 className="dd-upd-title">{t("update.title")}</h2>
          <div className="dd-upd-version">v{state.version}</div>
          <div className="dd-upd-bar" aria-hidden="true">
            <div
              className={known || state.phase === "installing" ? "dd-upd-fill" : "dd-upd-fill dd-upd-fill--indeterminate"}
              style={known || state.phase === "installing" ? { width: `${percent}%` } : undefined}
            />
          </div>
          <div className="dd-upd-status">
            <span>{state.phase === "installing" ? t("update.installing") : t("update.downloading")}</span>
            {state.phase === "downloading" && known && (
              <span>
                {percent}% · {mb(state.downloaded)} / {mb(state.total)} MB
              </span>
            )}
          </div>
          <div className="dd-upd-hint">{t("update.dontClose")}</div>
        </div>
      </div>
    );
  }

  return null;
}

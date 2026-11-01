import { useT } from "../../i18n";
import { checkForUpdate, closeUpdateMessage, installUpdate, useUpdateState } from "./updaterStore";

/** Row for the settings panel: check manually and see the result. */
export function UpdateRow() {
  const t = useT();
  const state = useUpdateState();

  let message = "";
  if (state.phase === "checking") message = t("update.checking");
  else if (state.phase === "uptodate") message = t("update.upToDate");
  else if (state.phase === "available") message = t("update.available", { version: state.version });
  else if (state.phase === "error") message = t("update.failed");

  const busy = state.phase === "checking" || state.phase === "downloading" || state.phase === "installing";

  return (
    <div className="dd-set-row dd-set-row--lang">
      <span className="dd-set-text">
        <span className="dd-set-label">{t("update.title2")}</span>
        {message && <span className="dd-set-hint">{message}</span>}
      </span>
      {state.phase === "available" ? (
        <button type="button" className="dd-set-btn" onClick={() => void installUpdate()}>
          {t("update.now")}
        </button>
      ) : (
        <button
          type="button"
          className="dd-set-btn"
          disabled={busy}
          onClick={() => {
            closeUpdateMessage();
            void checkForUpdate(true);
          }}
        >
          {t("update.check")}
        </button>
      )}
    </div>
  );
}

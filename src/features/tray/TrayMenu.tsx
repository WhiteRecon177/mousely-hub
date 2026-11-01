import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./tray-menu.css";
import { useT } from "../../i18n";

// Small menu shown by the tray icon (its own window, hidden when it loses focus).
export function TrayMenu() {
  const t = useT();
  const [title, setTitle] = useState("Mousely Hub");
  const [status, setStatus] = useState("");

  useEffect(() => {
    // The window is kept hidden between uses: read the text again each time it is shown.
    const load = () => {
      invoke<[string, string]>("tray_text")
        .then(([name, line]) => {
          setTitle(name);
          setStatus(line);
        })
        .catch(() => {});
    };

    load();

    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") void invoke("tray_close");
    };
    window.addEventListener("focus", load);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("focus", load);
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <div className="dd-tm">
      <div className="dd-tm-head">
        <div className="dd-tm-title">{title}</div>
        {status !== "" && <div className="dd-tm-status">{status}</div>}
      </div>
      <div className="dd-tm-line" />
      <button type="button" className="dd-tm-item" onClick={() => void invoke("tray_open")}>
        {t("tray.open")}
      </button>
      <button
        type="button"
        className="dd-tm-item dd-tm-item--quit"
        onClick={() => void invoke("tray_quit")}
      >
        {t("tray.quit")}
      </button>
    </div>
  );
}

import React from "react";
import ReactDOM from "react-dom/client";
import "./theme/themes.css";
import "./theme/theme";

// The tray menu is a small second window of the app: it shows its own page, not the whole app.
const internals = (
  window as unknown as {
    __TAURI_INTERNALS__?: { metadata?: { currentWindow?: { label?: string } } };
  }
).__TAURI_INTERNALS__;
const isTrayMenu = internals?.metadata?.currentWindow?.label === "tray-menu";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

if (isTrayMenu) {
  void import("./features/tray/TrayMenu").then(({ TrayMenu }) => {
    root.render(<TrayMenu />);
  });
} else {
  void import("./App").then(({ default: App }) => {
    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );
  });
}

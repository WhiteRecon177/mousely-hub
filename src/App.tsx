import { useEffect, useState } from "react";
import "./app/shell.css";
import { Brand, Wires } from "./app/Brand";
import { TitleBar } from "./app/TitleBar";
import { MousePage as MacrosPage, useRemapRuntime } from "./features/mouse";
import { DpiPage } from "./features/dpi";
import { WebcamPage } from "./features/webcam";
import { SettingsButton, applyCloseSetting, loadSettings } from "./features/settings";
import { UpdateNotice } from "./features/updater";
import { useT, type Key } from "./i18n";

type TabId = "mouse" | "macros" | "webcam";

const TABS: { id: TabId; label: Key }[] = [
  { id: "mouse", label: "tab.mouse" },
  { id: "macros", label: "tab.macro" },
  { id: "webcam", label: "tab.webcam" },
];

function App() {
  const t = useT();
  const [tab, setTab] = useState<TabId>("mouse");
  // Pages already opened stay in memory (hidden), so switching back is instant
  const [opened, setOpened] = useState<TabId[]>(["mouse"]);
  useRemapRuntime();

  // The backend does not remember the close setting: tell it at every start.
  useEffect(() => {
    applyCloseSetting(loadSettings());
  }, []);

  const select = (id: TabId) => {
    setTab(id);
    setOpened((prev) => (prev.includes(id) ? prev : [...prev, id]));
  };

  return (
    <div className="dd-app">
      <TitleBar />
      <div className="dd-shell-bar">
        <Brand />
        <div className="dd-mid">
          <Wires side="left" />
          <Wires side="right" />
          <div className="dd-tabs">
            {TABS.map((item) => (
              <button
                key={item.id}
                type="button"
                className={tab === item.id ? "dd-tab dd-tab--active" : "dd-tab"}
                onClick={() => select(item.id)}
              >
                {t(item.label)}
              </button>
            ))}
          </div>
        </div>
        <SettingsButton />
      </div>

      {opened.includes("mouse") && (
        <div hidden={tab !== "mouse"}>
          <DpiPage />
        </div>
      )}
      {opened.includes("macros") && (
        <div hidden={tab !== "macros"}>
          <MacrosPage />
        </div>
      )}
      {/* The webcam is released when you leave the tab, so other apps can use it */}
      {tab === "webcam" && <WebcamPage />}
      <UpdateNotice />
    </div>
  );
}

export default App;
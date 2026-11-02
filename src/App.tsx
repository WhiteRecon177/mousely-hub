import { useEffect, useState } from "react";
import "./app/shell.css";
import { TopBar } from "./app/TitleBar";
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
      <TopBar
        tabs={TABS.map((item) => ({ id: item.id, label: t(item.label) }))}
        active={tab}
        onSelect={(id) => select(id as TabId)}
      >
        <SettingsButton />
      </TopBar>

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
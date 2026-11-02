import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { ReactNode } from "react";
import { RELEASES_URL } from "./links";

export type TopTab = { id: string; label: string };

type TopBarProps = {
  tabs: TopTab[];
  active: string;
  onSelect: (id: string) => void;
  /** the settings button (it brings its own panel) */
  children?: ReactNode;
};

const ICON = {
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.7,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

function TabIcon({ id }: { id: string }) {
  if (id === "mouse") {
    return (
      <svg width="16" height="16" viewBox="0 0 24 24" {...ICON} aria-hidden="true">
        <rect x="6" y="3" width="12" height="18" rx="6" />
        <path d="M12 7v4" />
      </svg>
    );
  }
  if (id === "macros") {
    return (
      <svg width="16" height="16" viewBox="0 0 24 24" {...ICON} aria-hidden="true">
        <rect x="3" y="6" width="18" height="12" rx="2.5" />
        <path d="M7 10h.01M11 10h.01M15 10h.01M7 14h10" />
      </svg>
    );
  }
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" {...ICON} aria-hidden="true">
      <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12Z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

/** One top bar: it is also the window title bar (the Windows one is off). */
export function TopBar({ tabs, active, onSelect, children }: TopBarProps) {
  const win = () => getCurrentWindow();

  return (
    <header className="dd-top" data-tauri-drag-region>
      <div className="dd-top-brand" data-tauri-drag-region>
        <svg className="dd-top-logo" width="34" height="18" viewBox="0 0 40 20" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <path d="M20 10C14 2 4 2 4 10s10 8 16 0 16-8 16 0-10 8-16 0" />
        </svg>
        <span className="dd-top-name" data-tauri-drag-region>
          Mousely<span className="dd-top-hub">Hub</span>
        </span>
      </div>

      <nav className="dd-tabs">
        {tabs.map((item) => (
          <button
            key={item.id}
            type="button"
            className={active === item.id ? "dd-tab dd-tab--active" : "dd-tab"}
            onClick={() => onSelect(item.id)}
          >
            <TabIcon id={item.id} />
            {item.label}
          </button>
        ))}
      </nav>

      <div className="dd-top-right" data-tauri-drag-region>
        <button type="button" className="dd-iconbtn" aria-label="GitHub" title="GitHub" onClick={() => void openUrl(RELEASES_URL)}>
          <svg width="18" height="18" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z" />
          </svg>
        </button>
        {children}
        <span className="dd-top-sep" />
        <button type="button" className="dd-tb-btn" aria-label="Minimize" onClick={() => void win().minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" stroke="currentColor" strokeWidth="1" /></svg>
        </button>
        <button type="button" className="dd-tb-btn dd-tb-btn--close" aria-label="Close" onClick={() => void win().close()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor" strokeWidth="1" /></svg>
        </button>
      </div>
    </header>
  );
}

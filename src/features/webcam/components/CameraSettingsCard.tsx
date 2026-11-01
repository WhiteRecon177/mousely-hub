import { useState } from "react";
import type { ControlName, ControlRange, ModeName, ModeValue } from "../types";
import { ControlSlider } from "./ControlSlider";
import { useT, type Key } from "../../../i18n";

type TabId = "exposure" | "image" | "focus" | "zoom";

type TabConfig = {
  id: TabId;
  label: Key;
  mode?: { name: ModeName; label: Key };
  controls: ControlName[];
};

// exposureCompensation is intentionally not shown
const TABS: TabConfig[] = [
  {
    id: "exposure",
    label: "cam.tab.exposure",
    mode: { name: "exposureMode", label: "cam.auto.exposure" },
    controls: ["exposureTime"],
  },
  {
    id: "image",
    label: "cam.tab.image",
    mode: { name: "whiteBalanceMode", label: "cam.auto.wb" },
    controls: ["colorTemperature", "brightness", "contrast", "saturation", "sharpness"],
  },
  {
    id: "focus",
    label: "cam.tab.focus",
    mode: { name: "focusMode", label: "cam.auto.focus" },
    controls: ["focusDistance"],
  },
  { id: "zoom", label: "cam.tab.zoom", controls: ["zoom"] },
];

// Controls that only work in manual mode: they follow the Auto switch of their tab
const FOLLOWS_MODE: ControlName[] = ["exposureTime", "colorTemperature", "focusDistance"];

const SHOWN_AS_PERCENT: ControlName[] = ["brightness", "contrast", "saturation", "sharpness", "zoom"];

type CameraSettingsCardProps = {
  controls: ControlRange[];
  modes: Partial<Record<ModeName, ModeValue>>;
  onChange: (name: ControlName, value: number) => void;
  onModeChange: (mode: ModeName, value: ModeValue) => void;
};

export function CameraSettingsCard({
  controls,
  modes,
  onChange,
  onModeChange,
}: CameraSettingsCardProps) {
  const t = useT();
  const [activeId, setActiveId] = useState<TabId>("exposure");

  // Only tabs that have at least one control on this camera
  const tabs = TABS.filter((tab) => tab.controls.some((n) => controls.some((c) => c.name === n)));
  if (tabs.length === 0) return null;

  const active = tabs.find((t) => t.id === activeId) ?? tabs[0];
  const modeValue = active.mode ? modes[active.mode.name] : undefined;
  const isAuto = modeValue === "continuous";

  return (
    <div className="dd-cam-card">
      <div className="dd-cam-title">{t("cam.settings")}</div>

      <div className="dd-cam-tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            className={tab.id === active.id ? "dd-cam-tab dd-cam-tab--active" : "dd-cam-tab"}
            onClick={() => setActiveId(tab.id)}
          >
            {t(tab.label)}
          </button>
        ))}
      </div>

      {active.mode !== undefined && modeValue !== undefined && (
        <label className="dd-cam-switch">
          <input
            type="checkbox"
            checked={isAuto}
            onChange={() => {
              if (active.mode) onModeChange(active.mode.name, isAuto ? "manual" : "continuous");
            }}
          />
          <span className="dd-cam-switch-track" />
          <span>{t(active.mode.label)}</span>
        </label>
      )}

      {active.controls.map((name) => {
        const control = controls.find((c) => c.name === name);
        if (!control) return null;
        return (
          <ControlSlider
            key={name}
            control={control}
            onChange={onChange}
            percent={SHOWN_AS_PERCENT.includes(name)}
            disabled={isAuto && FOLLOWS_MODE.includes(name)}
          />
        );
      })}
    </div>
  );
}
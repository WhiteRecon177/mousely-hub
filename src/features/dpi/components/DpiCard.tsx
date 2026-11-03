import { useEffect, useRef, useState } from "react";
import { MouseInfo } from "../types";
import { useT } from "../../../i18n";

interface DpiCardProps {
  info: MouseInfo;
  dpi: number;
  stages: number[];
  disabled: boolean;
  /** true when the stages are the ones stored in the mouse (read-only list) */
  fromDevice: boolean;
  onChange: (value: number) => void;
  onSelectStage: (value: number) => void;
  onAddStage: () => void;
  onRemoveStage: (value: number) => void;
}

// Ring gauge: a 270 degree arc that opens at the bottom.
const R = 70;
const CX = 90;
const CY = 90;
const START = 135;
const SWEEP = 270;

function point(angle: number): [number, number] {
  const rad = (angle * Math.PI) / 180;
  return [CX + R * Math.cos(rad), CY + R * Math.sin(rad)];
}

function arc(from: number, to: number): string {
  const [x1, y1] = point(from);
  const [x2, y2] = point(to);
  const large = to - from > 180 ? 1 : 0;
  return `M ${x1.toFixed(2)} ${y1.toFixed(2)} A ${R} ${R} 0 ${large} 1 ${x2.toFixed(2)} ${y2.toFixed(2)}`;
}

export function DpiCard({
  info,
  dpi,
  stages,
  disabled,
  fromDevice,
  onChange,
  onSelectStage,
  onAddStage,
  onRemoveStage,
}: DpiCardProps) {
  const t = useT();
  const [customOpen, setCustomOpen] = useState(false);
  const [customText, setCustomText] = useState("");
  const customRef = useRef<HTMLDivElement>(null);

  // the small custom-DPI box closes with Esc or a click outside it
  useEffect(() => {
    if (!customOpen) return;
    const onDown = (e: MouseEvent) => {
      if (customRef.current && !customRef.current.contains(e.target as Node)) setCustomOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setCustomOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [customOpen]);

  const customValue = Number(customText);
  const customValid = customText.trim() !== "" && Number.isFinite(customValue) && customValue > 0;
  const toggleCustom = (): void => {
    setCustomText(String(dpi > 0 ? dpi : info.dpiMin));
    setCustomOpen((open) => !open);
  };
  const applyCustom = (): void => {
    if (!customValid) return;
    onSelectStage(customValue); // the hook rounds it to the mouse step and range
    setCustomOpen(false);
  };
  const outOfRange = (stage: number): boolean => stage < info.dpiMin || stage > info.dpiMax;
  // The ring uses a log scale: the useful range (400-3200) would be a sliver of 100-44000 otherwise.
  const lo = Math.max(1, info.dpiMin);
  const fraction =
    dpi > 0 && info.dpiMax > lo
      ? Math.min(1, Math.max(0, Math.log(Math.max(dpi, lo) / lo) / Math.log(info.dpiMax / lo)))
      : 0;
  const fmt = (n: number): string => n.toLocaleString("en-US").replace(/,/g, " ");

  return (
    <div className="dd-dpi-card dd-dpi-main">
      <div className="dd-card-head">
        <span className="dd-card-title">DPI</span>
        <span className="dd-card-head-right">
          <span className="dd-micro">
            {fmt(info.dpiMin)} — {fmt(info.dpiMax)}
          </span>
          <div className="dd-custom" ref={customRef}>
            <button
              type="button"
              className={customOpen ? "dd-custom-btn dd-custom-btn--open" : "dd-custom-btn"}
              disabled={disabled}
              aria-expanded={customOpen}
              onClick={toggleCustom}
            >
              {t("dpi.custom")}
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.4" aria-hidden="true">
                <path d="M2 3.5 5 6.5 8 3.5" />
              </svg>
            </button>
            {customOpen && (
              <div className="dd-custom-pop" role="dialog" aria-label={t("dpi.customTitle")}>
                <div className="dd-custom-title">{t("dpi.customTitle")}</div>
                <form
                  className="dd-custom-row"
                  onSubmit={(e) => {
                    e.preventDefault();
                    applyCustom();
                  }}
                >
                  <input
                    className="dd-custom-input"
                    type="number"
                    inputMode="numeric"
                    autoFocus
                    min={info.dpiMin}
                    max={info.dpiMax}
                    step={info.dpiStep}
                    value={customText}
                    onChange={(e) => setCustomText(e.target.value)}
                  />
                  <button type="submit" className="dd-custom-apply" disabled={!customValid}>
                    {t("dpi.apply")}
                  </button>
                </form>
                <div className="dd-micro dd-custom-note">
                  {t("dpi.customRange", { min: fmt(info.dpiMin), max: fmt(info.dpiMax), step: info.dpiStep })}
                </div>
              </div>
            )}
          </div>
        </span>
      </div>

      <div className="dd-dpi-body">
        <div className="dd-ring">
          <svg viewBox="0 0 180 180" className="dd-ring-svg" aria-hidden="true">
            <defs>
              <linearGradient id="dd-ring-grad" x1="0" y1="1" x2="1" y2="0">
                <stop offset="0" style={{ stopColor: "var(--holo)" }} />
                <stop offset="1" style={{ stopColor: "var(--holo-2)" }} />
              </linearGradient>
            </defs>
            <path d={arc(START, START + SWEEP)} className="dd-ring-track" />
            {fraction > 0 && (
              <path d={arc(START, START + Math.max(1, SWEEP * fraction))} className="dd-ring-fill" />
            )}
          </svg>
          <div className="dd-ring-center">
            <div className="dd-ring-value">{dpi > 0 ? dpi : "-"}</div>
            <div className="dd-ring-unit">DPI</div>
          </div>
        </div>

        <div className="dd-stage-list" title={fromDevice ? t("dpi.levelsFromMouse") : undefined}>
          {stages.map((stage, index) => (
            <div key={stage} className="dd-stage-wrap">
              <button
                type="button"
                className={stage === dpi ? "dd-stage-row dd-stage-row--active" : "dd-stage-row"}
                disabled={disabled || outOfRange(stage)}
                title={outOfRange(stage) ? t("dpi.stageUnsupported", { min: info.dpiMin, max: info.dpiMax }) : undefined}
                onClick={() => onSelectStage(stage)}
              >
                <span className="dd-stage-idx">{String(index + 1).padStart(2, "0")}</span>
                <span className="dd-stage-val">{stage}</span>
                {stage === dpi && <span className="dd-stage-dot" />}
              </button>
              {!fromDevice && stages.length > 1 && (
                <button
                  type="button"
                  className="dd-dpi-remove"
                  title={t("dpi.removeStage")}
                  aria-label={t("dpi.removeStageAria", { stage })}
                  disabled={disabled}
                  onClick={() => onRemoveStage(stage)}
                >
                  x
                </button>
              )}
            </div>
          ))}
          {!fromDevice && (
            <button type="button" className="dd-stage-add" disabled={disabled} onClick={onAddStage}>
              {t("dpi.addStage")}
            </button>
          )}
        </div>
      </div>

      <input
        type="range"
        className="dd-dpi-slider dd-dpi-slider--thin"
        min={info.dpiMin}
        max={info.dpiMax}
        step={info.dpiStep}
        value={Math.min(info.dpiMax, Math.max(info.dpiMin, dpi))}
        disabled={disabled}
        onChange={(e) => onChange(parseInt(e.target.value, 10))}
      />
    </div>
  );
}

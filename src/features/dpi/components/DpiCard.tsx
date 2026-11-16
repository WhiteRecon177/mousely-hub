import { useEffect, useId, useRef, useState } from "react";
import { MouseInfo } from "../types";
import { useT } from "../../../i18n";

interface DpiCardProps {
  info: MouseInfo;
  dpi: number;
  stages: number[];
  disabled: boolean;
  /** true when the stages are the ones stored in the mouse (read-only list) */
  fromDevice: boolean;
  readOnlyNote?: string;
  onSelectStage: (value: number) => void;
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
  readOnlyNote,
  onSelectStage,
}: DpiCardProps) {
  const t = useT();
  const customInputId = useId();
  const isUnset = dpi <= 0;
  const unsetDefault = Math.min(info.dpiMax, Math.max(info.dpiMin, 800));
  const clampedDpi = isUnset ? 0 : Math.min(info.dpiMax, Math.max(info.dpiMin, dpi));
  const stepBase = isUnset ? unsetDefault : clampedDpi;
  const [customText, setCustomText] = useState(() => (isUnset ? "" : String(clampedDpi)));
  const editingCustomRef = useRef(false);

  useEffect(() => {
    if (!editingCustomRef.current) {
      setCustomText(dpi > 0 ? String(clampedDpi) : "");
    }
  }, [dpi, clampedDpi]);

  const customValue = Number(customText);
  const customValid = customText.trim() !== "" && Number.isFinite(customValue) && customValue > 0;
  const step = Math.max(1, info.dpiStep || 100);

  const stepCustom = (delta: number): void => {
    if (disabled) return;
    const next = Math.min(info.dpiMax, Math.max(info.dpiMin, stepBase + delta));
    if (dpi > 0 && next === stepBase) return;
    editingCustomRef.current = false;
    setCustomText(String(next));
    onSelectStage(next);
  };

  const applyCustom = (): void => {
    if (disabled || !customValid) return;
    editingCustomRef.current = false;
    const rounded = Math.min(info.dpiMax, Math.max(info.dpiMin, Math.round(customValue / step) * step));
    setCustomText(String(rounded));
    onSelectStage(customValue); // the hook rounds it to the mouse step and range
  };

  const outOfRange = (stage: number): boolean => stage < info.dpiMin || stage > info.dpiMax;
  // The ring uses a log scale: the useful range (400-3200) would be a sliver of 100-44000 otherwise.
  const lo = Math.max(1, info.dpiMin);
  const fraction =
    dpi > 0 && info.dpiMax > lo
      ? Math.min(1, Math.max(0, Math.log(Math.max(dpi, lo) / lo) / Math.log(info.dpiMax / lo)))
      : 0;
  const fmt = (n: number): string => n.toLocaleString("en-US").replace(/,/g, " ");
  // Keep 5-8 hardware levels inside 2 balanced rows (3 cols for 5-6, 4 cols for 7-8) so no scrollbar appears.
  const stageListClass = [
    "dd-stage-list",
    stages.length > 6 ? "dd-stage-list--4col" : stages.length > 4 ? "dd-stage-list--3col" : "",
    stages.length > 8 ? "dd-stage-list--scroll" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className="dd-dpi-card dd-dpi-main">
      <div className="dd-card-head">
        <span className="dd-card-title">DPI</span>
        <span className="dd-micro dd-card-range">
          {fmt(info.dpiMin)} — {fmt(info.dpiMax)}
        </span>
      </div>

      {readOnlyNote ? (
        <div className="dd-dpi-readonly">{readOnlyNote}</div>
      ) : info.dpiSource !== "hardware" ? (
        <div
          className={
            info.dpiSource === "unset"
              ? "dd-dpi-source-note dd-dpi-source-note--unset"
              : "dd-dpi-source-note dd-dpi-source-note--saved"
          }
        >
          <span className="dd-source-dot" aria-hidden="true" />
          <span>{t(info.dpiSource === "unset" ? "source.unset" : "source.saved")}</span>
        </div>
      ) : null}

      <div className="dd-dpi-body">
        <div className="dd-ring-slot">
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
        </div>

        <div className="dd-dpi-stack">
          {stages.length > 0 && (
            <div
              className={stageListClass}
              title={fromDevice ? t("dpi.levelsFromMouse") : undefined}
            >
              {stages.map((stage, index) => {
                const activeStage = dpi > 0 && stage === dpi;
                return (
                  <button
                    key={stage}
                    type="button"
                    className={activeStage ? "dd-stage-row dd-stage-row--active" : "dd-stage-row"}
                    disabled={disabled || outOfRange(stage)}
                    title={outOfRange(stage) ? t("dpi.stageUnsupported", { min: info.dpiMin, max: info.dpiMax }) : undefined}
                    onClick={() => onSelectStage(stage)}
                  >
                    <span className="dd-stage-top">
                      <span className="dd-stage-idx">{String(index + 1).padStart(2, "0")}</span>
                      {activeStage && <span className="dd-stage-dot" />}
                    </span>
                    <span className="dd-stage-val">
                      {stage}
                      <small>DPI</small>
                    </span>
                  </button>
                );
              })}
            </div>
          )}

          <form
            className="dd-custom-card"
            title={t("dpi.customRange", { min: fmt(info.dpiMin), max: fmt(info.dpiMax), step: info.dpiStep })}
            onSubmit={(e) => {
              e.preventDefault();
              applyCustom();
            }}
          >
            <label className="dd-custom-card-title" htmlFor={customInputId}>
              {t("dpi.customTitle")}
            </label>

            <div className="dd-custom-controls">
              <button
                type="button"
                className="dd-step-btn"
                aria-label={`-${step} DPI`}
                disabled={disabled || stepBase <= info.dpiMin}
                onClick={() => stepCustom(-step)}
              >
                −
              </button>
              <div className="dd-custom-field-wrap">
                <input
                  id={customInputId}
                  className="dd-custom-field"
                  type="number"
                  inputMode="numeric"
                  min={info.dpiMin}
                  max={info.dpiMax}
                  step={info.dpiStep}
                  value={customText}
                  placeholder={isUnset ? String(unsetDefault) : undefined}
                  disabled={disabled}
                  onFocus={() => {
                    editingCustomRef.current = true;
                  }}
                  onBlur={() => {
                    editingCustomRef.current = false;
                  }}
                  onChange={(e) => setCustomText(e.target.value)}
                />
                <span className="dd-custom-unit" aria-hidden="true">
                  DPI
                </span>
              </div>
              <button
                type="button"
                className="dd-step-btn"
                aria-label={`+${step} DPI`}
                disabled={disabled || stepBase >= info.dpiMax}
                onClick={() => stepCustom(step)}
              >
                +
              </button>
              <button
                type="submit"
                className="dd-custom-apply"
                disabled={disabled || !customValid}
                onMouseDown={(e) => e.preventDefault()}
              >
                {t("dpi.apply")}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

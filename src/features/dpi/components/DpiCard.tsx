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
  const outOfRange = (stage: number): boolean => stage < info.dpiMin || stage > info.dpiMax;

  return (
    <div className="dd-dpi-card dd-dpi-card--compact">
      <div className="dd-dpi-head">
        <span className="dd-dpi-title dd-dpi-title--inline">DPI</span>
        <span className="dd-dpi-value-inline">{dpi > 0 ? dpi : "-"}</span>
      </div>

      <input
        type="range"
        className="dd-dpi-slider"
        min={info.dpiMin}
        max={info.dpiMax}
        step={info.dpiStep}
        value={Math.min(info.dpiMax, Math.max(info.dpiMin, dpi))}
        disabled={disabled}
        onChange={(e) => onChange(parseInt(e.target.value, 10))}
      />
      <div className="dd-dpi-range dd-dpi-range--tight">
        <span>{info.dpiMin}</span>
        <span>{info.dpiMax}</span>
      </div>

      <div
        className="dd-dpi-btn-row dd-dpi-btn-row--compact"
        title={fromDevice ? t("dpi.levelsFromMouse") : undefined}
      >
        {stages.map((stage) => (
          <div key={stage} className="dd-dpi-stage-wrap">
            <button
              type="button"
              className={stage === dpi ? "dd-dpi-stage dd-dpi-stage--active" : "dd-dpi-stage"}
              disabled={disabled || outOfRange(stage)}
              title={outOfRange(stage) ? t("dpi.stageUnsupported", { min: info.dpiMin, max: info.dpiMax }) : undefined}
              onClick={() => onSelectStage(stage)}
            >
              {stage}
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
          <button type="button" className="dd-dpi-ghost" disabled={disabled} onClick={onAddStage}>
            {t("dpi.addStage")}
          </button>
        )}
      </div>
    </div>
  );
}
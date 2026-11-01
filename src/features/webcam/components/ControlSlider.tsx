import type { ControlName, ControlRange } from "../types";
import { useT, type Key } from "../../../i18n";

type ControlSliderProps = {
  control: ControlRange;
  onChange: (name: ControlName, value: number) => void;
  disabled?: boolean;
  percent?: boolean;
};

const CONTROL_LABEL_KEYS: Record<ControlName, Key> = {
  brightness: "ctl.brightness",
  contrast: "ctl.contrast",
  saturation: "ctl.saturation",
  sharpness: "ctl.sharpness",
  colorTemperature: "ctl.colorTemperature",
  exposureTime: "ctl.exposureTime",
  exposureCompensation: "ctl.exposureCompensation",
  focusDistance: "ctl.focusDistance",
  zoom: "ctl.zoom",
};

function formatValue(control: ControlRange, percent: boolean): string {
  if (percent) {
    const span = control.max - control.min;
    if (span <= 0) return "100%";
    return `${Math.round(((control.value - control.min) / span) * 100)}%`;
  }
  return Number.isInteger(control.value) ? String(control.value) : control.value.toFixed(2);
}

export function ControlSlider({
  control,
  onChange,
  disabled = false,
  percent = false,
}: ControlSliderProps) {
  const t = useT();

  return (
    <div className={disabled ? "dd-cam-control dd-cam-control--off" : "dd-cam-control"}>
      <div className="dd-cam-control-head">
        <span>{t(CONTROL_LABEL_KEYS[control.name])}</span>
        <span className="dd-cam-control-value">{formatValue(control, percent)}</span>
      </div>
      <input
        type="range"
        className="dd-cam-slider"
        min={control.min}
        max={control.max}
        step={control.step}
        value={control.value}
        disabled={disabled}
        onChange={(e) => onChange(control.name, parseFloat(e.target.value))}
      />
    </div>
  );
}
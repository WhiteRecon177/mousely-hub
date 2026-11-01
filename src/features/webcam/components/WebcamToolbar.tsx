import { useT } from "../../../i18n";

type WebcamToolbarProps = {
  mirrored: boolean;
  onMirrorChange: (value: boolean) => void;
  fps: number | null;
  fpsOptions: number[];
  onFpsChange: (fps: number) => void;
  onAuto: () => void;
};

export function WebcamToolbar({
  mirrored,
  onMirrorChange,
  fps,
  fpsOptions,
  onFpsChange,
  onAuto,
}: WebcamToolbarProps) {
  const t = useT();

  // Always include the current fps so it is shown as active
  const options = Array.from(new Set([...fpsOptions, ...(fps ? [fps] : [])])).sort(
    (a, b) => a - b
  );

  return (
    <div className="dd-cam-options">
      <label className="dd-cam-switch">
        <input
          type="checkbox"
          checked={mirrored}
          onChange={(e) => onMirrorChange(e.target.checked)}
        />
        <span className="dd-cam-switch-track" />
        <span>{t("cam.mirror")}</span>
      </label>

      {options.length > 0 && (
        <div className="dd-cam-fps">
          <span className="dd-cam-muted">{t("cam.fps")}</span>
          {options.map((f) => (
            <button
              key={f}
              type="button"
              className={f === fps ? "dd-cam-seg dd-cam-seg--active" : "dd-cam-seg"}
              onClick={() => onFpsChange(f)}
            >
              {f}
            </button>
          ))}
        </div>
      )}

      <button type="button" className="dd-cam-btn dd-cam-btn--primary" onClick={onAuto}>
        {t("cam.autoAdjust")}
      </button>
    </div>
  );
}
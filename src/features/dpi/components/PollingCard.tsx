import { MouseInfo } from "../types";
import { useT } from "../../../i18n";

interface PollingCardProps {
  info: MouseInfo;
  disabled: boolean;
  onChange: (hz: number) => void;
}

export function PollingCard({ info, disabled, onChange }: PollingCardProps) {
  const t = useT();
  // Only the rates the mouse can really use with the current connection are shown.
  // They appear as soon as the mouse reports them (e.g. after plugging a faster receiver).
  const unknown = info.pollingSupported.length === 0;
  const options = unknown ? info.pollingOptions : info.pollingSupported;
  const maxSupported = unknown ? 0 : Math.max(...info.pollingSupported);

  return (
    <div className="dd-dpi-card">
      <div className="dd-dpi-title">{t("polling.title")}</div>
      <div className="dd-dpi-btn-row dd-dpi-btn-row--tight">
        {options.map((hz) => (
          <button
            key={hz}
            type="button"
            className={hz === info.pollingRate ? "dd-dpi-seg dd-dpi-seg--active" : "dd-dpi-seg"}
            disabled={disabled}
            onClick={() => onChange(hz)}
          >
            {hz} Hz
          </button>
        ))}
      </div>
      <div className="dd-dpi-range">
        <span>{unknown ? t("polling.notReported") : t("polling.maxNow", { hz: maxSupported })}</span>
      </div>
    </div>
  );
}
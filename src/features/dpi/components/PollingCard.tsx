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
  const ms = (hz: number): string => `${Math.round((1000 / hz) * 10) / 10} MS`;

  return (
    <div className="dd-dpi-card dd-poll">
      <div className="dd-card-head">
        <span className="dd-card-title">{t("polling.title")}</span>
        <span className="dd-micro">{unknown ? t("polling.notReported") : t("polling.maxNow", { hz: maxSupported })}</span>
      </div>
      <div className="dd-poll-grid" style={{ gridTemplateColumns: `repeat(${Math.max(1, options.length)}, minmax(0, 1fr))` }}>
        {options.map((hz) => (
          <button
            key={hz}
            type="button"
            className={hz === info.pollingRate ? "dd-poll-tile dd-poll-tile--active" : "dd-poll-tile"}
            disabled={disabled}
            onClick={() => onChange(hz)}
          >
            <span className="dd-poll-hz">
              {hz}
              <small>Hz</small>
            </span>
            <span className="dd-micro dd-poll-ms">{ms(hz)}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

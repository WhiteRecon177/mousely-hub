import type { ConnectionType, MouseInfo } from "../types";
import { useT, type Key } from "../../../i18n";

interface DeviceCardProps {
  info: MouseInfo | null;
}

export const CONNECTION_KEYS: Record<ConnectionType, Key> = {
  wired: "device.wired",
  wireless: "device.wireless",
  bluetooth: "device.bluetooth",
  unknown: "common.unknown",
};

function ConnectionIcon({ type }: { type: ConnectionType }) {
  const props = {
    className: "dd-st-icon",
    width: 18,
    height: 18,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };

  // cable: a lightning bolt
  if (type === "wired") {
    return (
      <svg {...props}>
        <path d="M13 2 4.5 13.5H11L10 22l8.5-11.5H12.5L13 2Z" />
      </svg>
    );
  }

  if (type === "bluetooth") {
    return (
      <svg {...props}>
        <path d="m7 7 10 10-5 5V2l5 5L7 17" />
      </svg>
    );
  }

  // wireless: radio waves around a dot
  return (
    <svg {...props}>
      <circle cx="12" cy="12" r="2" />
      <path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.5M16.2 7.8c2.3 2.3 2.3 6.1 0 8.5M19.1 4.9C23 8.8 23 15.1 19.1 19" />
    </svg>
  );
}

/** Status strip: state, connection and battery of the mouse. */
export function DeviceCard({ info }: DeviceCardProps) {
  const t = useT();
  const connected = info !== null && info.connected;
  const hasBattery = connected && info.capabilities.battery && info.battery !== null && info.connection !== "wired";
  const pct = hasBattery ? Math.max(0, Math.min(100, info.battery ?? 0)) : 0;

  return (
    <div className="dd-dpi-card dd-status">
      <div className="dd-status-grid">
        <div className="dd-st-cell">
          <div className="dd-micro">{t("dev.state")}</div>
          <div className="dd-st-value">
            <span className={connected ? "dd-st-dot" : "dd-st-dot dd-st-dot--off"} />
            {info === null ? t("device.looking") : connected ? t("dev.detectedShort") : t("device.none")}
          </div>
        </div>

        <div className="dd-st-cell">
          <div className="dd-micro">{t("dev.connection")}</div>
          <div className="dd-st-value">
            {connected ? (
              <>
                <ConnectionIcon type={info.connection} />
                {t(CONNECTION_KEYS[info.connection])}
              </>
            ) : (
              "-"
            )}
          </div>
        </div>

        <div className="dd-st-cell dd-st-cell--batt">
          <div className="dd-micro">{t("dev.battery")}</div>
          {connected && info.charging ? (
            <div className="dd-st-value">{info.battery !== null ? t("device.chargingPct", { pct: info.battery }) : t("device.charging")}</div>
          ) : hasBattery ? (
            <>
              <div className="dd-st-pct">
                {pct}
                <span>%</span>
              </div>
              <div className="dd-st-bar">
                <div className={pct <= 20 ? "dd-st-bar-fill dd-st-bar-fill--low" : "dd-st-bar-fill"} style={{ width: `${pct}%` }} />
              </div>
            </>
          ) : (
            <div className="dd-st-value">-</div>
          )}
        </div>
      </div>

      {connected && info.support === "expected" && (
        <div className="dd-dpi-support-note">{t("device.notTested")}</div>
      )}
      {connected && info.support === "recognized" && (
        <div className="dd-dpi-support-note">{t("device.recognized", { brand: info.brand })}</div>
      )}
      {connected && info.support === "unknown" && (
        <div className="dd-dpi-support-note">{t("device.unsupported")}</div>
      )}
    </div>
  );
}

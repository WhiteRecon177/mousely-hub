import type { ConnectionType, MouseInfo } from "../types";
import { useT, type Key } from "../../../i18n";

interface DeviceCardProps {
  info: MouseInfo | null;
}

const CONNECTION_KEYS: Record<ConnectionType, Key> = {
  wired: "device.wired",
  wireless: "device.wireless",
  bluetooth: "device.bluetooth",
  unknown: "common.unknown",
};

export function DeviceCard({ info }: DeviceCardProps) {
  const t = useT();

  function batteryText(mouse: MouseInfo): string | null {
    if (!mouse.capabilities.battery) return null;

    if (mouse.charging) {
      return mouse.battery !== null
        ? t("device.chargingPct", { pct: mouse.battery })
        : t("device.charging");
    }

    if (mouse.connection === "wired") return null;

    return mouse.battery !== null ? t("device.battery", { pct: mouse.battery }) : null;
  }

  const battery = info === null ? null : batteryText(info);

  return (
    <div className="dd-dpi-card">
      <div className="dd-dpi-title">{t("common.device")}</div>

      {info === null ? (
        <div className="dd-dpi-muted">{t("device.looking")}</div>
      ) : !info.connected ? (
        <div className="dd-dpi-row">
          <span className="dd-dpi-dot dd-dpi-dot--off" />
          <span className="dd-dpi-muted">{t("device.none")}</span>
        </div>
      ) : (
        <>
          <div className="dd-dpi-row">
            <span className="dd-dpi-dot" />
            <span>{t("device.detected")}</span>
            <span className="dd-dpi-muted">{info.name}</span>
            <span className="dd-dpi-badge">{t(CONNECTION_KEYS[info.connection])}</span>

            {info.support === "expected" && (
              <span className="dd-dpi-badge">{t("device.notTested")}</span>
            )}

            {battery !== null && <span className="dd-dpi-muted">{battery}</span>}
          </div>

          {info.support === "recognized" && (
            <div className="dd-dpi-support-note">{t("device.recognized", { brand: info.brand })}</div>
          )}

          {info.support === "unknown" && (
            <div className="dd-dpi-support-note">{t("device.unsupported")}</div>
          )}
        </>
      )}
    </div>
  );
}

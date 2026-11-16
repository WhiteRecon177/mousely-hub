import "../mouse/mouse.css";
import "./dpi.css";
import { useDpi } from "./hooks/useDpi";
import { DeviceCard, CONNECTION_KEYS } from "./components/DeviceCard";
import { DiagnosticsDialog } from "./components/DiagnosticsDialog";
import { DpiCard } from "./components/DpiCard";
import { PollingCard } from "./components/PollingCard";
import { DPI_WRITE_ENABLED, POLLING_WRITE_ENABLED } from "./services/dpiApi";
import { MouseDiagram } from "../mouse/components/MouseDiagram";
import { useT } from "../../i18n";

function noop(): void {}

export function DpiPage({ active = true }: { active?: boolean } = {}) {
  const t = useT();
  const dpi = useDpi(active);
  const { info } = dpi;
  const ready = info !== null && info.connected;
  const multiMouse = dpi.devices.length >= 2;
  const selectedId = dpi.activeId ?? info?.id ?? null;
  const activeIdx = multiMouse
    ? Math.max(0, dpi.devices.findIndex((dev) => dev.id === selectedId))
    : 0;

  const canReset =
    info !== null &&
    info.connected &&
    DPI_WRITE_ENABLED &&
    POLLING_WRITE_ENABLED &&
    info.capabilities.dpiWrite &&
    info.capabilities.pollingWrite;

  return (
    <div className="dd-dpi-page">
      <section className="dd-dev-panel">
        <span className="dd-screw dd-screw--tl" />
        <span className="dd-screw dd-screw--tr" />
        <span className="dd-screw dd-screw--bl" />
        <span className="dd-screw dd-screw--br" />

        <div className="dd-dev-head">
          <span className="dd-micro">
            {t("common.device")} · {String(activeIdx + 1).padStart(2, "0")}
            {multiMouse ? ` / ${String(dpi.devices.length).padStart(2, "0")}` : ""}
          </span>
          <span className="dd-micro dd-micro--accent">{ready ? info.brand : ""}</span>
        </div>

        <div className="dd-dpi-mouse">
          <MouseDiagram
            selected={null}
            detected={null}
            enabled={ready || dpi.scanning}
            scanning={dpi.scanning}
            onSelect={noop}
          />
        </div>

        <div className="dd-dev-foot">
          <div className="dd-micro">{t("dev.model")}</div>
          <div className="dd-dev-name">
            {ready ? info.name : dpi.scanning ? t("device.looking") : t("device.none")}
          </div>
          {ready && (
            <div className="dd-micro">
              {info.brand} · {t(CONNECTION_KEYS[info.connection])}
            </div>
          )}
        </div>

        {multiMouse && (
          <div className="dd-dev-switcher" role="group" aria-label={t("device.switch")}>
            {dpi.devices.map((dev, idx) => {
              const isSelected = dev.id === selectedId;
              return (
                <button
                  key={dev.id}
                  type="button"
                  className={isSelected ? "dd-dev-pill dd-dev-pill--active" : "dd-dev-pill"}
                  aria-pressed={isSelected}
                  title={`${dev.brand} · ${dev.name}`}
                  disabled={dpi.loading}
                  onClick={() => void dpi.selectDevice(dev.id)}
                >
                  <span className="dd-dev-pill-idx">{String(idx + 1).padStart(2, "0")}</span>
                  <span className="dd-dev-pill-name">{dev.name}</span>
                </button>
              );
            })}
          </div>
        )}
      </section>

      <div className="dd-dpi-right">
        <DeviceCard info={info} scanning={dpi.scanning} />

        {info !== null && !info.connected && !dpi.scanning && (
          <div className="dd-dpi-empty-bar">
            <span className="dd-dpi-empty-hint">{t("device.notFoundHint")}</span>
            <button
              type="button"
              className="dd-dpi-rescan"
              disabled={dpi.loading}
              onClick={() => void dpi.rescan()}
            >
              {t("device.rescan")}
            </button>
          </div>
        )}

        {info !== null && info.connected && (
          <>
            {!info.capabilities.dpiRead && !info.capabilities.dpiWrite ? (
              <div className="dd-dpi-notice">{t("mouse.dpiNotAvailable")}</div>
            ) : info.dpiMax > 0 ? (
              <DpiCard
                key={info.id}
                info={info}
                dpi={dpi.dpi}
                stages={info.dpiLevels.length > 0 ? info.dpiLevels : dpi.stages}
                fromDevice={info.dpiLevels.length > 0}
                disabled={!DPI_WRITE_ENABLED || !info.capabilities.dpiWrite}
                readOnlyNote={
                  !DPI_WRITE_ENABLED || !info.capabilities.dpiWrite
                    ? t("mouse.dpiReadOnly")
                    : undefined
                }
                onSelectStage={dpi.selectStage}
              />
            ) : (
              <div className="dd-dpi-notice">{t("mouse.dpiRangeUnknown")}</div>
            )}

            {!info.capabilities.pollingRead && !info.capabilities.pollingWrite ? (
              <div className="dd-dpi-notice">{t("mouse.pollingNotAvailable")}</div>
            ) : (
              <PollingCard
                key={info.id}
                info={info}
                disabled={!POLLING_WRITE_ENABLED || !info.capabilities.pollingWrite}
                onChange={dpi.setPolling}
              />
            )}
          </>
        )}

        {info !== null &&
          info.connected &&
          info.capabilities.dpiRead &&
          info.dpiMax <= 0 &&
          (!DPI_WRITE_ENABLED || !info.capabilities.dpiWrite) && (
            <div className="dd-dpi-notice">
              {t("mouse.dpiReadOnly")}
            </div>
          )}

        <div className="dd-dpi-footer">
          <span
            className={
              dpi.error
                ? "dd-dpi-status dd-dpi-status--error"
                : "dd-dpi-status dd-micro"
            }
          >
            {dpi.status}
          </span>

          <div className="dd-dpi-footer-actions">
            <DiagnosticsDialog
              info={info}
              diagnostics={dpi.diagnostics}
              loading={dpi.loading}
              onRefresh={dpi.refresh}
            />

            <button
              type="button"
              className="dd-dpi-ghost"
              disabled={!canReset}
              onClick={dpi.resetToDefault}
            >
              {t("mouse.resetDefault")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
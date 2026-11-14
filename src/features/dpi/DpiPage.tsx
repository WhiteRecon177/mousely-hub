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

export function DpiPage() {
  const t = useT();
  const dpi = useDpi();
  const { info } = dpi;
  const ready = info !== null && info.connected;

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
          <span className="dd-micro">{t("common.device")} · 01</span>
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
          <div className="dd-dev-name">{ready ? info.name : t("device.none")}</div>
          {ready && (
            <div className="dd-micro">
              {info.brand} · {t(CONNECTION_KEYS[info.connection])}
            </div>
          )}
        </div>
      </section>

      <div className="dd-dpi-right">
        <DeviceCard info={info} />

        {info !== null && info.connected && (
          <>
            {!info.capabilities.dpiRead ? (
              <div className="dd-dpi-notice">{t("mouse.dpiNotAvailable")}</div>
            ) : info.dpiMax > 0 ? (
              <DpiCard
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

            {!info.capabilities.pollingRead ? (
              <div className="dd-dpi-notice">{t("mouse.pollingNotAvailable")}</div>
            ) : (
              <PollingCard
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
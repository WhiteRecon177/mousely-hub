import { useEffect, useRef, useState } from "react";
import "../mouse/mouse.css";
import "./dpi.css";
import { useDpi } from "./hooks/useDpi";
import { DeviceCard } from "./components/DeviceCard";
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
  const [copied, setCopied] = useState(false);
  const copyTimer = useRef<number | undefined>(undefined);

  const canReset =
    info !== null &&
    info.connected &&
    DPI_WRITE_ENABLED &&
    POLLING_WRITE_ENABLED &&
    info.capabilities.dpiWrite &&
    info.capabilities.pollingWrite;

  useEffect(
    () => () => {
      window.clearTimeout(copyTimer.current);
    },
    []
  );

  async function copyReport(): Promise<void> {
    const report = [
      `Brand: ${info?.brand ?? "Unknown"}`,
      `Name: ${info?.name ?? "Unknown"}`,
      `Capabilities: ${JSON.stringify(info?.capabilities ?? null)}`,
      "",
      ...dpi.diagnostics,
    ].join("\n");

    try {
      await navigator.clipboard.writeText(report);
      window.clearTimeout(copyTimer.current);
      setCopied(true);
      copyTimer.current = window.setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  }

  return (
    <div className="dd-dpi-page">
      <div className="dd-dpi-left">
        <div className="dd-dpi-mouse">
          <MouseDiagram
            selected={null}
            detected={null}
            enabled={ready || dpi.scanning}
            scanning={dpi.scanning}
            onSelect={noop}
          />
        </div>
      </div>

      <div className="dd-dpi-right">
        <DeviceCard info={info} />

        {info !== null && info.connected && (
          <>
            {!info.capabilities.dpiRead ? (
              <div className="dd-dpi-muted">{t("mouse.dpiNotAvailable")}</div>
            ) : info.dpiMax > 0 ? (
              <DpiCard
                info={info}
                dpi={dpi.dpi}
                stages={info.dpiLevels.length > 0 ? info.dpiLevels : dpi.stages}
                fromDevice={info.dpiLevels.length > 0}
                disabled={!DPI_WRITE_ENABLED || !info.capabilities.dpiWrite}
                onChange={dpi.setDpiLive}
                onSelectStage={dpi.selectStage}
                onAddStage={dpi.addStage}
                onRemoveStage={dpi.removeStage}
              />
            ) : (
              <div className="dd-dpi-muted">{t("mouse.dpiRangeUnknown")}</div>
            )}

            {!info.capabilities.pollingRead ? (
              <div className="dd-dpi-muted">{t("mouse.pollingNotAvailable")}</div>
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
          (!DPI_WRITE_ENABLED || !info.capabilities.dpiWrite) && (
            <div className="dd-dpi-muted">
              {t("mouse.dpiReadOnly")}
            </div>
          )}

        <div className="dd-dpi-footer">
          <span
            className={
              dpi.error
                ? "dd-dpi-status dd-dpi-status--error"
                : "dd-dpi-status"
            }
          >
            {dpi.status}
          </span>

          <div className="dd-dpi-footer-actions">
            <details className="dd-dpi-diag">
              <summary className="dd-dpi-diag-summary">{t("diag.title")}</summary>

              <div className="dd-dpi-diag-panel">
                <div className="dd-dpi-diagnostics-actions">
                  <button
                    type="button"
                    className="dd-dpi-ghost"
                    disabled={dpi.loading}
                    onClick={() => void dpi.refresh()}
                  >
                    {dpi.loading ? t("diag.reading") : t("diag.refresh")}
                  </button>

                  <button
                    type="button"
                    className="dd-dpi-ghost"
                    onClick={() => void copyReport()}
                  >
                    {copied ? t("diag.copied") : t("diag.copyReport")}
                  </button>
                </div>

                <pre className="dd-dpi-diagnostics-log">
                  {dpi.diagnostics.join("\n")}
                </pre>
              </div>
            </details>

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
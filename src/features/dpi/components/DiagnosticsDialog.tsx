import { useEffect, useId, useRef, useState } from "react";
import { MouseInfo } from "../types";
import { useT } from "../../../i18n";

interface DiagnosticsDialogProps {
  info: MouseInfo | null;
  diagnostics: string[];
  loading: boolean;
  onRefresh: () => Promise<void>;
}

let supportsDisplayTransition: boolean | undefined;

function canTransitionDisplay(): boolean {
  if (supportsDisplayTransition !== undefined) return supportsDisplayTransition;
  if (!window.CSS?.supports?.("transition-behavior", "allow-discrete") || !document.body) {
    return false;
  }
  const probe = document.createElement("div");
  probe.style.cssText = "transition: display 1s allow-discrete !important; display: block;";
  document.body.appendChild(probe);
  getComputedStyle(probe).display;
  probe.style.display = "none";
  supportsDisplayTransition = getComputedStyle(probe).display === "block";
  probe.remove();
  return supportsDisplayTransition;
}

function supportsTopLayerExit(): boolean {
  return Boolean(window.CSS?.supports?.("overlay", "auto")) && canTransitionDisplay();
}

async function closeTopLayer(dialog: HTMLDialogElement): Promise<void> {
  if (!dialog.open) return;
  if (!supportsTopLayerExit()) {
    dialog.setAttribute("data-closing", "");
    const animations = dialog.getAnimations({ subtree: true });
    if (animations.length > 0) {
      await Promise.race([
        Promise.allSettled(animations.map((a) => a.finished)),
        new Promise<void>((resolve) => window.setTimeout(resolve, 400)),
      ]);
    }
    if (!dialog.hasAttribute("data-closing")) return;
    dialog.removeAttribute("data-closing");
  }
  if (dialog.open) dialog.close();
}

export function DiagnosticsDialog({
  info,
  diagnostics,
  loading,
  onRefresh,
}: DiagnosticsDialogProps) {
  const t = useT();
  const titleId = useId();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [copied, setCopied] = useState(false);
  const copyTimer = useRef<number | undefined>(undefined);

  useEffect(
    () => () => {
      window.clearTimeout(copyTimer.current);
    },
    []
  );

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    const onCancel = (event: Event) => {
      event.preventDefault();
      void closeTopLayer(dialog);
    };
    dialog.addEventListener("cancel", onCancel);

    // Fallback light dismiss when native `closedby="any"` is unavailable.
    // Require both press and release outside the dialog bounds so text-selection drags in the log don't close.
    let cleanupOutside: (() => void) | undefined;
    if (!("closedBy" in HTMLDialogElement.prototype)) {
      let pressedOutside = false;
      const isOutside = (e: PointerEvent): boolean => {
        const rect = dialog.getBoundingClientRect();
        return (
          e.clientX < rect.left ||
          e.clientX > rect.right ||
          e.clientY < rect.top ||
          e.clientY > rect.bottom
        );
      };
      const onPointerDown = (e: PointerEvent) => {
        if (!dialog.open) return;
        pressedOutside = isOutside(e);
      };
      const onPointerUp = (e: PointerEvent) => {
        if (pressedOutside && dialog.open && isOutside(e)) {
          void closeTopLayer(dialog);
        }
        pressedOutside = false;
      };
      dialog.addEventListener("pointerdown", onPointerDown);
      dialog.addEventListener("pointerup", onPointerUp);
      cleanupOutside = () => {
        dialog.removeEventListener("pointerdown", onPointerDown);
        dialog.removeEventListener("pointerup", onPointerUp);
      };
    }

    return () => {
      dialog.removeEventListener("cancel", onCancel);
      cleanupOutside?.();
    };
  }, []);

  function openDialog(): void {
    const dialog = dialogRef.current;
    if (!dialog) return;
    dialog.removeAttribute("data-closing");
    if (!dialog.open) {
      dialog.showModal();
    }
  }

  function closeDialog(): void {
    const dialog = dialogRef.current;
    if (!dialog) return;
    void closeTopLayer(dialog);
  }

  async function copyReport(): Promise<void> {
    const report = [
      `Brand: ${info?.brand ?? "Unknown"}`,
      `Name: ${info?.name ?? "Unknown"}`,
      `Capabilities: ${JSON.stringify(info?.capabilities ?? null)}`,
      "",
      ...diagnostics,
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

  const deviceSummary = info?.connected ? `${info.brand} · ${info.name}` : t("device.none");
  const logText =
    diagnostics.length > 0 ? diagnostics.join("\n") : loading ? t("diag.reading") : "—";

  return (
    <>
      <button
        type="button"
        className="dd-dpi-ghost dd-dpi-diag-trigger"
        onClick={openDialog}
      >
        {t("diag.title")}
      </button>

      <dialog
        ref={dialogRef}
        className="dd-diag-dialog"
        aria-labelledby={titleId}
        {...({ closedby: "any" } as Record<string, string>)}
      >
        <div className="dd-diag-head">
          <div className="dd-diag-head-text">
            <h2 id={titleId} className="dd-diag-title">
              {t("diag.title")}
            </h2>
            <div className="dd-micro dd-diag-sub">{deviceSummary}</div>
          </div>
          <button
            type="button"
            className="dd-diag-close"
            aria-label={t("common.close")}
            title={t("common.close")}
            onClick={closeDialog}
          >
            <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
              <path
                d="M2 2l10 10M12 2L2 12"
                stroke="currentColor"
                strokeWidth="1.6"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>

        <div className="dd-diag-actions">
          <button
            type="button"
            className="dd-diag-btn"
            disabled={loading}
            onClick={() => void onRefresh()}
          >
            {loading ? t("diag.reading") : t("diag.refresh")}
          </button>

          <button
            type="button"
            className={copied ? "dd-diag-btn dd-diag-btn--copied" : "dd-diag-btn"}
            onClick={() => void copyReport()}
          >
            {copied ? t("diag.copied") : t("diag.copyReport")}
          </button>
        </div>

        <pre className="dd-diag-log" tabIndex={0}>
          {logText}
        </pre>
      </dialog>
    </>
  );
}

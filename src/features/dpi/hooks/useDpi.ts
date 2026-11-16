import { useCallback, useEffect, useRef, useState } from "react";
import { ConnectionType, MouseInfo } from "../types";
import { listen } from "@tauri-apps/api/event";
import { dpiApi, type ProbeResult } from "../services/dpiApi";
import { translate } from "../../../i18n";

const DEFAULT_STAGES = [400, 800, 1600, 3200];
const DEFAULT_DPI = 800;
const DEFAULT_POLLING = 1000;
const STARTUP_RETRY_AT_MS = [1500, 3000, 5000] as const;
const LIVE_SYNC_MS = 3000;

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function normalize(value: number, info: MouseInfo): number {
  const step = Math.max(1, info.dpiStep || 1);
  const stepped = Math.round(value / step) * step;
  return Math.min(info.dpiMax, Math.max(info.dpiMin, stepped));
}

function needsStartupRetry(info: MouseInfo): boolean {
  return !info.connected || (!info.capabilities.dpiRead && !info.capabilities.dpiWrite);
}

function hasHardwareRead(info: MouseInfo | null): boolean {
  if (!info || !info.connected) return false;
  return (
    info.capabilities.dpiRead ||
    info.capabilities.pollingRead ||
    info.capabilities.battery
  );
}

export function useDpi(active = true) {
  const cached = dpiApi.cached();
  const [info, setInfo] = useState<MouseInfo | null>(() => (cached ? cached.info : null));
  const [devices, setDevices] = useState<MouseInfo[]>(() => (cached ? cached.devices : []));
  const [activeId, setActiveId] = useState<string | null>(() => (cached ? cached.activeId : null));
  const [dpi, setDpi] = useState<number>(() => (cached ? cached.info.dpi : 0));
  const [status, setStatus] = useState<string>("");
  const [error, setError] = useState<boolean>(false);
  const [diagnostics, setDiagnostics] = useState<string[]>(() => (cached ? cached.diagnostics : []));
  const [loading, setLoading] = useState<boolean>(false);
  // The scan animation runs only while the mouse is being looked for
  const [scanning, setScanning] = useState<boolean>(() => dpiApi.cached() === null);

  const startedAt = useRef<number>(Date.now());
  const timer = useRef<number | undefined>(undefined);
  const scanTimer = useRef<number | undefined>(undefined);
  const lastGood = useRef<number>(cached && cached.info.dpi > 0 ? cached.info.dpi : 0);
  const busy = useRef<boolean>(false);
  const applySeq = useRef<number>(0);
  const startupCancelled = useRef<boolean>(false);
  const startupWakeups = useRef<Set<() => void>>(new Set());
  const lastFreshAt = useRef<number>(0);
  const mountedRef = useRef<boolean>(true);
  const prevActive = useRef<boolean>(active);

  const cancelStartupRetry = useCallback(() => {
    startupCancelled.current = true;
    startupWakeups.current.forEach((wake) => wake());
    startupWakeups.current.clear();
  }, []);

  const stopScanAfterPass = useCallback((minDurationMs = 1500) => {
    window.clearTimeout(scanTimer.current);
    const left = Math.max(0, minDurationMs - (Date.now() - startedAt.current));
    if (left === 0) {
      setScanning(false);
    } else {
      scanTimer.current = window.setTimeout(() => {
        if (mountedRef.current) setScanning(false);
      }, left);
    }
  }, []);

  const applyProbeResult = useCallback((result: ProbeResult) => {
    setInfo(result.info);
    setDevices(result.devices);
    setActiveId(result.activeId);
    setDiagnostics(result.diagnostics);
    if (result.info.connected) {
      setDpi(result.info.dpi);
      if (result.info.dpi > 0) {
        lastGood.current = result.info.dpi;
      }
    } else {
      setDpi(0);
    }
  }, []);

  const syncQuiet = useCallback(async () => {
    if (busy.current) return;
    const seq = applySeq.current;
    try {
      const result = await dpiApi.probe(true);
      lastFreshAt.current = Date.now();
      if (!mountedRef.current || busy.current || applySeq.current !== seq) return;
      applyProbeResult(result);
      setError((prevErr) => {
        if (prevErr) setStatus("");
        return false;
      });
    } catch {
      // Background alignment should not disturb the user on transient failures
    }
  }, [applyProbeResult]);

  const refresh = useCallback(async () => {
    cancelStartupRetry();
    const seq = ++applySeq.current;
    setLoading(true);
    try {
      const result = await dpiApi.probe(true);
      lastFreshAt.current = Date.now();
      if (!mountedRef.current || applySeq.current !== seq) return;
      applyProbeResult(result);
      setStatus("");
      setError(false);
    } catch (e) {
      if (!mountedRef.current || applySeq.current !== seq) return;
      setStatus(errorText(e));
      setError(true);
    } finally {
      if (mountedRef.current) {
        setLoading(false);
        setScanning(false);
      }
    }
  }, [applyProbeResult, cancelStartupRetry]);

  const rescan = useCallback(async () => {
    cancelStartupRetry();
    const seq = ++applySeq.current;
    startedAt.current = Date.now();
    window.clearTimeout(scanTimer.current);
    setScanning(true);
    setLoading(true);
    setError(false);
    setStatus("");
    try {
      const result = await dpiApi.probe(true);
      lastFreshAt.current = Date.now();
      if (!mountedRef.current || applySeq.current !== seq) return;
      applyProbeResult(result);
      setStatus("");
      setError(false);
    } catch (e) {
      if (!mountedRef.current || applySeq.current !== seq) return;
      setStatus(errorText(e));
      setError(true);
    } finally {
      if (mountedRef.current) {
        setLoading(false);
        stopScanAfterPass(1200);
      }
    }
  }, [applyProbeResult, cancelStartupRetry, stopScanAfterPass]);

  const selectDevice = useCallback(
    async (id: string) => {
      if (busy.current || id === activeId) return;
      cancelStartupRetry();
      window.clearTimeout(timer.current);
      window.clearTimeout(scanTimer.current);
      const seq = ++applySeq.current;
      busy.current = true;
      setLoading(true);
      setScanning(false);
      setError(false);
      setStatus("");
      try {
        const result = await dpiApi.selectActive(id);
        lastFreshAt.current = Date.now();
        if (!mountedRef.current || applySeq.current !== seq) return;
        applyProbeResult(result);
      } catch (e) {
        if (!mountedRef.current || applySeq.current !== seq) return;
        setStatus(errorText(e));
        setError(true);
      } finally {
        busy.current = false;
        if (mountedRef.current) setLoading(false);
      }
    },
    [activeId, applyProbeResult, cancelStartupRetry]
  );

  // Mount: instant shared backend cache, immediate fresh probe, ~5s startup retries, and backend events
  useEffect(() => {
    mountedRef.current = true;
    startupCancelled.current = false;
    let alive = true;
    const stops: Array<() => void> = [];

    const waitRetry = (ms: number): Promise<boolean> =>
      new Promise((resolve) => {
        if (!alive || startupCancelled.current) {
          resolve(false);
          return;
        }
        let done = false;
        const finish = (ok: boolean) => {
          if (done) return;
          done = true;
          window.clearTimeout(id);
          startupWakeups.current.delete(wake);
          resolve(ok);
        };
        const wake = () => finish(false);
        const id = window.setTimeout(() => finish(true), ms);
        startupWakeups.current.add(wake);
      });

    const runStartup = async () => {
      const seq = applySeq.current;
      let resolvedFast = false;

      // 1) Populate immediately from backend shared cache (e.g. tray already probed at boot)
      try {
        const fast = await dpiApi.probe(false);
        if (!alive || busy.current || applySeq.current !== seq) return;
        applyProbeResult(fast);
        if (!needsStartupRetry(fast.info)) {
          resolvedFast = true;
        }
      } catch {
        // Proceed straight to fresh probe
      }

      // 2) Immediately trigger fresh hardware probe to align with live hardware
      let fresh: ProbeResult | null = null;
      try {
        fresh = await dpiApi.probe(true);
        lastFreshAt.current = Date.now();
        if (!alive || busy.current || applySeq.current !== seq || startupCancelled.current) return;
        applyProbeResult(fresh);
        setStatus("");
        setError(false);
        if (!needsStartupRetry(fresh.info)) {
          stopScanAfterPass(resolvedFast ? 900 : 1500);
          return;
        }
      } catch (e) {
        if (!alive || applySeq.current !== seq || startupCancelled.current) return;
        setStatus(errorText(e));
        setError(true);
      }

      // 3) Startup brief auto-retry over first ~5s if no readable/connected mouse yet
      for (const targetMs of STARTUP_RETRY_AT_MS) {
        if (!alive || startupCancelled.current || applySeq.current !== seq) break;
        const delay = Math.max(250, targetMs - (Date.now() - startedAt.current));
        const proceed = await waitRetry(delay);
        if (!proceed || !alive || startupCancelled.current || applySeq.current !== seq) break;
        if (busy.current) continue;
        try {
          const retry = await dpiApi.probe(true);
          lastFreshAt.current = Date.now();
          if (!alive || busy.current || applySeq.current !== seq || startupCancelled.current) break;
          applyProbeResult(retry);
          setStatus("");
          setError(false);
          if (!needsStartupRetry(retry.info)) {
            stopScanAfterPass(1200);
            return;
          }
        } catch {
          // Keep retrying within the ~5s startup window
        }
      }

      if (alive && !startupCancelled.current && applySeq.current === seq) {
        setScanning(false);
      }
    };

    void runStartup();

    const keep = (pending: Promise<() => void>) => {
      void pending.then((off) => {
        if (alive) stops.push(off);
        else off();
      });
    };

    // Instant guess (cable plugged or unplugged) ...
    keep(
      listen<{ connection: ConnectionType; charging: boolean }>("mouse-quick", (event) => {
        setInfo((prev) =>
          prev
            ? { ...prev, connection: event.payload.connection, charging: event.payload.charging }
            : prev
        );
      })
    );

    // ... and the full reading a moment later.
    keep(
      listen<ProbeResult>("mouse-changed", (event) => {
        if (busy.current) return;
        const result = event.payload;
        dpiApi.setFromEvent(result);
        applyProbeResult(result);
        if (result.info.connected && !needsStartupRetry(result.info)) {
          cancelStartupRetry();
          setScanning(false);
        }
      })
    );

    return () => {
      alive = false;
      mountedRef.current = false;
      cancelStartupRetry();
      stops.forEach((off) => off());
      window.clearTimeout(timer.current);
      window.clearTimeout(scanTimer.current);
    };
  }, [applyProbeResult, cancelStartupRetry, stopScanAfterPass]);

  // Focus / visibility / tab-switch live alignment
  useEffect(() => {
    const triggerAlign = () => {
      if (!active || document.hidden || busy.current) return;
      if (Date.now() - lastFreshAt.current < 400) return;
      void syncQuiet();
    };

    const onVisibility = () => {
      if (document.visibilityState === "visible") triggerAlign();
    };

    window.addEventListener("focus", triggerAlign);
    document.addEventListener("visibilitychange", onVisibility);

    if (active && !prevActive.current) {
      triggerAlign();
    }
    prevActive.current = active;

    return () => {
      window.removeEventListener("focus", triggerAlign);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [active, syncQuiet]);

  // Lightweight periodic sync (~3s) only while visible, focused, connected, and readable
  const readable = hasHardwareRead(info);
  useEffect(() => {
    if (!active || !readable) return;
    const id = window.setInterval(() => {
      if (document.hidden || !document.hasFocus() || busy.current) return;
      void syncQuiet();
    }, LIVE_SYNC_MS);
    return () => window.clearInterval(id);
  }, [active, readable, syncQuiet]);

  const apply = useCallback(
    async (value: number) => {
      cancelStartupRetry();
      applySeq.current += 1;
      busy.current = true;
      setStatus(translate("common.applying"));
      setError(false);
      try {
        const applied = await dpiApi.setDpi(value);
        if (applied > 0) lastGood.current = applied;
        setDpi(applied);
        setInfo((prev) => {
          if (!prev) return prev;
          const nextSource = applied > 0 ? (prev.capabilities.dpiRead ? "hardware" : "saved") : "unset";
          return { ...prev, dpi: applied, dpiSource: nextSource };
        });
        setDevices((prev) =>
          prev.map((dev) =>
            dev.id === info?.id
              ? {
                  ...dev,
                  dpi: applied,
                  dpiSource: applied > 0 ? (dev.capabilities.dpiRead ? "hardware" : "saved") : "unset",
                }
              : dev
          )
        );
        setStatus(translate("common.applied"));
      } catch (e) {
        // the mouse refused: show the value it really has
        setDpi(lastGood.current);
        setStatus(errorText(e));
        setError(true);
      } finally {
        busy.current = false;
      }
    },
    [cancelStartupRetry, info?.id]
  );

  const setDpiLive = useCallback(
    (value: number) => {
      if (!info) return;
      const next = normalize(value, info);
      setDpi(next);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => {
        void apply(next);
      }, 250);
    },
    [info, apply]
  );

  const selectStage = useCallback(
    (value: number) => {
      if (!info) return;
      const next = normalize(value, info);
      window.clearTimeout(timer.current);
      setDpi(next);
      void apply(next);
    },
    [info, apply]
  );

  const setPolling = useCallback(
    async (hz: number) => {
      if (!info || busy.current || hz === info.pollingRate) return;
      if (info.pollingSupported.length > 0 && !info.pollingSupported.includes(hz)) return;
      cancelStartupRetry();
      applySeq.current += 1;
      const previous = info.pollingRate;
      const previousSource = info.pollingSource;
      const nextSource = info.capabilities.pollingRead ? "hardware" : "saved";
      busy.current = true;
      // show the new value at once, undo it only if the mouse refuses
      setInfo((prev) => (prev ? { ...prev, pollingRate: hz, pollingSource: nextSource } : prev));
      setDevices((prev) =>
        prev.map((dev) =>
          dev.id === info.id ? { ...dev, pollingRate: hz, pollingSource: nextSource } : dev
        )
      );
      setStatus(translate("common.applying"));
      setError(false);
      try {
        await dpiApi.setPollingRate(hz);
        setStatus(translate("common.applied"));
      } catch (e) {
        setInfo((prev) =>
          prev ? { ...prev, pollingRate: previous, pollingSource: previousSource } : prev
        );
        setDevices((prev) =>
          prev.map((dev) =>
            dev.id === info.id
              ? { ...dev, pollingRate: previous, pollingSource: previousSource }
              : dev
          )
        );
        setStatus(errorText(e));
        setError(true);
      } finally {
        busy.current = false;
      }
    },
    [cancelStartupRetry, info]
  );

  const resetToDefault = useCallback(async () => {
    if (!info || busy.current) return;
    cancelStartupRetry();
    applySeq.current += 1;
    window.clearTimeout(timer.current);
    busy.current = true;
    setStatus(translate("common.applying"));
    setError(false);
    const resetDpi = normalize(DEFAULT_DPI, info);
    const pollingPool =
      info.pollingSupported.length > 0 ? info.pollingSupported : info.pollingOptions;
    const resetPolling = pollingPool.includes(DEFAULT_POLLING)
      ? DEFAULT_POLLING
      : pollingPool.length > 0
      ? Math.max(...pollingPool)
      : DEFAULT_POLLING;
    try {
      const appliedDpi = await dpiApi.setDpi(resetDpi);
      await dpiApi.setPollingRate(resetPolling);
      if (appliedDpi > 0) lastGood.current = appliedDpi;
      setDpi(appliedDpi);
      setInfo((prev) =>
        prev
          ? {
              ...prev,
              dpi: appliedDpi,
              dpiSource: prev.capabilities.dpiRead ? "hardware" : "saved",
              pollingRate: resetPolling,
              pollingSource: prev.capabilities.pollingRead ? "hardware" : "saved",
            }
          : prev
      );
      setDevices((prev) =>
        prev.map((dev) =>
          dev.id === info.id
            ? {
                ...dev,
                dpi: appliedDpi,
                dpiSource: dev.capabilities.dpiRead ? "hardware" : "saved",
                pollingRate: resetPolling,
                pollingSource: dev.capabilities.pollingRead ? "hardware" : "saved",
              }
            : dev
        )
      );
      setStatus(translate("common.applied"));
    } catch (e) {
      setStatus(errorText(e));
      setError(true);
    } finally {
      busy.current = false;
    }
  }, [cancelStartupRetry, info]);

  return {
    info,
    devices,
    activeId,
    dpi,
    stages: DEFAULT_STAGES,
    status,
    error,
    setDpiLive,
    selectStage,
    setPolling,
    resetToDefault,
    selectDevice,
    rescan,
    diagnostics,
    scanning,
    loading,
    refresh,
  };
}
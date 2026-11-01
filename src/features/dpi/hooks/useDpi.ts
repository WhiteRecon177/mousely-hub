import { useCallback, useEffect, useRef, useState } from "react";
import { ConnectionType, MouseInfo } from "../types";
import { listen } from "@tauri-apps/api/event";
import { dpiApi, type ProbeResult } from "../services/dpiApi";
import { translate } from "../../../i18n";

const STORAGE_KEY = "devicedeck.dpi.stages";
const DEFAULT_STAGES = [400, 800, 1600, 3200, 6400];
const MAX_STAGES = 6;
const DEFAULT_DPI = 800;
const DEFAULT_POLLING = 1000;

function loadStages(): number[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_STAGES;
    const parsed: unknown = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      const numbers = parsed.filter((v): v is number => typeof v === "number");
      if (numbers.length > 0) return numbers;
    }
  } catch {
    // ignore, use defaults
  }
  return DEFAULT_STAGES;
}

function saveStages(stages: number[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(stages));
  } catch {
    // storage not available
  }
}

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function normalize(value: number, info: MouseInfo): number {
  const stepped = Math.round(value / info.dpiStep) * info.dpiStep;
  return Math.min(info.dpiMax, Math.max(info.dpiMin, stepped));
}

export function useDpi() {
  const cached = dpiApi.cached();
  const [info, setInfo] = useState<MouseInfo | null>(() => (cached ? cached.info : null));
  const [dpi, setDpi] = useState<number>(() => (cached ? cached.info.dpi : DEFAULT_DPI));
  const [stages, setStages] = useState<number[]>(() => loadStages());
  const [status, setStatus] = useState<string>("");
  const [error, setError] = useState<boolean>(false);
  const [diagnostics, setDiagnostics] = useState<string[]>(() => (cached ? cached.diagnostics : []));
  const [loading, setLoading] = useState<boolean>(false);
  // The scan animation runs only while the mouse is being looked for (first visit)
  const [scanning, setScanning] = useState<boolean>(() => dpiApi.cached() === null);
  const startedAt = useRef<number>(Date.now());
  const timer = useRef<number | undefined>(undefined);
  const lastGood = useRef<number>(DEFAULT_DPI);
  const busy = useRef<boolean>(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await dpiApi.probe();
      setInfo(result.info);
      setDiagnostics(result.diagnostics);
      if (result.info.connected) {
        setDpi(result.info.dpi);
        lastGood.current = result.info.dpi;
      }
      setStatus("");
      setError(false);
    } catch (e) {
      setStatus(errorText(e));
      setError(true);
    } finally {
      setLoading(false);
      // let the scan finish one full pass, then stop it
      const left = Math.max(0, 1800 - (Date.now() - startedAt.current));
      window.setTimeout(() => setScanning(false), left);
    }
  }, []);

  useEffect(() => {
    // Coming back to the tab: use the saved reading, refresh only if it is old
    if (dpiApi.cached() === null || dpiApi.cacheAge() > 30000) void refresh();

    // The backend tells us at once when the mouse changes (cable, receiver...), without polling.
    let alive = true;
    const stops: Array<() => void> = [];
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
        setInfo(result.info);
        setDiagnostics(result.diagnostics);
        if (result.info.connected) {
          setDpi(result.info.dpi);
          lastGood.current = result.info.dpi;
        }
      })
    );

    return () => {
      alive = false;
      stops.forEach((off) => off());
      window.clearTimeout(timer.current);
    };
  }, [refresh]);

  const apply = useCallback(async (value: number) => {
    busy.current = true;
    setStatus(translate("common.applying"));
    setError(false);
    try {
      const applied = await dpiApi.setDpi(value);
      lastGood.current = applied;
      setDpi(applied);
      setInfo((prev) => (prev ? { ...prev, dpi: applied } : prev));
      setStatus(translate("common.applied"));
    } catch (e) {
      // the mouse refused: show the value it really has
      setDpi(lastGood.current);
      setStatus(errorText(e));
      setError(true);
    } finally {
      busy.current = false;
    }
  }, []);

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

  const addStage = useCallback(() => {
    if (stages.includes(dpi) || stages.length >= MAX_STAGES) return;
    const next = [...stages, dpi].sort((a, b) => a - b);
    setStages(next);
    saveStages(next);
  }, [stages, dpi]);

  const removeStage = useCallback(
    (value: number) => {
      if (stages.length <= 1) return;
      const next = stages.filter((s) => s !== value);
      setStages(next);
      saveStages(next);
    },
    [stages]
  );

  const setPolling = useCallback(
    async (hz: number) => {
      if (!info || busy.current || hz === info.pollingRate) return;
      if (info.pollingSupported.length > 0 && !info.pollingSupported.includes(hz)) return;
      const previous = info.pollingRate;
      busy.current = true;
      // show the new value at once, undo it only if the mouse refuses
      setInfo((prev) => (prev ? { ...prev, pollingRate: hz } : prev));
      setStatus(translate("common.applying"));
      setError(false);
      try {
        await dpiApi.setPollingRate(hz);
        setStatus(translate("common.applied"));
      } catch (e) {
        setInfo((prev) => (prev ? { ...prev, pollingRate: previous } : prev));
        setStatus(errorText(e));
        setError(true);
      } finally {
        busy.current = false;
      }
    },
    [info]
  );

  const resetToDefault = useCallback(async () => {
    if (!info) return;
    window.clearTimeout(timer.current);
    setStatus(translate("common.applying"));
    setError(false);
    const resetDpi = normalize(DEFAULT_DPI, info);
    const resetPolling = info.pollingSupported.includes(DEFAULT_POLLING)
      ? DEFAULT_POLLING
      : Math.max(...info.pollingSupported);
    try {
      await dpiApi.setDpi(resetDpi);
      await dpiApi.setPollingRate(resetPolling);
      setDpi(resetDpi);
      setInfo((prev) =>
        prev ? { ...prev, dpi: resetDpi, pollingRate: resetPolling } : prev
      );
      setStatus(translate("common.applied"));
    } catch (e) {
      setStatus(errorText(e));
      setError(true);
    }
  }, [info]);

  return {
    info,
    dpi,
    stages,
    status,
    error,
    setDpiLive,
    selectStage,
    addStage,
    removeStage,
    setPolling,
    resetToDefault,
    diagnostics,
    scanning,
    loading,
    refresh,
  };
}
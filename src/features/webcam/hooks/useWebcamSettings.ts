import { useState, useEffect, useRef, useCallback } from "react";
import type { CameraSnapshot, ControlName, ControlRange, ModeName, ModeValue } from "../types";
import { translate } from "../../../i18n";
import {
  applyControl,
  applyFps,
  applyMode,
  applySnapshot,
  autoAdjust,
  getCurrentFps,
  getFpsOptions,
  getModes,
  getSupportedControls,
  readSnapshot,
  whenIdle,
} from "../services/cameraControls";
import {
  isAutosaveEnabled,
  loadAutosave,
  saveAutosave,
  setAutosaveEnabled,
} from "../services/profileStorage";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export function useWebcamSettings(track: MediaStreamTrack | null, cameraLabel: string) {
  const [controls, setControls] = useState<ControlRange[]>([]);
  const [fps, setFps] = useState<number | null>(null);
  const [fpsOptions, setFpsOptions] = useState<number[]>([]);
  const [modes, setModes] = useState<Partial<Record<ModeName, ModeValue>>>({});
  const [mirrored, setMirrored] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [autosave, setAutosave] = useState<boolean>(() => isAutosaveEnabled());
  const startRef = useRef<CameraSnapshot | null>(null);
  const readyRef = useRef(false);

  // Re-read the sliders and fps from the camera
  const refresh = useCallback(() => {
    if (!track) return;
    setControls(getSupportedControls(track));
    setFps(getCurrentFps(track));
    setModes(getModes(track));
  }, [track]);

  // Current state of the camera, ready to be saved
  const buildSnapshot = useCallback((): CameraSnapshot | null => {
    if (!track) return null;
    const base = readSnapshot(track, mirrored);
    const values: CameraSnapshot["values"] = {};
    for (const c of controls) values[c.name] = c.value;
    return { ...base, values, fps: fps ?? base.fps };
  }, [track, controls, mirrored, fps]);

  // When a camera opens: remember its start values, then restore the last saved ones
  useEffect(() => {
    readyRef.current = false;
    if (!track) {
      setControls([]);
      setFpsOptions([]);
      setFps(null);
      setModes({});
      startRef.current = null;
      return;
    }

    let cancelled = false;
    startRef.current = readSnapshot(track, false);
    setFpsOptions(getFpsOptions(track));
    refresh();

    const restore = async () => {
      const saved = isAutosaveEnabled() ? loadAutosave(cameraLabel) : null;
      if (saved) {
        await applySnapshot(track, saved);
        await wait(300);
        if (cancelled) return;
        setMirrored(saved.mirrored);
        refresh();
        setStatus(translate("cam.status.lastRestored"));
      } else {
        setStatus(null);
      }
      if (!cancelled) readyRef.current = true;
    };
    void restore();

    return () => {
      cancelled = true;
    };
  }, [track, cameraLabel, refresh]);

  // Auto-save: store the settings half a second after the last change
  useEffect(() => {
    if (!track || !autosave || !readyRef.current || controls.length === 0) return;
    const timer = setTimeout(async () => {
      await whenIdle();
      const snapshot = buildSnapshot();
      if (snapshot) saveAutosave(cameraLabel, snapshot);
    }, 500);
    return () => clearTimeout(timer);
  }, [track, autosave, controls, buildSnapshot, cameraLabel]);

  const changeControl = useCallback(
    (name: ControlName, value: number) => {
      if (!track) return;
      setControls((prev) => prev.map((c) => (c.name === name ? { ...c, value } : c)));
      applyControl(track, name, value).catch((err) =>
        console.error("Failed to apply control:", err)
      );
    },
    [track]
  );

  // Switches a mode between automatic and manual. The switch moves at once,
  // then the camera is asked and the real state is read back.
  const changeMode = useCallback(
    async (mode: ModeName, value: ModeValue) => {
      if (!track) return;
      setModes((prev) => ({ ...prev, [mode]: value }));
      try {
        await applyMode(track, mode, value);
        await wait(400);
        refresh();
        const now = getModes(track)[mode];
        setStatus(
          now === value || now === undefined
            ? null
            : translate("cam.status.modeRefused")
        );
      } catch (err) {
        console.error("Failed to change mode:", err);
        refresh();
        setStatus(translate("cam.status.notAllowed"));
      }
    },
    [track, refresh]
  );

  const changeFps = useCallback(
    async (value: number) => {
      if (!track) return;
      setFps(value);
      try {
        await applyFps(track, value);
        setFps(getCurrentFps(track));
        setStatus(null);
      } catch (err) {
        console.error("Failed to set fps:", err);
        setFps(getCurrentFps(track));
        setStatus(translate("cam.status.fpsUnavailable"));
      }
    },
    [track]
  );

  // Restart automatic modes, then read the values the camera chose
  const runAuto = useCallback(async () => {
    if (!track) return;
    setStatus(translate("cam.status.adjusting"));
    const applied = await autoAdjust(track);
    await wait(800);
    refresh();
    await wait(1200);
    refresh();
    setStatus(
      applied.length > 0
        ? translate("cam.status.autoDone")
        : translate("cam.status.resetNoAuto")
    );
  }, [track, refresh]);

  const restoreStart = useCallback(async () => {
    if (!track || !startRef.current) return;
    await applySnapshot(track, startRef.current);
    await wait(300);
    refresh();
    setStatus(translate("cam.status.startRestored"));
  }, [track, refresh]);

  const applyProfileSnapshot = useCallback(
    async (snapshot: CameraSnapshot) => {
      if (!track) return;
      setStatus(translate("cam.status.applyingProfile"));
      await applySnapshot(track, snapshot);
      setMirrored(snapshot.mirrored);
      await wait(300);
      refresh();
      setStatus(translate("cam.status.profileApplied"));
    },
    [track, refresh]
  );

  const changeAutosave = useCallback((value: boolean) => {
    setAutosave(value);
    setAutosaveEnabled(value);
  }, []);

  return {
    controls,
    fps,
    fpsOptions,
    mirrored,
    setMirrored,
    modes,
    changeMode,
    status,
    autosave,
    changeAutosave,
    changeControl,
    changeFps,
    runAuto,
    restoreStart,
    applyProfileSnapshot,
    buildSnapshot,
  };
}
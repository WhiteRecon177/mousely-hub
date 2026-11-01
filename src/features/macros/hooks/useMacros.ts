import { useCallback, useEffect, useRef, useState } from "react";
import { macroApi } from "../services/macroApi";
import { loadMacros, saveMacros } from "../services/macroStorage";
import { MacroAction, MacroStatus, SavedMacro } from "../types";
import { translate } from "../../../i18n";

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function useMacros() {
  const [status, setStatus] = useState<MacroStatus>("idle");
  const [recorded, setRecorded] = useState<MacroAction[]>([]);
  const [saved, setSaved] = useState<SavedMacro[]>(() => loadMacros());
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const timer = useRef<number | undefined>(undefined);

  const flash = useCallback((text: string) => {
    setMessage(text);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setMessage(null), 3000);
  }, []);

  useEffect(() => {
    let alive = true;
    const offs: Array<() => void> = [];
    const add = (p: Promise<() => void>) => {
      p.then((off) => {
        if (alive) offs.push(off);
        else off();
      }).catch(() => {});
    };

    add(
      macroApi.onRecordingStopped((actions) => {
        setRecorded(actions);
        setStatus("idle");
        flash(translate("macro.recordingStopped"));
      })
    );
    add(
      macroApi.onPlaybackFinished(() => {
        setStatus("idle");
        flash(translate("macro.playbackFinished"));
      })
    );

    return () => {
      alive = false;
      offs.forEach((off) => off());
      window.clearTimeout(timer.current);
    };
  }, [flash]);

  const record = useCallback(async () => {
    try {
      setError(null);
      await macroApi.startRecording();
      setRecorded([]);
      setStatus("recording");
    } catch (e) {
      setError(errorText(e));
    }
  }, []);

  const stopRecording = useCallback(async () => {
    try {
      const actions = await macroApi.stopRecording();
      setRecorded(actions);
      setStatus("idle");
    } catch (e) {
      setError(errorText(e));
    }
  }, []);

  const play = useCallback(async (actions: MacroAction[], repeat: number) => {
    try {
      setError(null);
      setStatus("playing");
      await macroApi.playMacro(actions, repeat);
    } catch (e) {
      setError(errorText(e));
      setStatus("idle");
    }
  }, []);

  const stopPlayback = useCallback(async () => {
    try {
      await macroApi.stopPlayback();
    } catch (e) {
      setError(errorText(e));
    }
    setStatus("idle");
  }, []);

  const save = useCallback(
    (name: string): string | undefined => {
      const trimmed = name.trim();
      if (!trimmed || recorded.length === 0) return undefined;
      const existing = saved.find((m) => m.name === trimmed);
      const id = existing ? existing.id : Date.now().toString();
      const macro: SavedMacro = { id, name: trimmed, actions: [...recorded] };
      const updated = existing ? saved.map((m) => (m.id === id ? macro : m)) : [...saved, macro];
      setSaved(updated);
      saveMacros(updated);
      flash(translate("macro.saved", { name: trimmed }));
      return id;
    },
    [recorded, saved, flash]
  );

  const remove = useCallback(
    (id: string) => {
      const updated = saved.filter((m) => m.id !== id);
      setSaved(updated);
      saveMacros(updated);
    },
    [saved]
  );

  const editMacro = useCallback((macro: SavedMacro) => {
    setRecorded([...macro.actions]);
  }, []);

  const removeAction = useCallback((index: number) => {
    setRecorded((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearRecorded = useCallback(() => {
    setRecorded([]);
  }, []);

  return {
    status,
    recorded,
    saved,
    error,
    message,
    record,
    stopRecording,
    play,
    stopPlayback,
    save,
    remove,
    editMacro,
    removeAction,
    clearRecorded,
  };
}
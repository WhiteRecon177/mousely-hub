import { useCallback, useEffect, useState } from "react";
import { ButtonId, RemapAction, RemapState } from "../types";
import { mouseApi } from "../services/mouseApi";
import { loadRemapState, saveRemapState } from "../services/mouseStorage";
import { syncRemap } from "../services/remapSync";
import { translate } from "../../../i18n";

const BUTTON_IDS: ButtonId[] = ["left", "right", "middle", "back", "forward"];

function isButtonId(value: string): value is ButtonId {
  return (BUTTON_IDS as string[]).includes(value);
}

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function useMouseRemap() {
  const [state, setState] = useState<RemapState>(() => loadRemapState());
  const [selected, setSelected] = useState<ButtonId>("back");
  const [detecting, setDetecting] = useState(false);
  const [detected, setDetected] = useState<ButtonId | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

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
      mouseApi.onDetected((button) => {
        setDetecting(false);
        if (!isButtonId(button)) return;
        setDetected(button);
        setSelected(button);
        setNotice(
          button === "left" || button === "right"
            ? translate("notice.protected")
            : translate("notice.detected", { button })
        );
        setTimeout(() => setDetected(null), 2500);
      })
    );

    add(
      mouseApi.onDisabled(() => {
        setState((prev) => ({ ...prev, enabled: false }));
        setNotice(translate("notice.remapOff"));
      })
    );

    return () => {
      alive = false;
      offs.forEach((off) => off());
      mouseApi.cancelDetect().catch(() => {});
    };
  }, []);

  const commit = useCallback((next: RemapState) => {
    setState(next);
    saveRemapState(next);
    syncRemap(next).catch((e) => setNotice(errorText(e)));
  }, []);

  const setAction = useCallback(
    (button: ButtonId, action: RemapAction) => {
      commit({ ...state, actions: { ...state.actions, [button]: action } });
    },
    [state, commit]
  );

  const setEnabled = useCallback(
    (enabled: boolean) => {
      commit({ ...state, enabled });
    },
    [state, commit]
  );

  const startDetect = useCallback(() => {
    setNotice(translate("notice.pressButton"));
    setDetecting(true);
    mouseApi.startDetect().catch((e) => {
      setDetecting(false);
      setNotice(errorText(e));
    });
  }, []);

  const cancelDetect = useCallback(() => {
    setDetecting(false);
    setNotice(null);
    mouseApi.cancelDetect().catch(() => {});
  }, []);

  const select = useCallback((id: ButtonId) => {
    setSelected(id);
  }, []);

  return {
    state,
    selected,
    detecting,
    detected,
    notice,
    select,
    setAction,
    setEnabled,
    startDetect,
    cancelDetect,
  };
}
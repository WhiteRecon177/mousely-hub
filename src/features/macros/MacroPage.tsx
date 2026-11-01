import { useState } from "react";
import { useMacros } from "./hooks/useMacros";
import { ActionList } from "./components/ActionList";
import { SavedMacrosList } from "./components/SavedMacrosList";

export function MacroPage() {
  const macros = useMacros();
  const { status, recorded } = macros;
  const [name, setName] = useState("");
  const [repeat, setRepeat] = useState(1);
  const busy = status !== "idle";

  const handleRecordButton = () => {
    if (status === "recording") void macros.stopRecording();
    else void macros.record();
  };

  const handleSave = () => {
    macros.save(name);
    setName("");
  };

  return (
    <div className="macro-page">
      <p className="warning">
        Recording captures every key you press, including passwords. Press F10 or Stop to end
        recording or playback.
      </p>
      {macros.error && <p className="error">{macros.error}</p>}
      {macros.message && <p className="hint">{macros.message}</p>}

      <div className="macro-row">
        <button onClick={handleRecordButton} disabled={status === "playing"}>
          {status === "recording" ? "Stop" : "Record"}
        </button>
        <button onClick={macros.clearRecorded} disabled={busy || recorded.length === 0}>
          Clear
        </button>
        {status === "recording" && <span className="status">Recording... (F10 to stop)</span>}
      </div>

      <ActionList actions={recorded} onRemove={macros.removeAction} />

      <div className="macro-row">
        <input
          type="text"
          placeholder="Macro name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button onClick={handleSave} disabled={!name.trim() || recorded.length === 0}>
          Save
        </button>
      </div>

      <div className="macro-row">
        <label>
          Repeat{" "}
          <input
            type="number"
            min={1}
            max={1000}
            value={repeat}
            onChange={(e) =>
              setRepeat(Math.min(1000, Math.max(1, parseInt(e.target.value, 10) || 1)))
            }
          />
        </label>
        <button
          onClick={() => void macros.play(recorded, repeat)}
          disabled={busy || recorded.length === 0}
        >
          Play recorded
        </button>
        {status === "playing" && (
          <button onClick={() => void macros.stopPlayback()}>Stop playback</button>
        )}
      </div>

      <SavedMacrosList
        macros={macros.saved}
        disabled={busy}
        onPlay={(macro) => void macros.play(macro.actions, repeat)}
        onDelete={macros.remove}
      />
    </div>
  );
}

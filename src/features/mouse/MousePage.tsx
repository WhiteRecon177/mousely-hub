import { useState } from "react";
import "./mouse.css";
import { useMacros } from "../macros/hooks/useMacros";
import { SavedMacro } from "../macros/types";
import { useMouseRemap } from "./hooks/useMouseRemap";
import { ButtonId, ButtonRow, RemapAction } from "./types";
import { useT, type Key } from "../../i18n";
import { ButtonList } from "./components/ButtonList";
import { MouseDiagram } from "./components/MouseDiagram";
import { MacroNameRow } from "./components/MacroNameRow";
import { ActionPanel } from "./components/ActionPanel";
import { MacroRecorder } from "./components/MacroRecorder";
import { SavedMacrosPanel } from "./components/SavedMacrosPanel";

const ROW_DEFS: { id: ButtonId; key: Key; protectedButton: boolean }[] = [
  { id: "left", key: "btn.left", protectedButton: true },
  { id: "right", key: "btn.right", protectedButton: true },
  { id: "middle", key: "btn.middle", protectedButton: false },
  { id: "back", key: "btn.back", protectedButton: false },
  { id: "forward", key: "btn.forward", protectedButton: false },
];

const NORMAL: RemapAction = { type: "Normal" };

export function MousePage() {
  const t = useT();
  const rows: ButtonRow[] = ROW_DEFS.map((row) => ({
    id: row.id,
    label: t(row.key),
    protectedButton: row.protectedButton,
  }));
  const macros = useMacros();
  const remap = useMouseRemap();

  const [name, setName] = useState("");
  const [repeat, setRepeat] = useState(1);
  const [highlightId, setHighlightId] = useState<string | null>(null);

  const protectedSelected = remap.selected === "left" || remap.selected === "right";
  const busy = macros.status !== "idle";

  const handleSave = () => {
    const id = macros.save(name);
    if (!id) return;
    setName("");
    setHighlightId(id);
    window.setTimeout(() => setHighlightId(null), 3000);
  };

  const handleClear = () => {
    macros.clearRecorded();
    setName("");
  };

  const handleEdit = (macro: SavedMacro) => {
    macros.editMacro(macro);
    setName(macro.name);
  };

  const handleRepeat = (value: number) => {
    setRepeat(Number.isFinite(value) ? Math.min(1000, Math.max(1, value)) : 1);
  };

  return (
    <div className="dd-mouse-root">
      <div className="dd-topbar">
        <span className="dd-hint">{t("remap.emergency")}</span>
        <div className="dd-topbar-right">
          {remap.notice && <span className="dd-message">{remap.notice}</span>}
          <button
            type="button"
            role="switch"
            aria-checked={remap.state.enabled}
            className={remap.state.enabled ? "dd-toggle dd-toggle--on" : "dd-toggle"}
            onClick={() => remap.setEnabled(!remap.state.enabled)}
          >
            <span className="dd-toggle-knob" />
          </button>
          <span>
            {t("remap.label")} {remap.state.enabled ? t("common.on") : t("common.off")}
          </span>
        </div>
      </div>

      <div className="dd-mouse-page">
        <ButtonList
          rows={rows}
          actions={remap.state.actions}
          macros={macros.saved}
          selected={remap.selected}
          detecting={remap.detecting}
          detected={remap.detected}
          onSelect={remap.select}
          onDetect={remap.startDetect}
          onCancelDetect={remap.cancelDetect}
        />

        <div className="dd-center">
          <MouseDiagram
            selected={remap.selected}
            detected={remap.detected}
            enabled={remap.state.enabled}
            onSelect={remap.select}
          />
          <MacroNameRow
            name={name}
            canSave={name.trim() !== "" && macros.recorded.length > 0}
            onNameChange={setName}
            onSave={handleSave}
            onClear={handleClear}
          />
        </div>

        <div className="dd-card dd-right">
          <ActionPanel
            selected={remap.selected}
            protectedButton={protectedSelected}
            action={remap.state.actions[remap.selected] ?? NORMAL}
            macros={macros.saved}
            onChange={(action) => remap.setAction(remap.selected, action)}
          />
          <MacroRecorder
            status={macros.status}
            recorded={macros.recorded}
            repeat={repeat}
            error={macros.error}
            message={macros.message}
            onRepeatChange={handleRepeat}
            onRecord={macros.record}
            onStopRecording={macros.stopRecording}
            onPlay={() => macros.play(macros.recorded, repeat)}
            onStopPlayback={macros.stopPlayback}
            onRemoveAction={macros.removeAction}
          />
          <SavedMacrosPanel
            macros={macros.saved}
            highlightId={highlightId}
            disabled={busy}
            onPlay={(macro) => macros.play(macro.actions, 1)}
            onEdit={handleEdit}
            onDelete={macros.remove}
          />
        </div>
      </div>
    </div>
  );
}
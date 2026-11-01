import { MacroAction, MacroStatus } from "../../macros/types";
import { describeAction } from "../../macros/services/describeAction";
import { CloseIcon, PlayIcon, RecordIcon, StopIcon } from "./Icons";
import { useT } from "../../../i18n";

interface MacroRecorderProps {
  status: MacroStatus;
  recorded: MacroAction[];
  repeat: number;
  error: string | null;
  message: string | null;
  onRepeatChange: (repeat: number) => void;
  onRecord: () => void;
  onStopRecording: () => void;
  onPlay: () => void;
  onStopPlayback: () => void;
  onRemoveAction: (index: number) => void;
}

export function MacroRecorder({
  status,
  recorded,
  repeat,
  error,
  message,
  onRepeatChange,
  onRecord,
  onStopRecording,
  onPlay,
  onStopPlayback,
  onRemoveAction,
}: MacroRecorderProps) {
  const t = useT();
  const recording = status === "recording";
  const playing = status === "playing";

  return (
    <section className="dd-section">
      <div className="dd-title">{t("macro.create")}</div>

      <div className="dd-row">
        <button
          type="button"
          className={recording ? "dd-record dd-record--active" : "dd-record"}
          disabled={playing}
          onClick={recording ? onStopRecording : onRecord}
        >
          <RecordIcon size={16} />
          {recording ? t("common.stop") : t("macro.record")}
        </button>

        {playing ? (
          <button type="button" className="dd-record" onClick={onStopPlayback}>
            <StopIcon size={16} />
            {t("macro.stopPlayback")}
          </button>
        ) : (
          <button
            type="button"
            className="dd-icon-btn"
            title={t("macro.playRecorded")}
            aria-label={t("macro.playRecorded")}
            disabled={recording || recorded.length === 0}
            onClick={onPlay}
          >
            <PlayIcon size={16} />
          </button>
        )}

        <label className="dd-repeat">
          <span className="dd-muted">{t("macro.repeat")}</span>
          <input
            type="number"
            min={1}
            max={1000}
            value={repeat}
            onChange={(e) => onRepeatChange(parseInt(e.target.value, 10))}
          />
        </label>
      </div>

      {recording && <div className="dd-hint">{t("macro.recordingHint")}</div>}

      <div className="dd-actions-list">
        {recorded.length === 0 ? (
          <div className="dd-empty">{t("macro.noActions")}</div>
        ) : (
          recorded.map((action, index) => (
            <div key={index} className="dd-action-row">
              <span>{describeAction(action)}</span>
              <button
                type="button"
                className="dd-mini-btn"
                title={t("common.remove")}
                aria-label={t("macro.removeAction")}
                onClick={() => onRemoveAction(index)}
              >
                <CloseIcon size={12} />
              </button>
            </div>
          ))
        )}
      </div>

      {error && <div className="dd-error">{error}</div>}
      {message && <div className="dd-message">{message}</div>}
    </section>
  );
}
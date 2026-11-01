import type { SavedMacro } from "../types";

type SavedMacrosListProps = {
  macros: SavedMacro[];
  disabled: boolean;
  onPlay: (macro: SavedMacro) => void;
  onDelete: (id: string) => void;
};

export function SavedMacrosList({ macros, disabled, onPlay, onDelete }: SavedMacrosListProps) {
  return (
    <div className="saved-macros">
      <h3>Saved macros</h3>
      {macros.length === 0 && <p className="hint">No saved macros yet</p>}
      {macros.map((macro) => (
        <div key={macro.id} className="saved-row">
          <span>
            {macro.name} ({macro.actions.length} actions)
          </span>
          <button disabled={disabled} onClick={() => onPlay(macro)}>
            Play
          </button>
          <button onClick={() => onDelete(macro.id)}>Delete</button>
        </div>
      ))}
    </div>
  );
}

import type { MacroAction } from "../types";
import { describeAction } from "../services/describeAction";

type ActionListProps = {
  actions: MacroAction[];
  onRemove: (index: number) => void;
};

export function ActionList({ actions, onRemove }: ActionListProps) {
  if (actions.length === 0) {
    return <p className="hint">No actions recorded</p>;
  }

  return (
    <div className="action-list">
      {actions.map((action, index) => (
        <div key={index} className="action-row">
          <span>{describeAction(action)}</span>
          <button onClick={() => onRemove(index)}>x</button>
        </div>
      ))}
    </div>
  );
}

import type { CameraDevice } from "../types";
import { useT } from "../../../i18n";

type CameraSelectorProps = {
  cameras: CameraDevice[];
  selectedId: string | null;
  onSelect: (id: string) => void;
};

export function CameraSelector({ cameras, selectedId, onSelect }: CameraSelectorProps) {
  const t = useT();

  return (
    <select
      className="dd-cam-select"
      value={selectedId ?? ""}
      onChange={(e) => {
        const id = e.target.value;
        if (id) onSelect(id);
      }}
    >
      <option value="">{t("cam.select")}</option>
      {cameras.map((cam) => (
        <option key={cam.deviceId} value={cam.deviceId}>
          {cam.label}
        </option>
      ))}
    </select>
  );
}
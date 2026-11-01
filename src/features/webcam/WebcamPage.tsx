import { useState, useEffect } from "react";
import "./webcam.css";
import { useCameras } from "./hooks/useCameras";
import { useCameraStream } from "./hooks/useCameraStream";
import { useWebcamSettings } from "./hooks/useWebcamSettings";
import { useWebcamProfiles } from "./hooks/useWebcamProfiles";
import { CameraSelector } from "./components/CameraSelector";
import { CameraPreview } from "./components/CameraPreview";
import { CameraSettingsCard } from "./components/CameraSettingsCard";
import { WebcamToolbar } from "./components/WebcamToolbar";
import { ProfilesPanel } from "./components/ProfilesPanel";
import { useT } from "../../i18n";

export function WebcamPage() {
  const t = useT();
  const { cameras, error: cameraError } = useCameras();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const { stream, track, error: streamError } = useCameraStream(selectedId);
  const cameraLabel = cameras.find((c) => c.deviceId === selectedId)?.label ?? "";

  const settings = useWebcamSettings(track, cameraLabel);
  const { profiles, saveProfile, deleteProfile } = useWebcamProfiles();

  // Select the first camera automatically
  useEffect(() => {
    if (cameras.length > 0 && selectedId === null) {
      setSelectedId(cameras[0].deviceId);
    }
  }, [cameras, selectedId]);

  // The selected camera was unplugged: forget it, the first one left is picked above.
  useEffect(() => {
    if (selectedId !== null && !cameras.some((c) => c.deviceId === selectedId)) {
      setSelectedId(null);
    }
  }, [cameras, selectedId]);

  const handleSaveProfile = (name: string) => {
    const snapshot = settings.buildSnapshot();
    if (snapshot) saveProfile(name, snapshot);
  };

  const errorText = cameraError ?? streamError;
  const connected = stream !== null;

  return (
    <div className="dd-cam-page">
      <div className="dd-cam-left">
        <CameraPreview stream={stream} mirrored={settings.mirrored} />
        <div className="dd-cam-caption">
          <span className={connected ? "dd-cam-dot" : "dd-cam-dot dd-cam-dot--off"} />
          <span>{connected ? t("cam.active") : t("cam.inactive")}</span>
          {cameraLabel !== "" && <span className="dd-cam-muted">{cameraLabel}</span>}
        </div>
      </div>

      <div className="dd-cam-right">
        <div className="dd-cam-card">
          <div className="dd-cam-title">{t("common.device")}</div>
          <CameraSelector cameras={cameras} selectedId={selectedId} onSelect={setSelectedId} />
          <WebcamToolbar
            mirrored={settings.mirrored}
            onMirrorChange={settings.setMirrored}
            fps={settings.fps}
            fpsOptions={settings.fpsOptions}
            onFpsChange={settings.changeFps}
            onAuto={settings.runAuto}
          />
        </div>

        <CameraSettingsCard
          controls={settings.controls}
          modes={settings.modes}
          onChange={settings.changeControl}
          onModeChange={settings.changeMode}
        />

        {connected && settings.controls.length === 0 && (
          <div className="dd-cam-muted">{t("cam.noControls")}</div>
        )}

        <ProfilesPanel
          profiles={profiles}
          autosave={settings.autosave}
          onAutosaveChange={settings.changeAutosave}
          onSave={handleSaveProfile}
          onApply={(profile) => void settings.applyProfileSnapshot(profile.snapshot)}
          onDelete={deleteProfile}
        />

        <div className="dd-cam-footer">
          <span className={errorText ? "dd-cam-status dd-cam-status--error" : "dd-cam-status"}>
            {errorText ?? settings.status ?? ""}
          </span>
          <button
            type="button"
            className="dd-cam-btn"
            disabled={settings.controls.length === 0}
            onClick={() => void settings.restoreStart()}
          >
            {t("cam.restoreStart")}
          </button>
        </div>
      </div>
    </div>
  );
}
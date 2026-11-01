import { useState, useEffect } from "react";
import type { CameraDevice } from "../types";
import { translate } from "../../../i18n";

export function useCameras() {
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    let timer: number | undefined;

    const list = async () => {
      try {
        let devices = await navigator.mediaDevices.enumerateDevices();
        let inputs = devices.filter((d) => d.kind === "videoinput");

        // The names stay hidden until the camera was allowed once.
        if (inputs.length > 0 && inputs.some((d) => d.label === "")) {
          const stream = await navigator.mediaDevices.getUserMedia({ video: true });
          stream.getTracks().forEach((track) => track.stop());
          devices = await navigator.mediaDevices.enumerateDevices();
          inputs = devices.filter((d) => d.kind === "videoinput");
        }

        if (!alive) return;

        setCameras(
          inputs.map((d) => ({
            deviceId: d.deviceId,
            label: d.label || "Unknown Camera",
          }))
        );
        setError(inputs.length === 0 ? translate("cam.err.noCamera") : null);
      } catch (err) {
        if (!alive) return;
        setCameras([]);
        setError(translate("cam.err.permission"));
        console.error(err);
      }
    };

    void list();

    // A camera plugged or unplugged while the app is open: look again, no restart needed.
    const onChange = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => void list(), 400);
    };
    navigator.mediaDevices.addEventListener("devicechange", onChange);

    return () => {
      alive = false;
      window.clearTimeout(timer);
      navigator.mediaDevices.removeEventListener("devicechange", onChange);
    };
  }, []);

  return { cameras, error };
}

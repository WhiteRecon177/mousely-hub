import { useState, useEffect } from "react";
import { translate } from "../../../i18n";

export function useCameraStream(deviceId: string | null) {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [track, setTrack] = useState<MediaStreamTrack | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!deviceId) {
      setStream(null);
      setTrack(null);
      return;
    }

    let cancelled = false;
    let currentStream: MediaStream | null = null;

    const startStream = async () => {
      try {
        const s = await navigator.mediaDevices.getUserMedia({
          video: { deviceId: { exact: deviceId } },
        });
        if (cancelled) {
          s.getTracks().forEach((t) => t.stop());
          return;
        }
        currentStream = s;
        setStream(s);
        setTrack(s.getVideoTracks()[0] ?? null);
        setError(null);
      } catch (err) {
        setError(translate("cam.err.access"));
        console.error(err);
      }
    };

    startStream();

    return () => {
      cancelled = true;
      if (currentStream) {
        currentStream.getTracks().forEach((t) => t.stop());
      }
      setStream(null);
      setTrack(null);
    };
  }, [deviceId]);

  return { stream, track, error };
}

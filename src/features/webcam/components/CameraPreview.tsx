import { useEffect, useRef } from "react";
import { WebcamHologram } from "./WebcamHologram";

type CameraPreviewProps = {
  stream: MediaStream | null;
  mirrored?: boolean;
};

export function CameraPreview({ stream, mirrored = false }: CameraPreviewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  return (
    <div className="dd-cam-frame">
      <span className="dd-cam-corner dd-cam-corner--tl" />
      <span className="dd-cam-corner dd-cam-corner--tr" />
      <span className="dd-cam-corner dd-cam-corner--bl" />
      <span className="dd-cam-corner dd-cam-corner--br" />

      <video
        ref={videoRef}
        className={mirrored ? "dd-cam-video dd-cam-video--mirrored" : "dd-cam-video"}
        autoPlay
        muted
        playsInline
      />

      {stream === null && (
        <div className="dd-cam-empty">
          <WebcamHologram />
          <span className="dd-cam-empty-text">No camera signal</span>
        </div>
      )}
      {stream !== null && <span className="dd-cam-live">LIVE</span>}
    </div>
  );
}
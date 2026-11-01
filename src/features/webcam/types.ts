export type CameraDevice = {
  deviceId: string;
  label: string;
};

export type ControlName =
  | "brightness"
  | "contrast"
  | "saturation"
  | "sharpness"
  | "colorTemperature"
  | "exposureTime"
  | "exposureCompensation"
  | "focusDistance"
  | "zoom";

export type ControlRange = {
  name: ControlName;
  min: number;
  max: number;
  step: number;
  value: number;
};

// Camera modes that can be automatic ("continuous") or manual
export type ModeName = "exposureMode" | "focusMode" | "whiteBalanceMode";
export type ModeValue = "continuous" | "manual";

// Everything we save for a camera: values, modes, mirror and fps
export type CameraSnapshot = {
  values: Partial<Record<ControlName, number>>;
  modes: Partial<Record<ModeName, ModeValue>>;
  mirrored: boolean;
  fps: number | null;
};

export type WebcamProfile = {
  id: string;
  name: string;
  snapshot: CameraSnapshot;
};

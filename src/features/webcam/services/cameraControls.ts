import type {
  CameraSnapshot,
  ControlName,
  ControlRange,
  ModeName,
  ModeValue,
} from "../types";

const CONTROL_NAMES: ControlName[] = [
  "brightness",
  "contrast",
  "saturation",
  "sharpness",
  "colorTemperature",
  "exposureTime",
  "exposureCompensation",
  "focusDistance",
  "zoom",
];

const MODE_NAMES: ModeName[] = ["exposureMode", "focusMode", "whiteBalanceMode"];

// Controls that only work when the matching mode is manual
const MANUAL_MODE: Partial<Record<ControlName, ModeName>> = {
  exposureTime: "exposureMode",
  focusDistance: "focusMode",
  colorTemperature: "whiteBalanceMode",
};

// Controls without an automatic mode: "Auto adjust" brings them to a neutral value
const NEUTRAL_CONTROLS: ControlName[] = ["brightness", "contrast", "saturation", "sharpness"];

const FPS_CHOICES = [15, 24, 30, 60];

type NumericRange = { min: number; max: number; step?: number };

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

// True if the capability is a numeric range (min/max)
function isRange(value: unknown): value is NumericRange {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.min === "number" && typeof v.max === "number";
}

function getCaps(track: MediaStreamTrack): Record<string, unknown> {
  if (typeof track.getCapabilities !== "function") return {};
  return track.getCapabilities() as unknown as Record<string, unknown>;
}

function getSettingsRecord(track: MediaStreamTrack): Record<string, unknown> {
  return track.getSettings() as unknown as Record<string, unknown>;
}

function toConstraintSet(obj: Record<string, unknown>): MediaTrackConstraintSet {
  return obj as unknown as MediaTrackConstraintSet;
}

function stepOf(range: NumericRange): number {
  return range.step && range.step > 0 ? range.step : 1;
}

function clamp(value: number, range: NumericRange): number {
  return Math.min(range.max, Math.max(range.min, value));
}

// Middle of the range, aligned to the step
function neutralValue(range: NumericRange): number {
  const step = stepOf(range);
  const middle = (range.min + range.max) / 2;
  return clamp(range.min + Math.round((middle - range.min) / step) * step, range);
}

// ---- Command queue: camera commands run one at a time, never overlapping ----

let queue: Promise<unknown> = Promise.resolve();

function enqueue<T>(task: () => Promise<T>): Promise<T> {
  const run = queue.then(task);
  queue = run.catch(() => undefined);
  return run;
}

// Resolves when every queued camera command has finished
export function whenIdle(): Promise<void> {
  return enqueue(async () => undefined);
}

async function applyNow(track: MediaStreamTrack, set: Record<string, unknown>): Promise<void> {
  await track.applyConstraints({ advanced: [toConstraintSet(set)] });
}

async function safely(label: string, task: () => Promise<void>): Promise<void> {
  try {
    await task();
  } catch (err) {
    console.error(`Camera command failed (${label}):`, err);
  }
}

// ---- Reading ----

export function getSupportedControls(track: MediaStreamTrack): ControlRange[] {
  const capabilities = getCaps(track);
  const settings = getSettingsRecord(track);
  const supported: ControlRange[] = [];

  for (const name of CONTROL_NAMES) {
    const cap = capabilities[name];
    if (!isRange(cap)) continue;
    const current = settings[name];
    supported.push({
      name,
      min: cap.min,
      max: cap.max,
      step: stepOf(cap),
      value: typeof current === "number" ? current : cap.min,
    });
  }

  return supported;
}

export function getModes(track: MediaStreamTrack): Partial<Record<ModeName, ModeValue>> {
  const settings = getSettingsRecord(track);
  const result: Partial<Record<ModeName, ModeValue>> = {};
  for (const mode of MODE_NAMES) {
    const value = settings[mode];
    if (value === "continuous" || value === "manual") result[mode] = value;
  }
  return result;
}

export function getFpsOptions(track: MediaStreamTrack): number[] {
  const range = getCaps(track).frameRate;
  if (!isRange(range)) return [];
  return FPS_CHOICES.filter((f) => f >= range.min && f <= range.max);
}

export function getCurrentFps(track: MediaStreamTrack): number | null {
  const value = track.getSettings().frameRate;
  return typeof value === "number" ? Math.round(value) : null;
}

export function readSnapshot(track: MediaStreamTrack, mirrored: boolean): CameraSnapshot {
  const values: Partial<Record<ControlName, number>> = {};
  for (const control of getSupportedControls(track)) values[control.name] = control.value;
  return { values, modes: getModes(track), mirrored, fps: getCurrentFps(track) };
}

// ---- Writing ----

const pendingValues = new Map<string, number>();

// Applies one control. While dragging a slider only the newest value is sent.
export function applyControl(
  track: MediaStreamTrack,
  name: ControlName,
  value: number
): Promise<void> {
  const key = `${track.id}:${name}`;
  const alreadyQueued = pendingValues.has(key);
  pendingValues.set(key, value);
  if (alreadyQueued) return Promise.resolve();

  return enqueue(async () => {
    const latest = pendingValues.get(key) ?? value;
    pendingValues.delete(key);
    const set: Record<string, unknown> = { [name]: latest };
    const modeKey = MANUAL_MODE[name];
    if (modeKey) set[modeKey] = "manual";
    await applyNow(track, set);
  });
}

export function applyFps(track: MediaStreamTrack, fps: number): Promise<void> {
  return enqueue(() => track.applyConstraints({ frameRate: { ideal: fps, max: fps } }));
}

// The slider that goes with each mode
const MODE_CONTROL: Record<ModeName, ControlName> = {
  exposureMode: "exposureTime",
  focusMode: "focusDistance",
  whiteBalanceMode: "colorTemperature",
};

// Switches one mode between automatic ("continuous") and manual.
// Going manual also sends the current slider value: many cameras ignore "manual" without it.
export function applyMode(track: MediaStreamTrack, mode: ModeName, value: ModeValue): Promise<void> {
  return enqueue(async () => {
    if (value === "manual") {
      const control = getSupportedControls(track).find((c) => c.name === MODE_CONTROL[mode]);
      if (control) {
        try {
          await applyNow(track, { [mode]: value, [control.name]: control.value });
          return;
        } catch (err) {
          console.error(`Manual ${mode} with value failed, trying the mode alone:`, err);
        }
      }
    }
    await applyNow(track, { [mode]: value });
  });
}

// Restarts the automatic modes and brings the other controls to a neutral value.
// Returns the automatic modes that were enabled.
export function autoAdjust(track: MediaStreamTrack): Promise<ModeName[]> {
  return enqueue(async () => {
    const caps = getCaps(track);
    const applied: ModeName[] = [];

    for (const mode of MODE_NAMES) {
      const supported = caps[mode];
      if (!Array.isArray(supported) || !supported.includes("continuous")) continue;
      try {
        // Go through manual first so the camera really restarts its automatic mode
        if (supported.includes("manual")) {
          await applyNow(track, { [mode]: "manual" });
          await wait(150);
        }
        await applyNow(track, { [mode]: "continuous" });
        applied.push(mode);
      } catch (err) {
        console.error("Failed to enable automatic mode:", mode, err);
      }
    }

    for (const name of NEUTRAL_CONTROLS) {
      const cap = caps[name];
      if (!isRange(cap)) continue;
      await safely(name, () => applyNow(track, { [name]: neutralValue(cap) }));
    }

    return applied;
  });
}

// Applies a saved snapshot: fps first, then modes, then values
export function applySnapshot(track: MediaStreamTrack, snapshot: CameraSnapshot): Promise<void> {
  return enqueue(async () => {
    const caps = getCaps(track);

    if (snapshot.fps) {
      const fps = snapshot.fps;
      await safely("fps", () =>
        track.applyConstraints({ frameRate: { ideal: fps, max: fps } })
      );
    }

    for (const mode of MODE_NAMES) {
      const wanted = snapshot.modes[mode];
      const supported = caps[mode];
      if (!wanted || !Array.isArray(supported) || !supported.includes(wanted)) continue;
      await safely(mode, () => applyNow(track, { [mode]: wanted }));
    }

    for (const name of CONTROL_NAMES) {
      const value = snapshot.values[name];
      const cap = caps[name];
      if (value === undefined || !isRange(cap)) continue;
      const modeKey = MANUAL_MODE[name];
      // A value tied to a mode is only applied when that mode is not automatic
      if (modeKey && snapshot.modes[modeKey] === "continuous") continue;
      await safely(name, () => applyNow(track, { [name]: clamp(value, cap) }));
    }
  });
}
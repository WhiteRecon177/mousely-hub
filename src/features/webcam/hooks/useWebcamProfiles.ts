import { useState, useCallback } from "react";
import type { CameraSnapshot, WebcamProfile } from "../types";
import { loadProfiles, saveProfiles } from "../services/profileStorage";

export function useWebcamProfiles() {
  const [profiles, setProfiles] = useState<WebcamProfile[]>(() => loadProfiles());

  // Saves a profile; a profile with the same name is overwritten
  const saveProfile = useCallback(
    (name: string, snapshot: CameraSnapshot) => {
      const trimmed = name.trim();
      if (!trimmed) return;
      const existing = profiles.find((p) => p.name.toLowerCase() === trimmed.toLowerCase());
      const next = existing
        ? profiles.map((p) => (p.id === existing.id ? { ...p, snapshot } : p))
        : [...profiles, { id: String(Date.now()), name: trimmed, snapshot }];
      setProfiles(next);
      saveProfiles(next);
    },
    [profiles]
  );

  const deleteProfile = useCallback(
    (id: string) => {
      const next = profiles.filter((p) => p.id !== id);
      setProfiles(next);
      saveProfiles(next);
    },
    [profiles]
  );

  return { profiles, saveProfile, deleteProfile };
}

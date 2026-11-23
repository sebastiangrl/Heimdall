import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { ProfileState } from "@/types/profile";

export function useActivityProfile(pollMs = 2000) {
  const [profile, setProfile] = useState<ProfileState | null>(null);

  const refresh = useCallback(async () => {
    try {
      const p = await invoke<ProfileState>("get_activity_profile");
      setProfile(p);
    } catch {
      /* sin Tauri */
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = setInterval(() => void refresh(), pollMs);
    return () => clearInterval(id);
  }, [pollMs, refresh]);

  return { profile, refresh };
}
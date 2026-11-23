import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export type MonitorMode = "daemon" | "embedded" | "unknown";

export interface MonitorStatus {
  mode: MonitorMode;
  daemonConnected: boolean;
  embeddedActive: boolean;
  samplingIntervalMs: number;
}

export function useMonitorStatus(pollMs = 5000) {
  const [status, setStatus] = useState<MonitorStatus | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [daemon, monitoring] = await Promise.all([
        invoke<{ running: boolean; embedded: boolean }>("get_daemon_status"),
        invoke<{ mode: string; daemonConnected: boolean; samplingIntervalMs: number }>(
          "get_monitoring_status",
        ),
      ]);
      const mode: MonitorMode =
        daemon.running || monitoring.daemonConnected
          ? "daemon"
          : daemon.embedded
            ? "embedded"
            : "unknown";
      setStatus({
        mode,
        daemonConnected: daemon.running || monitoring.daemonConnected,
        embeddedActive: daemon.embedded || monitoring.mode === "embedded",
        samplingIntervalMs: monitoring.samplingIntervalMs,
      });
    } catch {
      setStatus(null);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), pollMs);
    return () => window.clearInterval(id);
  }, [pollMs, refresh]);

  return { status, refresh };
}
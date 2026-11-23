import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Diagnosis } from "@/types/diagnosis";
import type { HardwareSummary, TelemetrySnapshot } from "@/types/telemetry";

const isTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function useTelemetry(pollMs = 1000) {
  const [snapshot, setSnapshot] = useState<TelemetrySnapshot | null>(null);
  const [hardware, setHardware] = useState<HardwareSummary | null>(null);
  const [diagnoses, setDiagnoses] = useState<Diagnosis[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [diagnosticsRunning, setDiagnosticsRunning] = useState(false);
  const [lastDiagnosticMessage, setLastDiagnosticMessage] = useState<string | null>(null);
  const visibleRef = useRef(true);

  const refresh = useCallback(async () => {
    if (!visibleRef.current) return;
    if (!isTauri()) {
      setError("Ejecuta `npm run tauri dev` para telemetría en vivo.");
      return;
    }
    try {
      const [snap, hw, dx] = await Promise.all([
        invoke<TelemetrySnapshot>("get_telemetry_snapshot"),
        invoke<HardwareSummary>("get_hardware_summary"),
        invoke<Diagnosis[]>("get_active_diagnoses"),
      ]);
      setSnapshot(snap);
      setHardware(hw);
      setDiagnoses(dx);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const runDiagnostics = useCallback(async () => {
    if (!isTauri()) {
      setError("Ejecuta `npm run tauri dev` para telemetría en vivo.");
      return;
    }
    setDiagnosticsRunning(true);
    setLastDiagnosticMessage(null);
    try {
      const dx = await invoke<Diagnosis[]>("run_diagnostics");
      setDiagnoses(dx);
      const active = dx.filter((d) => d.lifecycle_state !== "resolved");
      setLastDiagnosticMessage(
        active.length === 0
          ? "diagnosticsCompleteNone"
          : "diagnosticsCompleteSome",
      );
      await refresh();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDiagnosticsRunning(false);
    }
  }, [refresh]);

  useEffect(() => {
    const onVis = () => {
      visibleRef.current = document.visibilityState === "visible";
      if (visibleRef.current) void refresh();
    };
    document.addEventListener("visibilitychange", onVis);
    void refresh();
    const id = setInterval(() => void refresh(), pollMs);
    return () => {
      clearInterval(id);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, [pollMs, refresh]);

  return {
    snapshot,
    hardware,
    diagnoses,
    error,
    refresh,
    runDiagnostics,
    diagnosticsRunning,
    lastDiagnosticMessage,
  };
}
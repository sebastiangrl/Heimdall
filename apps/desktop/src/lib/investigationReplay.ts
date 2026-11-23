import type { InvestigationDossier, TelemetryFrameLite } from "@/types/investigation";

export type ProcessCpuState =
  | "available"
  | "unavailable"
  | "not_tracked"
  | "exited";

export interface ReplayPoint {
  at: string;
  t: number;
  globalCpu: number | null;
  processCoreCpu: number | null;
  processMachineCpu: number | null;
  processState: ProcessCpuState;
  psiCpu: number | null;
  isBurstHit: boolean;
  afterExit: boolean;
}

export function buildReplayTimeline(dossier: InvestigationDossier): ReplayPoint[] {
  const frames: TelemetryFrameLite[] = [...dossier.context_frames, ...dossier.post_frames];
  const samples = [...dossier.process_series].sort(
    (a, b) => Date.parse(a.observed_at) - Date.parse(b.observed_at),
  );
  const burst = new Set(dossier.attribution.burst_hit_times);
  const exitAt = dossier.attribution.exit?.observed_absent_at
    ? Date.parse(dossier.attribution.exit.observed_absent_at)
    : null;

  const stamps = new Set<number>();
  for (const f of frames) stamps.add(Date.parse(f.observed_at));
  for (const s of samples) stamps.add(Date.parse(s.observed_at));
  const sorted = [...stamps].sort((a, b) => a - b);

  return sorted.map((t) => {
    const at = new Date(t).toISOString();
    const frame = nearestFrame(frames, t);
    const sample = samples.find((s) => Date.parse(s.observed_at) === t);
    let processState: ProcessCpuState = "not_tracked";
    let processCoreCpu: number | null = null;
    let processMachineCpu: number | null = null;
    if (exitAt !== null && t > exitAt) {
      processState = "exited";
    } else if (sample) {
      if (sample.availability === "unavailable" || sample.cpu_core_percent == null) {
        processState = "unavailable";
      } else {
        processState = "available";
        processCoreCpu = sample.cpu_core_percent;
        processMachineCpu = sample.cpu_machine_percent;
      }
    }
    return {
      at,
      t,
      globalCpu: frame?.cpu_percent ?? null,
      processCoreCpu,
      processMachineCpu,
      processState,
      psiCpu: frame?.psi_cpu ?? null,
      isBurstHit: burst.has(at) || samples.some((s) => s.observed_at === at && burst.has(s.observed_at)),
      afterExit: exitAt !== null && t >= exitAt,
    };
  });
}

function nearestFrame(frames: TelemetryFrameLite[], t: number): TelemetryFrameLite | undefined {
  let best: TelemetryFrameLite | undefined;
  let bestDist = Number.POSITIVE_INFINITY;
  for (const f of frames) {
    const ft = Date.parse(f.observed_at);
    const d = Math.abs(ft - t);
    if (d < bestDist) {
      bestDist = d;
      best = f;
    }
  }
  return best;
}
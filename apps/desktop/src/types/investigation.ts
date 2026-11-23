/** Alineado con `heimdall_investigation` (serde snake_case). */

export type InvestigationLevel = "event" | "anomaly" | "incident";
export type InvestigationState = "open" | "closed" | "incomplete";
export type AttributionConfidence = "high" | "medium" | "insufficient";
export type ImpactSeverity = "none" | "minor" | "major";
export type SampleAvailability = "available" | "unavailable";

export interface ProcessIdentity {
  boot_id: string;
  pid: number;
  start_time_ticks: number;
  name: string;
}

export interface InvestigationSummary {
  id: string;
  created_at: string;
  updated_at: string;
  state: InvestigationState;
  level: InvestigationLevel;
  identity: ProcessIdentity;
  attribution_confidence: AttributionConfidence;
  impact_severity: ImpactSeverity;
  incomplete: boolean;
}

export interface HistoricalProcessSample {
  boot_id: string;
  pid: number;
  start_time_ticks: number;
  name: string;
  observed_at: string;
  cpu_core_percent: number | null;
  cpu_machine_percent: number | null;
  memory_bytes: number | null;
  availability: SampleAvailability;
}

export interface ProcessExitEvent {
  boot_id: string;
  pid: number;
  start_time_ticks: number;
  name: string;
  last_seen_at: string;
  observed_absent_at: string;
  exit_observed_between_samples: boolean;
}

export interface TelemetryFrameLite {
  observed_at: string;
  cpu_percent: number | null;
  psi_cpu?: number | null;
  psi_mem?: number | null;
  psi_io?: number | null;
}

export interface AttributionEvidence {
  identity: ProcessIdentity;
  activity_interval: { start: string; end: string };
  cpu_core_max: number | null;
  cpu_core_mean: number | null;
  cpu_machine_mean: number | null;
  burst_hit_times: string[];
  baseline_global_cpu: number | null;
  activity_global_cpu: number | null;
  global_cpu_delta: number | null;
  process_contribution_estimate: number | null;
  temporal_overlap: boolean;
  alternative_processes: {
    identity: ProcessIdentity;
    cpu_machine_mean: number | null;
    share_of_process_contribution: number | null;
  }[];
  sample_coverage: number;
  samples_available: number;
  samples_total: number;
  exit: ProcessExitEvent | null;
  confidence: AttributionConfidence;
  confidence_rationale: string[];
}

export interface ImpactAssessment {
  severity: ImpactSeverity;
  psi_cpu_some_avg10: number | null;
  psi_mem_some_avg10: number | null;
  psi_io_some_avg10: number | null;
  global_cpu_peak: number | null;
  global_cpu_baseline: number | null;
  global_cpu_delta: number | null;
  recovery_observed: boolean;
  activity_profile: string;
  degradation_confirmed: boolean;
  notes: string[];
}

export interface InvestigationDossier {
  schema_version: number;
  id: string;
  created_at: string;
  updated_at: string;
  state: InvestigationState;
  level: InvestigationLevel;
  rule_id: string;
  identity: ProcessIdentity;
  attribution: AttributionEvidence;
  impact: ImpactAssessment;
  explanation: string;
  process_series: HistoricalProcessSample[];
  context_frames: TelemetryFrameLite[];
  post_frames: TelemetryFrameLite[];
  limitations: string[];
}
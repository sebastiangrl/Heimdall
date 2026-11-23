export type DiagnosisSeverity = "info" | "warning" | "critical";
export type Confidence = "low" | "medium" | "high";
export type DiagnosisLifecycleState = "detected" | "persistent" | "resolved" | "dismissed";

export interface Evidence {
  kind: string;
  reference: string;
  summary: string;
}

export interface RelatedProcess {
  pid: number;
  name: string;
  cpu_percent?: number;
  memory_bytes?: number;
}

export interface RecommendedAction {
  kind: string;
  label: string;
  detail?: string;
}

export interface Diagnosis {
  id: string;
  rule_id: string;
  title: string;
  explanation: string;
  severity: DiagnosisSeverity;
  observed_at: string;
  first_detected_at: string;
  last_seen_at: string;
  lifecycle_state: DiagnosisLifecycleState;
  active_seconds: number;
  evidence: Evidence[];
  affected_resources: string[];
  related_processes: RelatedProcess[];
  confidence: Confidence;
  recommended_actions: RecommendedAction[];
  limitations?: string[];
}
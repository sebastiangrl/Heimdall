export type ActivityProfileKey =
  | "gaming"
  | "content_creation"
  | "development"
  | "general"
  | "idle";

export interface ProfileSignal {
  kind: string;
  detail: string;
  weight: number;
}

export interface ActivityAssessment {
  profile: ActivityProfileKey;
  confidence: number;
  signals: ProfileSignal[];
  dominant_apps: string[];
}

export interface ProfileBaselineSummary {
  profile: ActivityProfileKey;
  sampleCount: number;
  learning: boolean;
  memUsedRatioTypicalMin: number;
  memUsedRatioTypicalMax: number;
  cpuPercentTypical: number;
  gpuPercentTypical?: number | null;
  observation?: string | null;
}

export interface ProfileState {
  profile: ActivityProfileKey;
  confidence: number;
  assessment: ActivityAssessment;
  sessionStartedAt: string;
  profileSince: string;
  learning: boolean;
  sessionSeconds: number;
  baseline: ProfileBaselineSummary;
}

export interface ProcessProfileOverride {
  pattern: string;
  profile: ActivityProfileKey;
}
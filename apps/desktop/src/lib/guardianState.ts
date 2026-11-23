import type { TFunction } from "i18next";
import type { Diagnosis } from "@/types/diagnosis";
import type { TelemetrySnapshot } from "@/types/telemetry";

export type GuardianTone = "healthy" | "warning" | "critical" | "limited";

export interface GuardianState {
  tone: GuardianTone;
  headline: string;
  subline: string;
  lastAnalysis: string | null;
  sensorGaps: string[];
}

export function buildGuardianState(
  t: TFunction,
  hostname: string | undefined,
  diagnoses: Diagnosis[],
  snapshot: TelemetrySnapshot | null,
  locale: string,
): GuardianState {
  const sensorGaps: string[] = [];
  if (snapshot?.gpu.utilization_percent.status !== "supported") {
    sensorGaps.push(t("guardian.gap.gpu"));
  }
  if (snapshot?.cpu.temperature_c.status !== "supported") {
    sensorGaps.push(t("guardian.gap.cpuTemp"));
  }

  const lastAnalysis = snapshot?.observed_at
    ? new Date(snapshot.observed_at).toLocaleString(locale, {
        dateStyle: "short",
        timeStyle: "medium",
      })
    : null;

  const host = hostname ?? t("guardian.thisMachine");

  if (!snapshot) {
    return {
      tone: "limited",
      headline: t("guardian.limited.headline"),
      subline: t("guardian.limited.subline"),
      lastAnalysis,
      sensorGaps,
    };
  }

  const critical = diagnoses.filter((d) => d.severity === "critical");
  const warning = diagnoses.filter((d) => d.severity === "warning");

  if (critical.length > 0) {
    const primary = critical[0];
    return {
      tone: "critical",
      headline: t(`diagnosis.rules.${primary.rule_id}.guardianHeadline`, {
        defaultValue: t("guardian.issue.headline"),
      }),
      subline: t(`diagnosis.rules.${primary.rule_id}.guardianSubline`, {
        defaultValue: t("guardian.issue.subline"),
        host,
      }),
      lastAnalysis,
      sensorGaps,
    };
  }

  if (warning.length > 0) {
    const primary = warning[0];
    return {
      tone: "warning",
      headline: t(`diagnosis.rules.${primary.rule_id}.guardianHeadline`, {
        defaultValue: t("guardian.issue.headline"),
      }),
      subline: t(`diagnosis.rules.${primary.rule_id}.guardianSubline`, {
        defaultValue: t("guardian.issue.subline"),
        host,
      }),
      lastAnalysis,
      sensorGaps,
    };
  }

  if (sensorGaps.length > 0) {
    return {
      tone: "limited",
      headline: t("guardian.partial.headline", { host }),
      subline: t("guardian.partial.subline"),
      lastAnalysis,
      sensorGaps,
    };
  }

  return {
    tone: "healthy",
    headline: t("guardian.ok.headline", { host }),
    subline: t("guardian.ok.subline"),
    lastAnalysis,
    sensorGaps,
  };
}
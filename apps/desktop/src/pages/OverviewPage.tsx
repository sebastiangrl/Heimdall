import { useMemo, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { DiagnosisCard } from "@/components/DiagnosisCard";
import { ActivityChart } from "@/components/overview/ActivityChart";
import { CompactMetricGrid } from "@/components/overview/CompactMetricGrid";
import { GuardianHero } from "@/components/overview/GuardianHero";
import { buildGuardianState } from "@/lib/guardianState";
import { useActivityProfile } from "@/hooks/useActivityProfile";
import { useTelemetry } from "@/hooks/useTelemetry";

export function OverviewPage() {
  const { t, i18n } = useTranslation();
  const {
    snapshot,
    hardware,
    diagnoses,
    error,
    runDiagnostics,
    diagnosticsRunning,
    lastDiagnosticMessage,
  } = useTelemetry(1000);
  const { profile } = useActivityProfile(3000);
  const [windowHidden, setWindowHidden] = useState(document.visibilityState !== "visible");

  useEffect(() => {
    const fn = () => setWindowHidden(document.visibilityState !== "visible");
    document.addEventListener("visibilitychange", fn);
    return () => document.removeEventListener("visibilitychange", fn);
  }, []);

  const guardian = useMemo(
    () =>
      buildGuardianState(
        t,
        hardware?.hostname,
        diagnoses,
        snapshot,
        i18n.language,
      ),
    [t, hardware?.hostname, diagnoses, snapshot, i18n.language],
  );

  const memPercent = useMemo(() => {
    if (!snapshot) return null;
    const used = snapshot.memory.used_bytes.value;
    const total = snapshot.memory.total_bytes.value;
    if (used === undefined || total === undefined || total === 0) return null;
    return (used / total) * 100;
  }, [snapshot]);

  const cpuVal = snapshot?.cpu.usage_percent_global.value ?? null;

  return (
    <div className="flex flex-col gap-5">
      <GuardianHero
        state={guardian}
        onRunDiagnostics={() => void runDiagnostics()}
        diagnosticsRunning={diagnosticsRunning}
        diagnosticFeedback={
          lastDiagnosticMessage ? t(`overview.${lastDiagnosticMessage}`) : null
        }
      />

      {error ? (
        <div className="glass-panel border border-[var(--color-critical)]/40 p-4 text-sm text-[var(--color-critical)]">
          {error}
        </div>
      ) : null}

      <ActivityChart cpu={cpuVal} memoryPercent={memPercent} hidden={windowHidden} />

      <section>
        <h3 className="font-display mb-3 text-lg">{t("overview.metricsSupport")}</h3>
        <CompactMetricGrid snapshot={snapshot} />
      </section>

      <section>
        <h3 className="font-display mb-3 text-lg text-[var(--color-text-primary)]">
          {t("overview.activeFindings")}
        </h3>
        {diagnoses.length === 0 ? (
          <p className="glass-panel p-4 text-sm text-[var(--color-text-secondary)]">
            {t("overview.noneActive")}
          </p>
        ) : (
          <div className="grid gap-3 lg:grid-cols-2">
            {diagnoses.map((d) => (
              <DiagnosisCard key={d.id} diagnosis={d} variant="compact" />
            ))}
          </div>
        )}
      </section>

      {profile ? (
        <p className="glass-panel p-3 text-sm text-[var(--color-text-secondary)]">
          {t("overview.activityProfile", {
            name: t(`profiles.kinds.${profile.profile}`),
          })}
        </p>
      ) : null}
    </div>
  );
}
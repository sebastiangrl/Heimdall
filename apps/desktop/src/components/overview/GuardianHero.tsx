import { useTranslation } from "react-i18next";
import type { GuardianState } from "@/lib/guardianState";
import { Shield } from "lucide-react";

export function GuardianHero({
  state,
  onRunDiagnostics,
  diagnosticsRunning,
  diagnosticFeedback,
}: {
  state: GuardianState;
  onRunDiagnostics: () => void;
  diagnosticsRunning?: boolean;
  diagnosticFeedback?: string | null;
}) {
  const { t } = useTranslation();

  return (
    <section className="glass-panel-strong relative overflow-hidden p-4 md:p-5">
      <div
        className="pointer-events-none absolute inset-0 opacity-80"
        style={{
          background: `linear-gradient(135deg, color-mix(in srgb, var(--color-violet) 18%, transparent), color-mix(in srgb, var(--color-pink) 12%, transparent))`,
        }}
        aria-hidden
      />
      <div className="relative grid gap-4 md:grid-cols-[1.2fr_0.8fr] md:items-center">
        <div>
          <div className="mb-3 flex items-center gap-2 text-xs uppercase tracking-widest text-[var(--color-lavender)]">
            <Shield size={14} />
            {t("guardian.badge")}
          </div>
          <h2 className="font-display text-xl font-semibold md:text-2xl">
            {state.headline}
          </h2>
          <p className="mt-2 max-w-xl text-sm leading-relaxed text-[var(--color-text-secondary)]">
            {state.subline}
          </p>
          {state.sensorGaps.length > 0 ? (
            <p className="mt-3 text-xs text-[var(--color-text-secondary)]">
              {t("guardian.sensorsNote")}: {state.sensorGaps.join(" · ")}
            </p>
          ) : null}
          {state.lastAnalysis ? (
            <p className="mt-4 font-mono text-xs text-[var(--color-text-secondary)]">
              {t("guardian.lastAnalysis")}: {state.lastAnalysis}
            </p>
          ) : null}
          <div className="mt-4 flex flex-col gap-2">
            <button
              type="button"
              disabled={diagnosticsRunning}
              onClick={onRunDiagnostics}
              className="rounded-2xl border border-[var(--color-pink)]/35 bg-[var(--color-violet)]/20 px-5 py-2.5 text-sm text-[var(--color-lavender)] transition hover:bg-[var(--color-violet)]/30 disabled:opacity-50"
            >
              {diagnosticsRunning ? t("overview.runningDiagnostics") : t("overview.runDiagnostics")}
            </button>
            {diagnosticFeedback ? (
              <p className="text-xs text-[var(--color-text-secondary)]" role="status">
                {diagnosticFeedback}
              </p>
            ) : null}
          </div>
        </div>
        <div className="flex items-center justify-center">
          <div className="relative h-32 w-32 md:h-40 md:w-40">
            <div className="absolute inset-0 rounded-full bg-[radial-gradient(circle,var(--color-pink)_0%,transparent_65%)] opacity-40 blur-2xl" />
            <svg viewBox="0 0 200 200" className="h-full w-full">
              <circle cx="100" cy="100" r="78" fill="none" stroke="#7C3AED" strokeWidth="1" opacity="0.35" />
              <circle cx="100" cy="100" r="54" fill="none" stroke="#EC4899" strokeWidth="1.5" opacity="0.5" />
              <circle cx="100" cy="100" r="28" fill="url(#g-core)" />
              <defs>
                <radialGradient id="g-core">
                  <stop offset="0%" stopColor="#F5F0FA" />
                  <stop offset="40%" stopColor="#EC4899" />
                  <stop offset="100%" stopColor="#7C3AED" />
                </radialGradient>
              </defs>
            </svg>
          </div>
        </div>
      </div>
    </section>
  );
}
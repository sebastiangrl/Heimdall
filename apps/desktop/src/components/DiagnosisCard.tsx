import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { AlertTriangle, Info } from "lucide-react";
import type { Diagnosis } from "@/types/diagnosis";
import { formatBytes } from "@/lib/format";
import { diagnosisExplanation, diagnosisTitle, severityLabel } from "@/lib/diagnosisText";

const severityTone: Record<Diagnosis["severity"], string> = {
  info: "border-[var(--color-violet)]/25",
  warning: "border-[var(--color-warning)]/35",
  critical: "border-[var(--color-critical)]/40",
};

export function DiagnosisCard({
  diagnosis,
  variant = "full",
  onDismiss,
}: {
  diagnosis: Diagnosis;
  variant?: "full" | "compact";
  onDismiss?: () => void;
}) {
  const { t, i18n } = useTranslation();
  const Icon = diagnosis.severity === "info" ? Info : AlertTriangle;
  const title = diagnosisTitle(diagnosis, t);
  const explanation = diagnosisExplanation(diagnosis, t);

  return (
    <article className={`glass-panel border p-4 md:p-5 ${severityTone[diagnosis.severity]}`}>
      <div className="flex items-start gap-3">
        <Icon className="mt-0.5 shrink-0 text-[var(--color-lavender)]" size={20} />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="font-display text-base text-[var(--color-text-primary)] md:text-lg">
              {title}
            </h3>
            <span className="rounded-lg bg-white/5 px-2 py-0.5 text-[10px] uppercase tracking-wide text-[var(--color-text-secondary)]">
              {severityLabel(diagnosis.severity, t)} ·{" "}
              {t(`lifecycle.${diagnosis.lifecycle_state}`)}
            </span>
            {variant === "full" && diagnosis.lifecycle_state !== "dismissed" ? (
              <button
                type="button"
                className="rounded-lg border border-white/10 px-2 py-0.5 text-[10px] uppercase text-[var(--color-text-secondary)] hover:bg-white/5"
                onClick={() =>
                  void invoke("dismiss_diagnosis", { ruleId: diagnosis.rule_id }).then(() =>
                    onDismiss?.(),
                  )
                }
              >
                {t("diagnosis.dismiss")}
              </button>
            ) : null}
          </div>
          <p className="mt-2 text-sm leading-relaxed text-[var(--color-text-secondary)]">
            {explanation}
          </p>

          {variant === "full" ? (
            <div className="mt-4 space-y-2">
              <p className="text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
                {t("diagnosis.facts")}
              </p>
              <ul className="space-y-1">
                {diagnosis.evidence.map((ev) => (
                  <li
                    key={`${ev.reference}-${ev.summary}`}
                    className="font-mono text-xs text-[var(--color-text-secondary)]"
                  >
                    {ev.reference}: {ev.summary}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          {diagnosis.related_processes.length > 0 && variant === "full" ? (
            <ul className="mt-3 text-sm text-[var(--color-text-secondary)]">
              {diagnosis.related_processes.map((p) => (
                <li key={p.pid}>
                  {p.name}{" "}
                  <span className="font-mono text-xs opacity-70">
                    pid {p.pid}
                    {p.cpu_percent != null ? ` · ${p.cpu_percent.toFixed(1)}% CPU` : ""}
                    {p.memory_bytes != null
                      ? ` · ${formatBytes(p.memory_bytes, i18n.language)}`
                      : ""}
                  </span>
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      </div>
    </article>
  );
}
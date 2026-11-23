import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Download, MessageSquare } from "lucide-react";
import type { InvestigationDossier } from "@/types/investigation";
import type { AssistantReply } from "@/types/intelligence";
import { IncidentReplay } from "@/components/investigations/IncidentReplay";

function downloadText(filename: string, text: string) {
  const blob = new Blob([text], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export function InvestigationDossierPanel({ dossier }: { dossier: InvestigationDossier }) {
  const { t, i18n } = useTranslation();
  const [aiText, setAiText] = useState<string | null>(null);
  const [aiLoading, setAiLoading] = useState(false);
  const [aiError, setAiError] = useState<string | null>(null);

  const exportJson = async () => {
    const json = await invoke<string>("export_investigation", { investigationId: dossier.id });
    downloadText(`heimdall-investigation-${dossier.id}.json`, json);
  };

  const askExplain = async () => {
    setAiLoading(true);
    setAiError(null);
    try {
      const res = await invoke<AssistantReply>("explain_investigation", {
        investigationId: dossier.id,
        question: null,
        model: null,
      });
      if (res.blockedReason && !res.text) {
        setAiError(res.blockedReason);
        setAiText(null);
      } else {
        setAiText(res.text || res.blockedReason || t("investigations.aiEmpty"));
      }
    } catch (e) {
      setAiError(String(e));
    } finally {
      setAiLoading(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <header className="glass-panel p-4">
        <p className="text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
          {t("investigations.deterministicConclusion")}
        </p>
        <p className="mt-2 text-sm leading-relaxed text-[var(--color-text-primary)]">{dossier.explanation}</p>
        <div className="mt-4 flex flex-wrap gap-2 text-xs">
          <Badge label={t("investigations.level")} value={t(`investigations.levels.${dossier.level}`)} />
          <Badge
            label={t("investigations.attribution")}
            value={t(`investigations.confidence.${dossier.attribution.confidence}`)}
          />
          <Badge
            label={t("investigations.impact")}
            value={t(`investigations.impactLevels.${dossier.impact.severity}`)}
          />
          <Badge label={t("investigations.state")} value={t(`investigations.states.${dossier.state}`)} />
        </div>
        <div className="mt-4 flex flex-wrap gap-2">
          <button type="button" className="btn-secondary flex items-center gap-2" onClick={() => void exportJson()}>
            <Download size={16} />
            {t("investigations.export")}
          </button>
        </div>
      </header>

      <IncidentReplay dossier={dossier} />

      <section className="glass-panel p-4">
        <h3 className="font-display text-base">{t("investigations.identity")}</h3>
        <dl className="mt-3 grid gap-2 text-sm md:grid-cols-2">
          <Field label={t("investigations.processName")} value={dossier.identity.name || "—"} />
          <Field label="PID" value={String(dossier.identity.pid)} />
          <Field label={t("investigations.startTicks")} value={String(dossier.identity.start_time_ticks)} />
          <Field label="boot_id" value={dossier.identity.boot_id} mono />
        </dl>
      </section>

      <section className="glass-panel space-y-3 p-4">
        <h3 className="font-display text-base">{t("investigations.evidence")}</h3>
        <ul className="list-disc space-y-1 pl-5 text-sm text-[var(--color-text-secondary)]">
          {dossier.attribution.confidence_rationale.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
        <p className="text-sm">
          {t("investigations.coverage", {
            avail: dossier.attribution.samples_available,
            total: dossier.attribution.samples_total,
            pct: Math.round(dossier.attribution.sample_coverage * 100),
          })}
        </p>
        {dossier.attribution.exit ? (
          <p className="text-sm text-[var(--color-amber)]">
            {t("investigations.exitBetweenSamples", {
              last: new Date(dossier.attribution.exit.last_seen_at).toLocaleString(i18n.language),
              absent: new Date(dossier.attribution.exit.observed_absent_at).toLocaleString(i18n.language),
            })}
          </p>
        ) : null}
      </section>

      {dossier.attribution.alternative_processes.length > 0 ? (
        <section className="glass-panel p-4">
          <h3 className="font-display text-base">{t("investigations.alternatives")}</h3>
          <ul className="mt-2 space-y-2 text-sm">
            {dossier.attribution.alternative_processes.map((alt) => (
              <li key={`${alt.identity.pid}-${alt.identity.start_time_ticks}`} className="border-t border-white/5 pt-2">
                {alt.identity.name} (pid {alt.identity.pid})
                {alt.cpu_machine_mean != null ? ` · ${alt.cpu_machine_mean.toFixed(2)} % máquina` : ""}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <section className="glass-panel space-y-3 p-4">
        <h3 className="font-display text-base">{t("investigations.impactSection")}</h3>
        <ul className="list-disc space-y-1 pl-5 text-sm text-[var(--color-text-secondary)]">
          {dossier.impact.notes.map((n) => (
            <li key={n}>{n}</li>
          ))}
        </ul>
      </section>

      <section className="glass-panel space-y-3 p-4">
        <div className="flex items-center justify-between gap-2">
          <h3 className="font-display text-base">{t("investigations.aiExplain")}</h3>
          <button type="button" className="btn-ghost flex items-center gap-1" disabled={aiLoading} onClick={() => void askExplain()}>
            <MessageSquare size={16} />
            {aiLoading ? t("investigations.aiLoading") : t("investigations.aiAsk")}
          </button>
        </div>
        <p className="text-xs text-[var(--color-text-secondary)]">{t("investigations.aiDisclaimer")}</p>
        {aiError ? <p className="text-sm text-[var(--color-danger)]">{aiError}</p> : null}
        {aiText ? (
          <p className="rounded-xl border border-white/5 bg-white/5 p-3 text-sm leading-relaxed">{aiText}</p>
        ) : null}
      </section>
    </div>
  );
}

function Badge({ label, value }: { label: string; value: string }) {
  return (
    <span className="rounded-full border border-white/10 bg-white/5 px-2.5 py-1">
      <span className="text-[var(--color-text-secondary)]">{label}: </span>
      <span className="text-[var(--color-lavender)]">{value}</span>
    </span>
  );
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <dt className="text-[var(--color-text-secondary)]">{label}</dt>
      <dd className={mono ? "font-mono text-xs break-all" : ""}>{value}</dd>
    </div>
  );
}
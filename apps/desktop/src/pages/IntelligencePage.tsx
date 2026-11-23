import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { DiagnosisCard } from "@/components/DiagnosisCard";
import { useTelemetry } from "@/hooks/useTelemetry";
import type { AssistantQueueStatus, AssistantReply, OllamaStatus } from "@/types/intelligence";

export function IntelligencePage() {
  const { t } = useTranslation();
  const {
    diagnoses,
    runDiagnostics,
    refresh,
    error,
    diagnosticsRunning,
    lastDiagnosticMessage,
  } = useTelemetry(1000);
  const [ollama, setOllama] = useState<OllamaStatus | null>(null);
  const [model, setModel] = useState("");
  const [question, setQuestion] = useState("");
  const [reply, setReply] = useState<AssistantReply | null>(null);
  const [asking, setAsking] = useState(false);
  const [ollamaErr, setOllamaErr] = useState<{ summary: string; detail?: string } | null>(null);
  const [queue, setQueue] = useState<AssistantQueueStatus | null>(null);

  const refreshOllama = useCallback(async () => {
    try {
      const status = await invoke<OllamaStatus>("get_ollama_status");
      setOllama(status);
      setOllamaErr(null);
      setModel((prev) => prev || status.models[0] || "");
    } catch (e) {
      setOllamaErr(friendlyOllamaError(String(e), t));
    }
  }, [t]);

  useEffect(() => {
    void refreshOllama();
    void invoke<{ preferredModel?: string | null }>("get_intelligence_settings").then((s) => {
      setModel((prev) => prev || s.preferredModel || "");
    });
  }, [refreshOllama]);

  useEffect(() => {
    const tick = async () => {
      try {
        const q = await invoke<AssistantQueueStatus>("get_assistant_queue_status");
        setQueue(q);
        const done = await invoke<AssistantReply | null>("take_completed_assistant");
        if (done?.text || done?.blockedReason) {
          setReply(done);
        }
      } catch {
        /* sin Tauri */
      }
    };
    const id = window.setInterval(() => void tick(), 2000);
    void tick();
    return () => window.clearInterval(id);
  }, []);

  const submitQuestion = async () => {
    const q = question.trim();
    if (!q) return;
    setAsking(true);
    setReply(null);
    try {
      const res = await invoke<AssistantReply>("ask_assistant", {
        question: q,
        model: model || null,
      });
      setReply(res);
      if (res.queued) {
        setQuestion("");
      }
    } catch (e) {
      setOllamaErr(friendlyOllamaError(String(e), t));
    } finally {
      setAsking(false);
    }
  };

  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 className="font-display text-3xl font-semibold text-white">
            {t("intelligence.title")}
          </h2>
          <p className="mt-2 max-w-2xl text-sm text-white/55">{t("intelligence.subtitle")}</p>
        </div>
        <button
          type="button"
          disabled={diagnosticsRunning}
          onClick={() => void runDiagnostics()}
          className="glass-panel border-accent/30 px-4 py-2.5 text-sm text-accent transition hover:bg-accent/10 disabled:opacity-50"
        >
          {diagnosticsRunning ? t("overview.runningDiagnostics") : t("overview.runDiagnostics")}
        </button>
      </header>

      {lastDiagnosticMessage ? (
        <p className="text-sm text-white/55" role="status">
          {t(`overview.${lastDiagnosticMessage}`)}
        </p>
      ) : null}

      {error ? (
        <div className="glass-panel border-critical/40 p-4 text-sm text-critical">{error}</div>
      ) : null}

      {diagnoses.length === 0 ? (
        <section className="glass-panel p-6 text-sm text-white/50">{t("intelligence.none")}</section>
      ) : (
        <div className="flex flex-col gap-4">
          {diagnoses.map((d) => (
            <DiagnosisCard key={d.id} diagnosis={d} onDismiss={() => void refresh()} />
          ))}
        </div>
      )}

      <section className="glass-panel-strong glass-accent flex flex-col gap-4 p-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 className="font-display text-xl text-[var(--color-lavender)]">
              {t("intelligence.assistantTitle")}
            </h3>
            <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
              {t("intelligence.assistantHint")}
            </p>
          </div>
          <button
            type="button"
            onClick={() => void refreshOllama()}
            className="rounded-xl border border-white/10 px-3 py-1.5 text-xs text-white/70 hover:bg-white/5"
          >
            {t("intelligence.refreshOllama")}
          </button>
        </div>

        {ollamaErr ? (
          <div className="rounded-xl border border-critical/30 bg-critical/10 p-3 text-sm text-critical">
            <p>{ollamaErr.summary}</p>
            {ollamaErr.detail ? (
              <details className="mt-2 text-xs text-white/60">
                <summary className="cursor-pointer">{t("intelligence.technicalDetail")}</summary>
                <pre className="mt-2 whitespace-pre-wrap font-mono">{ollamaErr.detail}</pre>
              </details>
            ) : null}
          </div>
        ) : ollama && !ollama.available ? (
          <p className="text-sm text-[var(--color-text-secondary)]">{t("intelligence.ollamaUnreachable")}</p>
        ) : ollama ? (
          <p className="text-sm text-[var(--color-text-secondary)]">
            {ollama.available ? t("status.supported") : ollama.message}
            {ollama.available ? (
              <span className="ml-2 font-mono text-xs text-white/40">{ollama.baseUrl}</span>
            ) : null}
          </p>
        ) : null}

        <div className="flex flex-col gap-2">
          <label className="text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
            {t("intelligence.model")}
          </label>
          {ollama?.models.length ? (
            <select
              value={model}
              onChange={(e) => setModel(e.target.value)}
              className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm"
            >
              {ollama.models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          ) : (
            <input
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder="llama3.2"
              className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm"
            />
          )}
        </div>

        <textarea
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          rows={3}
          placeholder={t("intelligence.questionPlaceholder")}
          className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm outline-none focus:border-[var(--color-violet)]/40"
        />

        <button
          type="button"
          disabled={asking || !question.trim()}
          onClick={() => void submitQuestion()}
          className="self-start rounded-xl border border-[var(--color-violet)]/35 bg-[var(--color-violet)]/15 px-4 py-2 text-sm text-[var(--color-lavender)] hover:bg-[var(--color-violet)]/25 disabled:opacity-40"
        >
          {asking ? t("intelligence.asking") : t("intelligence.ask")}
        </button>

        {queue?.pending ? (
          <p className="text-sm text-[var(--color-text-secondary)]">
            {t("intelligence.queuePending")}
            {queue.questionPreview ? `: «${queue.questionPreview}»` : null}
          </p>
        ) : null}

        {reply?.blockedReason ? (
          <p className="rounded-xl border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-100/90">
            {reply.blockedReason}
          </p>
        ) : null}

        {reply?.text ? (
          <article className="rounded-xl border border-white/10 bg-black/20 p-4 text-sm leading-relaxed text-white/85 whitespace-pre-wrap">
            {reply.text}
          </article>
        ) : null}
      </section>
    </div>
  );
}

function friendlyOllamaError(
  raw: string,
  t: (key: string, opts?: Record<string, string>) => string,
): { summary: string; detail?: string } {
  const lower = raw.toLowerCase();
  if (
    lower.includes("connection refused") ||
    lower.includes("failed to fetch") ||
    lower.includes("network") ||
    lower.includes("11434")
  ) {
    return { summary: t("intelligence.ollamaUnreachable"), detail: raw };
  }
  return { summary: t("intelligence.ollamaErrorDetail", { detail: raw.split("\n")[0] }), detail: raw };
}
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Download, Radio } from "lucide-react";
import { IncidentTimeline, type TelemetryFrame } from "@/components/blackbox/IncidentTimeline";

interface BlackBoxStatus {
  recording: boolean;
  frames_in_memory: number;
  incidents_stored: number;
  db_path: string;
  session_id: string;
}

interface IncidentSummary {
  id: string;
  detected_at: string;
  severity: string;
  title_key: string;
  rule_id?: string;
  incomplete: boolean;
  pre_samples: number;
  post_samples: number;
}

interface IncidentRecord {
  id: string;
  detected_at: string;
  severity: string;
  title_key: string;
  summary: string;
  pre_window: TelemetryFrame[];
  post_window: TelemetryFrame[];
  incomplete: boolean;
  limitations: string[];
}

function downloadText(filename: string, text: string) {
  const blob = new Blob([text], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export function BlackBoxPage() {
  const { t, i18n } = useTranslation();
  const [status, setStatus] = useState<BlackBoxStatus | null>(null);
  const [incidents, setIncidents] = useState<IncidentSummary[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<IncidentRecord | null>(null);

  const loadList = useCallback(async () => {
    try {
      const [st, list] = await Promise.all([
        invoke<BlackBoxStatus>("get_blackbox_status"),
        invoke<IncidentSummary[]>("list_incidents", { limit: 20 }),
      ]);
      setStatus(st);
      setIncidents(list);
    } catch {
      /* sin Tauri */
    }
  }, []);

  useEffect(() => {
    void loadList();
    const id = setInterval(() => void loadList(), 5000);
    return () => clearInterval(id);
  }, [loadList]);

  useEffect(() => {
    if (!selectedId) {
      setDetail(null);
      return;
    }
    void invoke<IncidentRecord | null>("get_incident", { id: selectedId }).then((r) => setDetail(r ?? null));
  }, [selectedId]);

  const exportTechnical = async () => {
    if (!selectedId) return;
    const json = await invoke<string>("export_incident_technical", { id: selectedId });
    downloadText(`heimdall-incident-${selectedId}.json`, json);
  };

  const exportHuman = async () => {
    if (!selectedId) return;
    const json = await invoke<string>("export_incident_human", { id: selectedId });
    downloadText(`heimdall-incident-${selectedId}-resumen.json`, json);
  };

  return (
    <div className="flex flex-col gap-5">
      <header>
        <h2 className="font-display text-3xl font-semibold">{t("blackbox.title")}</h2>
        <p className="mt-2 max-w-3xl text-sm text-[var(--color-text-secondary)]">{t("blackbox.subtitle")}</p>
        <p className="mt-2 max-w-3xl text-xs text-[var(--color-text-secondary)]">{t("blackbox.investigationsLink")}</p>
      </header>

      <section className="glass-panel-strong glass-accent grid gap-3 p-4 md:grid-cols-3">
        <div className="flex items-center gap-2">
          <Radio
            size={18}
            className={
              status?.recording ? "text-[var(--color-healthy)]" : "text-[var(--color-text-secondary)]"
            }
          />
          <span className="text-sm">
            {status?.recording ? t("blackbox.recording") : t("blackbox.stopped")}
          </span>
        </div>
        <p className="font-mono text-xs text-[var(--color-text-secondary)]">
          {t("blackbox.frames")}: {status?.frames_in_memory ?? "—"}
        </p>
        <p className="font-mono text-xs text-[var(--color-text-secondary)]">
          {t("blackbox.incidents")}: {status?.incidents_stored ?? "—"}
        </p>
      </section>

      <div className="grid min-h-0 gap-4 lg:grid-cols-2">
        <section className="glass-panel flex flex-col gap-2 p-3">
          <h3 className="font-display text-sm font-medium">{t("blackbox.incidentList")}</h3>
          {incidents.length === 0 ? (
            <p className="text-sm text-[var(--color-text-secondary)]">{t("blackbox.empty")}</p>
          ) : (
            <ul className="flex max-h-72 flex-col gap-2 overflow-auto">
              {incidents.map((inc) => (
                <li key={inc.id}>
                  <button
                    type="button"
                    onClick={() => setSelectedId(inc.id)}
                    className={`w-full rounded-2xl border px-3 py-2 text-left text-sm transition ${
                      selectedId === inc.id
                        ? "border-[var(--color-violet)]/40 bg-[var(--color-violet)]/10"
                        : "border-white/5 hover:bg-white/5"
                    }`}
                  >
                    <span className="font-medium">
                      {t(`diagnosis.rules.${inc.title_key}.title`, { defaultValue: inc.title_key })}
                    </span>
                    <span className="mt-1 block text-[10px] text-[var(--color-text-secondary)]">
                      {new Date(inc.detected_at).toLocaleString(i18n.language)} · {inc.severity} ·{" "}
                      {t("blackbox.samples", {
                        pre: inc.pre_samples,
                        post: inc.post_samples,
                      })}
                      {inc.incomplete ? ` · ${t("blackbox.incomplete")}` : ""}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="glass-panel p-4">
          {detail ? (
            <div className="flex flex-col gap-4">
              <div>
                <h3 className="font-display text-lg">
                  {t(`diagnosis.rules.${detail.title_key}.title`, { defaultValue: detail.title_key })}
                </h3>
                <p className="text-xs text-[var(--color-text-secondary)]">
                  {new Date(detail.detected_at).toLocaleString(i18n.language)}
                </p>
              </div>
              <IncidentTimeline
                pre={detail.pre_window}
                post={detail.post_window}
                incidentAt={detail.detected_at}
              />
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={() => void exportTechnical()}
                  className="inline-flex items-center gap-2 rounded-xl border border-white/10 px-3 py-2 text-xs hover:bg-white/5"
                >
                  <Download size={14} />
                  {t("blackbox.exportTechnical")}
                </button>
                <button
                  type="button"
                  onClick={() => void exportHuman()}
                  className="inline-flex items-center gap-2 rounded-xl border border-white/10 px-3 py-2 text-xs hover:bg-white/5"
                >
                  <Download size={14} />
                  {t("blackbox.exportHuman")}
                </button>
              </div>
              {detail.limitations.length > 0 ? (
                <p className="text-xs text-[var(--color-text-secondary)]">{detail.limitations[0]}</p>
              ) : null}
            </div>
          ) : (
            <p className="text-sm text-[var(--color-text-secondary)]">{t("blackbox.selectIncident")}</p>
          )}
        </section>
      </div>
    </div>
  );
}
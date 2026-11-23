import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { formatBytes, formatCpuPercent } from "@/lib/format";
import type { ProcessFootprint, ProcessSample } from "@/types/process";

export function ProcessesPage() {
  const { t, i18n } = useTranslation();
  const [sample, setSample] = useState<ProcessSample | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const s = await invoke<ProcessSample>("get_process_sample");
      setSample(s);
      setLoadError(null);
    } catch (e) {
      setLoadError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void load();
    const id = window.setInterval(() => void load(), 1500);
    return () => window.clearInterval(id);
  }, [load]);

  return (
    <div className="flex flex-col gap-5">
      <header>
        <h2 className="font-display text-3xl font-semibold">{t("processes.title")}</h2>
        <p className="mt-2 text-sm text-[var(--color-text-secondary)]">{t("processes.subtitle")}</p>
        <p className="mt-1 text-xs text-[var(--color-text-secondary)]">{t("processes.memoryHint")}</p>
      </header>

      {loadError ? (
        <div className="glass-panel border-critical/40 p-4 text-sm text-critical">{loadError}</div>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        <ProcessTable
          title={t("processes.byCpu")}
          rows={sample?.byCpu ?? []}
          locale={i18n.language}
          metric="cpu"
        />
        <ProcessTable
          title={t("processes.byMemory")}
          rows={sample?.byMemory ?? []}
          locale={i18n.language}
          metric="memory"
        />
      </div>
    </div>
  );
}

function ProcessTable({
  title,
  rows,
  locale,
  metric,
}: {
  title: string;
  rows: ProcessFootprint[];
  locale: string;
  metric: "cpu" | "memory";
}) {
  const { t } = useTranslation();
  return (
    <section className="glass-panel overflow-hidden p-4">
      <h3 className="font-display mb-3 text-base">{title}</h3>
      {!rows.length ? (
        <p className="text-sm text-[var(--color-text-secondary)]">—</p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-white/10 text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
              <th className="py-2 text-left font-normal">{t("processes.colProcess")}</th>
              <th className="py-2 text-right font-normal">{t("processes.colPid")}</th>
              <th className="py-2 text-right font-normal">
                {metric === "cpu" ? t("processes.colCpu") : t("processes.colMemory")}
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.map((p) => (
              <tr key={`${p.pid}-${p.name}`} className="border-t border-white/5">
                <td className="py-2 pr-2 font-mono text-[var(--color-text-primary)]">{p.name}</td>
                <td className="py-2 text-right text-[var(--color-text-secondary)]">{p.pid}</td>
                <td className="py-2 text-right text-[var(--color-text-secondary)]">
                  {metric === "cpu"
                    ? formatCpuPercent(p.cpu_percent, locale)
                    : formatBytes(p.memory_bytes, locale)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
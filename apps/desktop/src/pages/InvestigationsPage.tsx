import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { RefreshCw, Search } from "lucide-react";
import type { InvestigationDossier, InvestigationLevel, InvestigationState, InvestigationSummary } from "@/types/investigation";
import { InvestigationDossierPanel } from "@/components/investigations/InvestigationDossierPanel";
import { useMonitorStatus } from "@/hooks/useMonitorStatus";

type LevelFilter = InvestigationLevel | "all";
type StateFilter = InvestigationState | "all";

export function InvestigationsPage() {
  const { t, i18n } = useTranslation();
  const [list, setList] = useState<InvestigationSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { status: monitor } = useMonitorStatus();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<InvestigationDossier | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [levelFilter, setLevelFilter] = useState<LevelFilter>("all");
  const [stateFilter, setStateFilter] = useState<StateFilter>("all");
  const [query, setQuery] = useState("");

  const refreshList = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const rows = await invoke<InvestigationSummary[]>("list_investigations", { limit: 80 });
      setList(rows);
    } catch (e) {
      setError(String(e));
      setList([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selectedId) {
      setDetail(null);
      return;
    }
    setDetailLoading(true);
    void invoke<InvestigationDossier | null>("get_investigation", { investigationId: selectedId })
      .then((d) => setDetail(d ?? null))
      .catch(() => setDetail(null))
      .finally(() => setDetailLoading(false));
  }, [selectedId]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return list.filter((row) => {
      if (levelFilter !== "all" && row.level !== levelFilter) return false;
      if (stateFilter !== "all" && row.state !== stateFilter) return false;
      if (!q) return true;
      return (
        row.identity.name.toLowerCase().includes(q) ||
        String(row.identity.pid).includes(q) ||
        row.id.toLowerCase().includes(q)
      );
    });
  }, [list, levelFilter, stateFilter, query]);

  return (
    <div className="flex flex-col gap-5">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="font-display text-3xl font-semibold">{t("investigations.title")}</h2>
          <p className="mt-2 max-w-2xl text-sm text-[var(--color-text-secondary)]">{t("investigations.subtitle")}</p>
          <p className="mt-1 text-xs text-[var(--color-text-secondary)]">{t("investigations.anomalyDisclaimer")}</p>
        </div>
        <button type="button" className="btn-secondary flex items-center gap-2" onClick={() => void refreshList()}>
          <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
          {t("investigations.refresh")}
        </button>
      </header>

      {error ? (
        <p className="rounded-xl border border-[var(--color-danger)]/30 bg-[var(--color-danger)]/10 p-3 text-sm">
          {t("investigations.error")}: {error}
        </p>
      ) : null}
      {monitor ? (
        <p className="rounded-xl border border-white/10 bg-white/5 p-3 text-xs text-[var(--color-text-secondary)]">
          {monitor.mode === "daemon"
            ? t("monitor.daemonConnected")
            : monitor.mode === "embedded"
              ? t("monitor.embeddedActive")
              : t("monitor.unknown")}
          {" · "}
          {t("monitor.investigationsNote")}
        </p>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-[minmax(280px,360px)_1fr]">
        <aside className="glass-panel flex flex-col gap-3 p-4">
          <div className="relative">
            <Search size={16} className="absolute left-3 top-2.5 text-[var(--color-text-secondary)]" />
            <input
              className="w-full rounded-xl border border-white/10 bg-black/20 py-2 pl-9 pr-3 text-sm"
              placeholder={t("investigations.search")}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
          <div className="flex flex-wrap gap-2 text-xs">
            <FilterSelect
              label={t("investigations.filterLevel")}
              value={levelFilter}
              onChange={(v) => setLevelFilter(v as LevelFilter)}
              options={[
                ["all", t("investigations.filterAll")],
                ["event", t("investigations.levels.event")],
                ["anomaly", t("investigations.levels.anomaly")],
                ["incident", t("investigations.levels.incident")],
              ]}
            />
            <FilterSelect
              label={t("investigations.filterState")}
              value={stateFilter}
              onChange={(v) => setStateFilter(v as StateFilter)}
              options={[
                ["all", t("investigations.filterAll")],
                ["open", t("investigations.states.open")],
                ["closed", t("investigations.states.closed")],
                ["incomplete", t("investigations.states.incomplete")],
              ]}
            />
          </div>
          {loading && !list.length ? (
            <p className="text-sm text-[var(--color-text-secondary)]">{t("investigations.loading")}</p>
          ) : null}
          {!loading && !filtered.length ? (
            <p className="text-sm text-[var(--color-text-secondary)]">{t("investigations.empty")}</p>
          ) : null}
          <ul className="max-h-[min(70vh,560px)] space-y-2 overflow-y-auto">
            {filtered.map((row) => (
              <li key={row.id}>
                <button
                  type="button"
                  onClick={() => setSelectedId(row.id)}
                  className={`w-full rounded-2xl border px-3 py-2.5 text-left text-sm transition-colors ${
                    selectedId === row.id
                      ? "border-[var(--color-violet)]/40 bg-[var(--color-violet)]/15"
                      : "border-white/5 bg-white/[0.02] hover:bg-white/5"
                  }`}
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-[var(--color-text-primary)]">
                      {row.identity.name || t("investigations.unknownProcess")} · {row.identity.pid}
                    </span>
                    <span className="text-[10px] uppercase text-[var(--color-text-secondary)]">
                      {t(`investigations.levels.${row.level}`)}
                    </span>
                  </div>
                  <p className="mt-1 text-xs text-[var(--color-text-secondary)]">
                    {new Date(row.updated_at).toLocaleString(i18n.language)}
                  </p>
                  <p className="mt-1 text-[10px] text-[var(--color-text-secondary)]">
                    {t("investigations.listMeta", {
                      attr: t(`investigations.confidence.${row.attribution_confidence}`),
                      impact: t(`investigations.impactLevels.${row.impact_severity}`),
                      state: t(`investigations.states.${row.state}`),
                    })}
                  </p>
                </button>
              </li>
            ))}
          </ul>
        </aside>

        <div className="min-w-0">
          {!selectedId ? (
            <p className="glass-panel p-6 text-sm text-[var(--color-text-secondary)]">{t("investigations.select")}</p>
          ) : detailLoading ? (
            <p className="glass-panel p-6 text-sm text-[var(--color-text-secondary)]">{t("investigations.loadingDetail")}</p>
          ) : detail ? (
            <InvestigationDossierPanel dossier={detail} />
          ) : (
            <p className="glass-panel p-6 text-sm text-[var(--color-danger)]">{t("investigations.detailMissing")}</p>
          )}
        </div>
      </div>
    </div>
  );
}

function FilterSelect({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  options: [string, string][];
}) {
  return (
    <label className="flex items-center gap-1">
      <span className="text-[var(--color-text-secondary)]">{label}</span>
      <select
        className="rounded-lg border border-white/10 bg-black/30 px-2 py-1"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {options.map(([k, text]) => (
          <option key={k} value={k}>
            {text}
          </option>
        ))}
      </select>
    </label>
  );
}
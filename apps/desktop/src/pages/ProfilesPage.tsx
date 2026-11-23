import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { useActivityProfile } from "@/hooks/useActivityProfile";
import type { ActivityProfileKey, ProcessProfileOverride } from "@/types/profile";
import { formatPercentRangeUi, isFiniteNumber } from "@/lib/numbers";

const PROFILE_OPTIONS: ActivityProfileKey[] = [
  "gaming",
  "content_creation",
  "development",
  "general",
  "idle",
];

export function ProfilesPage() {
  const { t, i18n } = useTranslation();
  const { profile, refresh } = useActivityProfile(1500);
  const [overrides, setOverrides] = useState<ProcessProfileOverride[]>([]);
  const [pattern, setPattern] = useState("");
  const [picked, setPicked] = useState<ActivityProfileKey>("gaming");

  const loadOverrides = useCallback(async () => {
    try {
      const list = await invoke<ProcessProfileOverride[]>("list_profile_overrides");
      setOverrides(list);
    } catch {
      /* sin Tauri */
    }
  }, []);

  useEffect(() => {
    void loadOverrides();
  }, [loadOverrides]);

  const saveOverride = async () => {
    if (!pattern.trim()) return;
    await invoke("set_process_profile_override", { pattern: pattern.trim(), profile: picked });
    setPattern("");
    await loadOverrides();
    await refresh();
  };

  return (
    <div className="flex flex-col gap-5">
      <header>
        <h2 className="font-display text-3xl font-semibold">{t("profiles.title")}</h2>
        <p className="mt-2 text-sm text-[var(--color-text-secondary)]">{t("profiles.subtitle")}</p>
      </header>

      <section className="glass-panel-strong glass-accent p-5">
        <p className="text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
          {t("profiles.current")}
        </p>
        <h3 className="font-display mt-2 text-2xl text-[var(--color-lavender)]">
          {profile ? t(`profiles.kinds.${profile.profile}`) : "—"}
        </h3>
        <p className="mt-2 text-sm text-[var(--color-text-secondary)]">
          {profile && isFiniteNumber(profile.confidence)
            ? t("profiles.confidence", {
                pct: Math.round(profile.confidence * 100),
              })
            : profile
              ? t("profiles.baselineInsufficient")
              : null}
          {profile?.learning ? ` · ${t("profiles.learning")}` : ""}
        </p>
        {profile?.profileSince ? (
          <p className="mt-1 font-mono text-xs text-[var(--color-text-secondary)]">
            {t("profiles.since")}{" "}
            {new Date(profile.profileSince).toLocaleString(i18n.language)}
          </p>
        ) : null}
      </section>

      {profile?.baseline ? (
        <section className="glass-panel p-4">
          <h4 className="font-display mb-3 text-base">{t("profiles.baselineTitle")}</h4>
          {profile.baseline.learning ? (
            <p className="text-sm text-[var(--color-text-secondary)]">{t("profiles.baselineLearning")}</p>
          ) : (() => {
            const memRange = formatPercentRangeUi(
              profile.baseline.memUsedRatioTypicalMin,
              profile.baseline.memUsedRatioTypicalMax,
              i18n.language,
            );
            const insufficient =
              !memRange &&
              !isFiniteNumber(profile.baseline.cpuPercentTypical) &&
              profile.baseline.gpuPercentTypical == null;
            if (insufficient) {
              return (
                <p className="text-sm text-[var(--color-text-secondary)]">{t("profiles.baselineInsufficient")}</p>
              );
            }
            return (
              <ul className="flex flex-col gap-2 text-sm text-[var(--color-text-secondary)]">
                <li>{t("profiles.baselineSamples", { count: profile.baseline.sampleCount })}</li>
                {memRange ? <li>{t("profiles.baselineMem", { range: memRange })}</li> : null}
                {isFiniteNumber(profile.baseline.cpuPercentTypical) ? (
                  <li>
                    {t("profiles.baselineCpu", {
                      pct: Math.round(profile.baseline.cpuPercentTypical),
                    })}
                  </li>
                ) : null}
                {profile.baseline.gpuPercentTypical != null &&
                isFiniteNumber(profile.baseline.gpuPercentTypical) ? (
                  <li>
                    {t("profiles.baselineGpu", {
                      pct: Math.round(profile.baseline.gpuPercentTypical),
                    })}
                  </li>
                ) : null}
              </ul>
            );
          })()}
          {profile.baseline.observation ? (
            <p className="mt-3 rounded-xl border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-100/90">
              {profile.baseline.observation}
            </p>
          ) : null}
        </section>
      ) : null}

      <section className="glass-panel p-4">
        <h4 className="font-display mb-3 text-base">{t("profiles.whyTitle")}</h4>
        {!profile?.assessment.signals.length ? (
          <p className="text-sm text-[var(--color-text-secondary)]">{t("profiles.noSignals")}</p>
        ) : (
          <ul className="flex flex-col gap-2 text-sm text-[var(--color-text-secondary)]">
            {profile.assessment.signals.map((s, i) => (
              <li key={`${s.kind}-${i}`} className="rounded-xl border border-white/5 px-3 py-2">
                <span className="text-[var(--color-violet)]">{s.kind}</span> — {s.detail}
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="glass-panel p-4">
        <h4 className="font-display mb-3 text-base">{t("profiles.overrideTitle")}</h4>
        <p className="mb-3 text-sm text-[var(--color-text-secondary)]">{t("profiles.overrideHint")}</p>
        <div className="flex flex-wrap gap-2">
          <input
            value={pattern}
            onChange={(e) => setPattern(e.target.value)}
            placeholder={t("profiles.patternPlaceholder")}
            className="min-w-[12rem] flex-1 rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm outline-none focus:border-[var(--color-violet)]/40"
          />
          <select
            value={picked}
            onChange={(e) => setPicked(e.target.value as ActivityProfileKey)}
            className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm"
          >
            {PROFILE_OPTIONS.map((k) => (
              <option key={k} value={k}>
                {t(`profiles.kinds.${k}`)}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => void saveOverride()}
            className="rounded-xl border border-[var(--color-violet)]/35 bg-[var(--color-violet)]/15 px-4 py-2 text-sm text-[var(--color-lavender)] hover:bg-[var(--color-violet)]/25"
          >
            {t("profiles.saveOverride")}
          </button>
        </div>
        {overrides.length > 0 ? (
          <ul className="mt-4 flex flex-col gap-2 text-sm">
            {overrides.map((o) => (
              <li
                key={o.pattern}
                className="flex justify-between gap-2 rounded-lg bg-white/5 px-3 py-2 text-[var(--color-text-secondary)]"
              >
                <span>
                  <span className="font-mono text-[var(--color-text-primary)]">{o.pattern}</span> →{" "}
                  {t(`profiles.kinds.${o.profile}`)}
                </span>
              </li>
            ))}
          </ul>
        ) : null}
      </section>
    </div>
  );
}
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "@/store/settings";

interface IntelligenceSettings {
  ollamaBaseUrl: string;
  preferredModel: string | null;
  deferWhenBlocked: boolean;
}

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const locale = useSettingsStore((s) => s.locale);
  const setLocale = useSettingsStore((s) => s.setLocale);
  const reducedGlass = useSettingsStore((s) => s.reducedGlass);
  const setReducedGlass = useSettingsStore((s) => s.setReducedGlass);

  const [ollamaUrl, setOllamaUrl] = useState("http://127.0.0.1:11434");
  const [preferredModel, setPreferredModel] = useState("");
  const [deferWhenBlocked, setDeferWhenBlocked] = useState(true);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    try {
      const s = await invoke<IntelligenceSettings>("get_intelligence_settings");
      setOllamaUrl(s.ollamaBaseUrl);
      setPreferredModel(s.preferredModel ?? "");
      setDeferWhenBlocked(s.deferWhenBlocked);
    } catch {
      /* sin Tauri */
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const saveIntelligence = async () => {
    await invoke("set_intelligence_settings", {
      settings: {
        ollamaBaseUrl: ollamaUrl.trim(),
        preferredModel: preferredModel.trim() || null,
        deferWhenBlocked,
      },
    });
    setSaved(true);
    window.setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="flex max-w-2xl flex-col gap-6">
      <header>
        <h2 className="font-display text-3xl font-semibold">{t("settings.title")}</h2>
        <p className="mt-2 text-sm text-[var(--color-text-secondary)]">{t("settings.subtitle")}</p>
      </header>

      <section className="glass-panel p-4">
        <h3 className="font-display mb-3 text-base">{t("settings.appearance")}</h3>
        <label className="flex items-center gap-2 text-sm text-[var(--color-text-secondary)]">
          <input
            type="checkbox"
            checked={reducedGlass}
            onChange={(e) => setReducedGlass(e.target.checked)}
          />
          {t("settings.reducedGlass")}
        </label>
        <div className="mt-4">
          <p className="mb-2 text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
            {t("settings.language")}
          </p>
          <select
            value={locale}
            onChange={(e) => {
              const l = e.target.value as "en" | "es";
              setLocale(l);
              void i18n.changeLanguage(l);
            }}
            className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm"
          >
            <option value="es">Español</option>
            <option value="en">English</option>
          </select>
        </div>
      </section>

      <section className="glass-panel p-4">
        <h3 className="font-display mb-1 text-base">{t("settings.ollamaTitle")}</h3>
        <p className="mb-4 text-sm text-[var(--color-text-secondary)]">{t("settings.ollamaHint")}</p>
        <div className="flex flex-col gap-3">
          <label className="text-xs text-[var(--color-text-secondary)]">{t("settings.ollamaUrl")}</label>
          <input
            value={ollamaUrl}
            onChange={(e) => setOllamaUrl(e.target.value)}
            className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 font-mono text-sm"
          />
          <label className="text-xs text-[var(--color-text-secondary)]">{t("settings.preferredModel")}</label>
          <input
            value={preferredModel}
            onChange={(e) => setPreferredModel(e.target.value)}
            placeholder="llama3.2"
            className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm"
          />
          <label className="flex items-center gap-2 text-sm text-[var(--color-text-secondary)]">
            <input
              type="checkbox"
              checked={deferWhenBlocked}
              onChange={(e) => setDeferWhenBlocked(e.target.checked)}
            />
            {t("settings.deferWhenBlocked")}
          </label>
          <button
            type="button"
            onClick={() => void saveIntelligence()}
            className="self-start rounded-xl border border-[var(--color-violet)]/35 bg-[var(--color-violet)]/15 px-4 py-2 text-sm text-[var(--color-lavender)] hover:bg-[var(--color-violet)]/25"
          >
            {saved ? t("settings.saved") : t("settings.save")}
          </button>
        </div>
      </section>
    </div>
  );
}
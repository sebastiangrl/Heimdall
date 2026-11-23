import { useTranslation } from "react-i18next";
import { useTelemetry } from "@/hooks/useTelemetry";
import { formatBytes, formatCpuPercent } from "@/lib/format";
import { groupStorageMounts } from "@/lib/storageGroups";

export function HardwarePage() {
  const { t, i18n } = useTranslation();
  const { hardware, snapshot } = useTelemetry(2000);

  if (!hardware) {
    return (
      <p className="text-sm text-[var(--color-text-secondary)]">{t("hardware.loading")}</p>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      <header>
        <h2 className="font-display text-3xl font-semibold">{t("hardware.title")}</h2>
        <p className="mt-2 text-sm text-[var(--color-text-secondary)]">{t("hardware.subtitle")}</p>
      </header>

      <section className="glass-panel grid gap-4 p-5 md:grid-cols-2">
        <div>
          <p className="text-xs uppercase text-[var(--color-text-secondary)]">{t("hardware.host")}</p>
          <p className="font-mono text-lg">{hardware.hostname}</p>
          <p className="mt-1 text-xs text-[var(--color-text-secondary)]">
            {hardware.os_name} {hardware.os_version} · {hardware.kernel_version}
          </p>
        </div>
        <div>
          <p className="text-xs uppercase text-[var(--color-text-secondary)]">CPU</p>
          <p className="text-sm">
            {hardware.cpu.name} · {hardware.cpu.logical_cores} hilos
          </p>
        </div>
        <div>
          <p className="text-xs uppercase text-[var(--color-text-secondary)]">RAM</p>
          <p className="text-sm">{formatBytes(hardware.memory.total_bytes, i18n.language)}</p>
        </div>
      </section>

      <section className="glass-panel p-5">
        <h3 className="font-display mb-3 text-base">{t("hardware.gpuListTitle")}</h3>
        {hardware.gpus.length ? (
          <ul className="space-y-2 text-sm">
            {hardware.gpus.map((g) => (
              <li key={g.name} className="border-t border-white/5 pt-2 first:border-0 first:pt-0">
                <span className="text-[var(--color-text-primary)]">{g.name}</span>
                <span className="ml-2 text-[var(--color-text-secondary)]">({g.vendor})</span>
                {g.driver_note ? (
                  <p className="mt-1 text-xs text-[var(--color-text-secondary)]">{g.driver_note}</p>
                ) : null}
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-[var(--color-text-secondary)]">{t("metrics.unavailable")}</p>
        )}
      </section>

      {snapshot?.storage.length ? (
        <section className="glass-panel p-5">
          <h3 className="font-display mb-3 text-base">{t("hardware.storageTitle")}</h3>
          <ul className="space-y-3 text-sm">
            {groupStorageMounts(snapshot.storage).map((g) => (
              <li
                key={`${g.totalBytes}:${g.usedBytes}:${g.mountPoints.join("|")}`}
                className="border-t border-white/5 pt-3 first:border-0 first:pt-0"
              >
                <p className="text-[var(--color-text-primary)]">
                  {formatBytes(g.usedBytes, i18n.language)} /{" "}
                  {formatBytes(g.totalBytes, i18n.language)}
                </p>
                <p className="mt-1 font-mono text-xs leading-relaxed text-[var(--color-text-secondary)]">
                  {g.mountPoints.length > 1
                    ? t("hardware.storageMountsGrouped", { mounts: g.mountPoints.join(" · ") })
                    : g.mountPoints[0] ?? g.deviceNames[0] ?? "—"}
                </p>
                {g.mountPoints.length > 1 && g.deviceNames.length > 1 ? (
                  <p className="mt-1 text-xs text-[var(--color-text-secondary)]">
                    {t("hardware.storageDevicesGrouped", { names: g.deviceNames.join(", ") })}
                  </p>
                ) : null}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {snapshot ? (
        <section className="glass-panel p-5">
          <h3 className="font-display mb-3 text-base">{t("hardware.sensorsTitle")}</h3>
          <dl className="grid gap-3 text-sm md:grid-cols-2">
            <div>
              <dt className="text-xs uppercase text-[var(--color-text-secondary)]">{t("metrics.cpu")}</dt>
              <dd>{formatCpuPercent(snapshot.cpu.usage_percent_global.value, i18n.language)}</dd>
            </div>
            <div>
              <dt className="text-xs uppercase text-[var(--color-text-secondary)]">{t("metrics.memory")}</dt>
              <dd>
                {formatBytes(snapshot.memory.used_bytes.value, i18n.language)} /{" "}
                {formatBytes(snapshot.memory.total_bytes.value, i18n.language)}
              </dd>
            </div>
            {snapshot.memory.swap_used_bytes.value != null ? (
              <div>
                <dt className="text-xs uppercase text-[var(--color-text-secondary)]">{t("metrics.swap")}</dt>
                <dd>{formatBytes(snapshot.memory.swap_used_bytes.value, i18n.language)}</dd>
              </div>
            ) : null}
          </dl>
          <p className="mt-3 text-xs text-[var(--color-text-secondary)]">{t("hardware.liveNote")}</p>
        </section>
      ) : null}
    </div>
  );
}
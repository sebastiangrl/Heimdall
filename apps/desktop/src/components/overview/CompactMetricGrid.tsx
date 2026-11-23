import { useTranslation } from "react-i18next";
import { Cpu, Gpu, HardDrive, MemoryStick, Thermometer } from "lucide-react";
import { formatBytes, formatMetric, formatPercent } from "@/lib/format";
import type { TelemetrySnapshot } from "@/types/telemetry";

function statusLabel(status: string, t: (k: string) => string) {
  if (status === "supported") return t("status.supported");
  const key = `status.${status}`;
  return t(key);
}

export function CompactMetricGrid({ snapshot }: { snapshot: TelemetrySnapshot | null }) {
  const { t, i18n } = useTranslation();
  const locale = i18n.language;

  if (!snapshot) return null;

  const vramPct =
    snapshot.gpu.vram_used_bytes.status === "supported" &&
    snapshot.gpu.vram_total_bytes.status === "supported" &&
    snapshot.gpu.vram_total_bytes.value
      ? (snapshot.gpu.vram_used_bytes.value! / snapshot.gpu.vram_total_bytes.value!) * 100
      : null;

  const memPct =
    snapshot.memory.total_bytes.value && snapshot.memory.used_bytes.value
      ? (snapshot.memory.used_bytes.value / snapshot.memory.total_bytes.value!) * 100
      : null;

  const disk = snapshot.storage[0];

  const cards = [
    {
      key: "cpu",
      icon: Cpu,
      label: t("metrics.cpu"),
      value: formatMetric(snapshot.cpu.usage_percent_global, (v) => formatPercent(v, locale)),
      sub: formatMetric(snapshot.cpu.frequency_mhz_avg, (v) => `${Math.round(v)} MHz`),
      status: snapshot.cpu.usage_percent_global.status,
    },
    {
      key: "memory",
      icon: MemoryStick,
      label: t("metrics.memory"),
      value: memPct !== null ? formatPercent(memPct, locale) : "—",
      sub: formatMetric(snapshot.memory.available_bytes, (v) => `${formatBytes(v, locale)} ${t("metrics.free")}`),
      status: snapshot.memory.used_bytes.status,
    },
    {
      key: "gpu",
      icon: Gpu,
      label: t("metrics.gpu"),
      value: formatMetric(snapshot.gpu.utilization_percent, (v) => formatPercent(v, locale)),
      sub:
        snapshot.gpu.active_provider ??
        (snapshot.gpu.hint_code ? t(snapshot.gpu.hint_code) : t("metrics.unavailable")),
      status: snapshot.gpu.utilization_percent.status,
    },
    {
      key: "vram",
      icon: Gpu,
      label: t("metrics.vram"),
      value: vramPct !== null ? formatPercent(vramPct, locale) : "—",
      sub: formatMetric(snapshot.gpu.vram_used_bytes, (v) => formatBytes(v, locale)),
      status: snapshot.gpu.vram_used_bytes.status,
    },
    {
      key: "disk",
      icon: HardDrive,
      label: t("metrics.storage"),
      value: disk
        ? formatMetric(disk.used_bytes, () => {
            const u = disk.used_bytes.value;
            const tot = disk.total_bytes.value;
            if (u === undefined || !tot) return "—";
            return formatPercent((u / tot) * 100, locale);
          })
        : "—",
      sub: disk?.name ?? "—",
      status: disk?.used_bytes.status ?? "unavailable",
    },
    {
      key: "temp",
      icon: Thermometer,
      label: t("metrics.temperature"),
      value: formatMetric(snapshot.cpu.temperature_c, (v) => `${v.toFixed(1)} °C`),
      sub:
        snapshot.gpu.temperature_c.status === "supported" && snapshot.gpu.temperature_c.value != null
          ? `GPU ${snapshot.gpu.temperature_c.value.toFixed(0)} °C`
          : t("metrics.gpuTempUnavailable"),
      status: snapshot.cpu.temperature_c.status,
    },
  ];

  return (
    <div className="grid grid-cols-2 gap-3 xl:grid-cols-3">
      {cards.map(({ key, icon: Icon, label, value, sub, status }) => (
        <button
          key={key}
          type="button"
          disabled
          title={t("common.comingSoon")}
          className="glass-panel flex flex-col items-start gap-2 p-4 text-left transition hover:border-[var(--color-violet)]/30 disabled:cursor-default"
        >
          <div className="flex w-full items-center justify-between text-[var(--color-text-secondary)]">
            <Icon size={16} className="text-[var(--color-violet)]" />
            <span className="text-[10px] uppercase tracking-wide">{statusLabel(status, t)}</span>
          </div>
          <p className="text-xs text-[var(--color-text-secondary)]">{label}</p>
          <p className="font-mono text-xl text-[var(--color-lavender)]">{value}</p>
          <p className="line-clamp-2 text-xs text-[var(--color-text-secondary)]">{sub}</p>
        </button>
      ))}
    </div>
  );
}
export function formatBytes(bytes: number | undefined, locale = "en"): string {
  if (bytes === undefined || Number.isNaN(bytes)) return "—";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toLocaleString(locale, { maximumFractionDigits: 1 })} ${units[i]}`;
}

export function formatPercent(value: number | undefined, locale = "en"): string {
  if (value === undefined || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString(locale, { maximumFractionDigits: 1 })}%`;
}

export function formatCpuPercent(
  value: number | null | undefined,
  locale = "en",
  maximumFractionDigits = 1,
): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString(locale, { maximumFractionDigits, minimumFractionDigits: 0 })}%`;
}

export function formatMetric<T>(
  metric: { status: string; value?: T },
  format: (v: T) => string,
): string {
  if (metric.status !== "supported" || metric.value === undefined) {
    return "—";
  }
  return format(metric.value);
}
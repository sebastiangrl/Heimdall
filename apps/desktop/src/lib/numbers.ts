export function isFiniteNumber(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

/** Porcentaje para UI (no para exportación JSON). */
export function formatPercentUi(
  value: number | null | undefined,
  locale: string,
  fractionDigits = 1,
): string {
  if (!isFiniteNumber(value)) return "—";
  return `${value.toLocaleString(locale, { maximumFractionDigits: fractionDigits, minimumFractionDigits: 0 })}%`;
}

/** Rango porcentual para línea base de perfil. */
export function formatPercentRangeUi(
  min: number | null | undefined,
  max: number | null | undefined,
  _locale: string,
): string | null {
  if (!isFiniteNumber(min) || !isFiniteNumber(max)) return null;
  const a = Math.round(min * 100);
  const b = Math.round(max * 100);
  if (!Number.isFinite(a) || !Number.isFinite(b)) return null;
  return `${a}–${b}%`;
}
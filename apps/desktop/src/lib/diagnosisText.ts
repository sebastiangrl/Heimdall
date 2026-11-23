import type { TFunction } from "i18next";
import type { Diagnosis } from "@/types/diagnosis";

export function diagnosisTitle(d: Diagnosis, t: TFunction): string {
  const key = `diagnosis.rules.${d.rule_id}.title`;
  const translated = t(key);
  return translated !== key ? translated : d.title;
}

export function diagnosisExplanation(d: Diagnosis, t: TFunction): string {
  const key = `diagnosis.rules.${d.rule_id}.explanation`;
  const translated = t(key);
  if (translated !== key) return translated;
  if (d.rule_id === "memory_pressure" && d.explanation.startsWith("memory_pressure:")) {
    return t("diagnosis.rules.memory_pressure.explanationDynamic", {
      hint: d.explanation.replace("memory_pressure:", ""),
    });
  }
  if (d.rule_id === "cpu_saturation" && d.explanation.startsWith("cpu_saturation:")) {
    return d.explanation.replace("cpu_saturation:", "");
  }
  if (d.rule_id === "storage_pressure" && d.explanation.startsWith("storage_pressure:")) {
    const [, name, mount, ratio] = d.explanation.split(":");
    return t("diagnosis.rules.storage_pressure.explanationDynamic", {
      name,
      mount,
      percent: (parseFloat(ratio) * 100).toFixed(1),
    });
  }
  return d.explanation;
}

export function severityLabel(severity: Diagnosis["severity"], t: TFunction): string {
  return t(`severity.${severity}`);
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisLifecycleState {
    Detected,
    Persistent,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    pub reference: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedProcess {
    pub pid: u32,
    pub name: String,
    /// Porcentaje de un núcleo lógico (100 = un núcleo ocupado). None si no hay muestra finita.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f32>,
    /// Porcentaje de la capacidad total de la máquina. None si no hay muestra finita.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_machine_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    pub id: Uuid,
    pub rule_id: String,
    pub title: String,
    pub explanation: String,
    pub severity: DiagnosisSeverity,
    pub observed_at: DateTime<Utc>,
    pub first_detected_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub lifecycle_state: DiagnosisLifecycleState,
    /// Approximate duration the condition has been active (seconds).
    pub active_seconds: u64,
    pub evidence: Vec<Evidence>,
    pub affected_resources: Vec<String>,
    pub related_processes: Vec<RelatedProcess>,
    pub confidence: Confidence,
    pub recommended_actions: Vec<RecommendedAction>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Huella de un proceso en un tick. `memory_bytes` es RSS vía sysinfo (residente);
/// puede incluir páginas compartidas y no debe sumarse entre procesos como consumo exclusivo.
pub struct ProcessFootprint {
    pub pid: u32,
    pub name: String,
    /// Porcentaje de un núcleo lógico según sysinfo: 100 equivale a un núcleo ocupado
    /// y el valor puede superar 100. None si todavía no hay delta o el número no es finito.
    /// Las reglas que destacan procesos comparan este campo. No es el porcentaje de la máquina.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f32>,
    /// Porcentaje de la capacidad total: `cpu_percent / núcleos lógicos`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_machine_percent: Option<f32>,
    pub memory_bytes: u64,
    /// `/proc/sys/kernel/random/boot_id`. Vacío si aún no se leyó.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub boot_id: String,
    /// Campo 22 de `/proc/[pid]/stat`: ticks de reloj desde el arranque. Junto con
    /// `boot_id` y `pid` distingue un proceso de otro que reutilice el mismo PID.
    /// 0 significa que no se pudo leer, no que el proceso empezara en el tick 0.
    #[serde(default)]
    pub start_time_ticks: u64,
}
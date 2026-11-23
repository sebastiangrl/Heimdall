use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 1 = frames antiguos, solo `top_processes` como `nombre:pid`.
/// 2 = identidad estable y CPU en las dos escalas.
pub const FRAME_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleAvailability {
    Available,
    Unavailable,
}

/// Muestra acotada de un proceso. No incluye argumentos de comando.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalProcessSample {
    pub boot_id: String,
    pub pid: u32,
    pub start_time_ticks: u64,
    pub name: String,
    pub observed_at: DateTime<Utc>,
    /// Porcentaje de un núcleo. None si la muestra no es finita o no había delta.
    pub cpu_core_percent: Option<f32>,
    /// Porcentaje de la capacidad total de la máquina.
    pub cpu_machine_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub availability: SampleAvailability,
}

/// La salida no tiene instante exacto: se observó entre dos muestras.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExitEvent {
    pub boot_id: String,
    pub pid: u32,
    pub start_time_ticks: u64,
    pub name: String,
    pub last_seen_at: DateTime<Utc>,
    pub observed_absent_at: DateTime<Utc>,
    pub exit_observed_between_samples: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReconstruction {
    pub boot_id: String,
    pub pid: u32,
    pub start_time_ticks: u64,
    pub name: String,
    pub samples: Vec<HistoricalProcessSample>,
    pub exit: Option<ProcessExitEvent>,
    /// True cuando el frame solo tenía `nombre:pid` y no hay ticks de inicio.
    pub legacy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub observed_at: DateTime<Utc>,
    pub cpu_percent: Option<f64>,
    pub mem_available_bytes: Option<u64>,
    pub swap_used_bytes: Option<u64>,
    pub gpu_percent: Option<f64>,
    pub vram_used_bytes: Option<u64>,
    pub cpu_temp_c: Option<f64>,
    pub gpu_temp_c: Option<f64>,
    pub psi_mem: Option<f64>,
    #[serde(default)]
    pub psi_cpu: Option<f64>,
    #[serde(default)]
    pub psi_io: Option<f64>,
    pub diagnosis_rules: Vec<String>,
    /// Compatibilidad: `nombre:pid` de los procesos seguidos por CPU.
    pub top_processes: Vec<String>,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub boot_id: String,
    #[serde(default)]
    pub processes: Vec<HistoricalProcessSample>,
    #[serde(default)]
    pub process_exits: Vec<ProcessExitEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub severity: IncidentSeverity,
    pub kind: String,
    pub title_key: String,
    pub summary: String,
    pub rule_id: Option<String>,
    pub pre_window: Vec<TelemetryFrame>,
    pub post_window: Vec<TelemetryFrame>,
    pub incomplete: bool,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentSummary {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub severity: IncidentSeverity,
    pub title_key: String,
    pub rule_id: Option<String>,
    pub incomplete: bool,
    pub pre_samples: usize,
    pub post_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackBoxStatus {
    pub recording: bool,
    pub frames_in_memory: usize,
    pub incidents_stored: u64,
    pub last_checkpoint_at: Option<DateTime<Utc>>,
    pub db_path: String,
    pub session_id: Uuid,
}
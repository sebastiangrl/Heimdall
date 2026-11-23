use chrono::{DateTime, Utc};
use heimdall_blackbox::{HistoricalProcessSample, ProcessExitEvent, TelemetryFrame};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const INVESTIGATION_SCHEMA_VERSION: u32 = 1;

/// Nivel del modelo de investigación (distinto de `IncidentRecord` de diagnóstico).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationLevel {
    Event,
    Anomaly,
    Incident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationState {
    Open,
    Closed,
    Incomplete,
}

/// Confianza de atribución determinista (no probabilística).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionConfidence {
    High,
    Medium,
    Insufficient,
}

/// Severidad de impacto del sistema (independiente de la atribución).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeverity {
    None,
    Minor,
    Major,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessIdentity {
    pub boot_id: String,
    pub pid: u32,
    pub start_time_ticks: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionEvidence {
    pub identity: ProcessIdentity,
    pub activity_interval: TimeInterval,
    pub cpu_core_max: Option<f32>,
    pub cpu_core_mean: Option<f32>,
    pub cpu_machine_mean: Option<f32>,
    /// Intervalos donde `cpu_core >= umbral` de ráfaga.
    pub burst_hit_times: Vec<DateTime<Utc>>,
    /// CPU global media antes de la actividad (referencia previa).
    pub baseline_global_cpu: Option<f64>,
    /// CPU global media durante la ventana de actividad.
    pub activity_global_cpu: Option<f64>,
    /// Incremento observado de CPU global (máquina, puntos porcentuales).
    pub global_cpu_delta: Option<f64>,
    /// Contribución estimada del proceso al incremento (máquina, puntos porcentuales).
    pub process_contribution_estimate: Option<f64>,
    pub temporal_overlap: bool,
    pub alternative_processes: Vec<AlternativeProcess>,
    pub sample_coverage: f64,
    pub samples_available: u32,
    pub samples_total: u32,
    pub exit: Option<ProcessExitEvent>,
    pub confidence: AttributionConfidence,
    /// Umbrales y reglas documentadas (texto fijo, no porcentajes inventados).
    pub confidence_rationale: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeProcess {
    pub identity: ProcessIdentity,
    pub cpu_machine_mean: Option<f32>,
    pub share_of_process_contribution: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub severity: ImpactSeverity,
    pub psi_cpu_some_avg10: Option<f64>,
    pub psi_mem_some_avg10: Option<f64>,
    pub psi_io_some_avg10: Option<f64>,
    pub global_cpu_peak: Option<f64>,
    pub global_cpu_baseline: Option<f64>,
    pub global_cpu_delta: Option<f64>,
    pub recovery_observed: bool,
    pub activity_profile: String,
    pub degradation_confirmed: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationSummary {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: InvestigationState,
    pub level: InvestigationLevel,
    pub identity: ProcessIdentity,
    pub attribution_confidence: AttributionConfidence,
    pub impact_severity: ImpactSeverity,
    pub incomplete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationDossier {
    pub schema_version: u32,
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: InvestigationState,
    pub level: InvestigationLevel,
    pub rule_id: String,
    pub identity: ProcessIdentity,
    pub attribution: AttributionEvidence,
    pub impact: ImpactAssessment,
    pub explanation: String,
    pub process_series: Vec<HistoricalProcessSample>,
    pub context_frames: Vec<TelemetryFrame>,
    pub post_frames: Vec<TelemetryFrame>,
    pub limitations: Vec<String>,
}
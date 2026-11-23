use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::metrics::MetricValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreUsage {
    pub core: u32,
    pub usage_percent: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSnapshot {
    pub usage_percent_global: MetricValue<f64>,
    pub usage_per_core: Vec<CoreUsage>,
    pub frequency_mhz_avg: MetricValue<f64>,
    pub temperature_c: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub used_bytes: MetricValue<u64>,
    pub available_bytes: MetricValue<u64>,
    pub total_bytes: MetricValue<u64>,
    pub swap_used_bytes: MetricValue<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSnapshot {
    pub name: Option<String>,
    pub active_provider: Option<String>,
    /// i18n key for UI hints when telemetry is partial (e.g. `gpu.hint.driver`).
    pub hint_code: Option<String>,
    pub utilization_percent: MetricValue<f64>,
    pub vram_used_bytes: MetricValue<u64>,
    pub vram_total_bytes: MetricValue<u64>,
    pub temperature_c: MetricValue<f64>,
    pub power_watts: MetricValue<f64>,
    pub core_clock_mhz: MetricValue<f64>,
    pub mem_clock_mhz: MetricValue<f64>,
    pub fan_speed_percent: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDeviceSnapshot {
    pub name: String,
    pub mount_point: Option<String>,
    pub used_bytes: MetricValue<u64>,
    pub total_bytes: MetricValue<u64>,
    pub read_bytes_per_sec: MetricValue<f64>,
    pub write_bytes_per_sec: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub rx_bytes_per_sec: MetricValue<f64>,
    pub tx_bytes_per_sec: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub uptime_seconds: MetricValue<u64>,
    pub load_avg_1: MetricValue<f64>,
    pub load_avg_5: MetricValue<f64>,
    pub load_avg_15: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsiSnapshot {
    pub cpu_some_avg10: MetricValue<f64>,
    pub memory_some_avg10: MetricValue<f64>,
    pub io_some_avg10: MetricValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub observed_at: DateTime<Utc>,
    /// Arranque del sampler. Cambia cada vez que se crea `LinuxSampler`.
    pub session_id: Uuid,
    /// Arranque del kernel (`/proc/sys/kernel/random/boot_id`). No cambia al reiniciar Heimdall.
    #[serde(default)]
    pub boot_id: String,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub gpu: GpuSnapshot,
    pub storage: Vec<StorageDeviceSnapshot>,
    pub network: NetworkSnapshot,
    pub system: SystemSnapshot,
    pub psi: PsiSnapshot,
}
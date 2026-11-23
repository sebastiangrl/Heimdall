use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Supported,
    Unsupported,
    Unavailable,
    PermissionDenied,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    None,
    Percent,
    Bytes,
    BytesPerSecond,
    Mhz,
    Celsius,
    Watts,
    Seconds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue<T> {
    pub status: SensorStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    pub observed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Descarta NaN e infinito. Un cero finito se conserva: no es lo mismo que “sin muestra”.
pub fn finite_f64(value: f64) -> Option<f64> {
    if value.is_finite() {
        Some(value)
    } else {
        None
    }
}

pub fn finite_f32(value: f32) -> Option<f32> {
    if value.is_finite() {
        Some(value)
    } else {
        None
    }
}

impl<T> MetricValue<T> {
    pub fn supported(value: T, unit: Unit, source: impl Into<String>) -> Self {
        Self {
            status: SensorStatus::Supported,
            value: Some(value),
            unit: Some(unit),
            observed_at: Utc::now(),
            source: Some(source.into()),
        }
    }

    pub fn unavailable(source: impl Into<String>) -> Self {
        Self {
            status: SensorStatus::Unavailable,
            value: None,
            unit: None,
            observed_at: Utc::now(),
            source: Some(source.into()),
        }
    }

    pub fn unsupported(source: impl Into<String>) -> Self {
        Self {
            status: SensorStatus::Unsupported,
            value: None,
            unit: None,
            observed_at: Utc::now(),
            source: Some(source.into()),
        }
    }

    pub fn permission_denied(source: impl Into<String>) -> Self {
        Self {
            status: SensorStatus::PermissionDenied,
            value: None,
            unit: None,
            observed_at: Utc::now(),
            source: Some(source.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub name: String,
    pub logical_cores: u32,
    pub physical_cores: Option<u32>,
    pub base_frequency_mhz: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_bytes: Option<u64>,
    pub driver_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: Option<String>,
    pub total_bytes: Option<u64>,
    pub filesystem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSummary {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu: CpuCoreInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub disks: Vec<DiskInfo>,
    pub discovered_at: DateTime<Utc>,
}
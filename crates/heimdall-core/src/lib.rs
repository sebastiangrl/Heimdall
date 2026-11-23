pub mod config;
pub mod diagnosis;
pub mod metrics;
pub mod telemetry;
pub mod traits;

pub use config::SamplingConfig;
pub use diagnosis::{
    Confidence, Diagnosis, DiagnosisLifecycleState, DiagnosisSeverity, Evidence, ProcessFootprint,
    RelatedProcess, RecommendedAction,
};
pub use metrics::{
    finite_f32, finite_f64, CpuCoreInfo, DiskInfo, GpuInfo, HardwareSummary, MemoryInfo,
    MetricValue, SensorStatus, Unit,
};
pub use telemetry::{
    CoreUsage, CpuSnapshot, GpuSnapshot, MemorySnapshot, NetworkSnapshot, PsiSnapshot,
    StorageDeviceSnapshot, SystemSnapshot, TelemetrySnapshot,
};
pub use traits::{CollectorError, CollectorResult, MetricCollector};
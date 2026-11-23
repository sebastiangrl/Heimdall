mod config;
mod db;
mod export;
mod process_track;
mod recorder;
mod types;

pub use config::BlackBoxConfig;
pub use process_track::LiveProcess;
pub use recorder::BlackBoxRecorder;
pub use types::{
    BlackBoxStatus, HistoricalProcessSample, IncidentRecord, IncidentSeverity, IncidentSummary,
    ProcessExitEvent, ProcessReconstruction, SampleAvailability, TelemetryFrame,
    FRAME_SCHEMA_VERSION,
};
// re-export for telemetry
pub use export::{export_incident_human_json, export_incident_technical_json};
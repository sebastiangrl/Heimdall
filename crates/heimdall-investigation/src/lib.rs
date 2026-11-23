mod types;
mod burst;
mod config;
mod engine;
mod evidence;
mod impact;
mod store;
#[cfg(test)]
mod tests;

pub use burst::{burst_status_for_identity, identities_with_burst, collect_identity_samples};
pub use config::InvestigationConfig;
pub use engine::{InvestigationEngine, InvestigationStepResult, RULE_INTERMITTENT_BURST};
pub use evidence::build_attribution;
pub use impact::{assess_impact, classify_level};
pub use store::{default_db_path, dossier_context_json, InvestigationStore};
pub use types::{
    AttributionConfidence, AttributionEvidence, ImpactAssessment, ImpactSeverity,
    InvestigationDossier, InvestigationLevel, InvestigationState, InvestigationSummary,
    ProcessIdentity, INVESTIGATION_SCHEMA_VERSION,
};
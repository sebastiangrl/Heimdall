use chrono::Utc;
use heimdall_core::{
    Confidence, Diagnosis, DiagnosisLifecycleState, DiagnosisSeverity, Evidence, RelatedProcess,
    RecommendedAction,
};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub fn shell_diagnosis(
    rule_id: impl Into<String>,
    severity: DiagnosisSeverity,
    explanation: impl Into<String>,
    evidence: Vec<Evidence>,
    affected_resources: Vec<String>,
    related_processes: Vec<RelatedProcess>,
    confidence: Confidence,
    recommended_actions: Vec<RecommendedAction>,
    limitations: Vec<String>,
) -> Diagnosis {
    let now = Utc::now();
    Diagnosis {
        id: Uuid::new_v4(),
        rule_id: rule_id.into(),
        title: String::new(),
        explanation: explanation.into(),
        severity,
        observed_at: now,
        first_detected_at: now,
        last_seen_at: now,
        lifecycle_state: DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence,
        affected_resources,
        related_processes,
        confidence,
        recommended_actions,
        limitations,
    }
}
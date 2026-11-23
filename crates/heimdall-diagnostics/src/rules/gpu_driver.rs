use chrono::Utc;
use heimdall_core::{
    Confidence, Diagnosis, DiagnosisSeverity, Evidence, RecommendedAction,
};
use uuid::Uuid;

use crate::journal::JournalHint;
use crate::journal::JournalHintKind;

/// NVIDIA / driver errors corroborated by journald — not triggered by GPU % alone.
pub fn evaluate_gpu_driver_journal(hints: &[JournalHint]) -> Option<Diagnosis> {
    let nvrm: Vec<_> = hints
        .iter()
        .filter(|h| h.kind == JournalHintKind::Nvrm)
        .take(3)
        .collect();
    if nvrm.is_empty() {
        return None;
    }

    let evidence: Vec<Evidence> = nvrm
        .iter()
        .map(|h| Evidence {
            kind: "journal".into(),
            reference: "journald:nvrm".into(),
            summary: h.message.chars().take(220).collect(),
        })
        .collect();

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "gpu_driver".into(),
        title: "gpu_driver".into(),
        explanation: "gpu_driver:journal_nvrm".into(),
        severity: DiagnosisSeverity::Warning,
        observed_at: Utc::now(),
        first_detected_at: Utc::now(),
        last_seen_at: Utc::now(),
        lifecycle_state: heimdall_core::DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence,
        affected_resources: vec!["gpu".into()],
        related_processes: vec![],
        confidence: Confidence::High,
        recommended_actions: vec![RecommendedAction {
            kind: "open_view".into(),
            label: "Review GPU diagnostics".into(),
            detail: Some("Check driver logs and temperature before changing drivers.".into()),
        }],
        limitations: vec![
            "A single NVRM log line may be benign; confirm with sustained symptoms.".into(),
            "High GPU usage in games is not automatically a driver fault.".into(),
        ],
    })
}
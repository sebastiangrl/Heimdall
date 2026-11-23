use chrono::Utc;
use heimdall_core::{Confidence, Diagnosis, DiagnosisSeverity, Evidence, RecommendedAction, SensorStatus};
use uuid::Uuid;

use crate::config::DiagnosticConfig;
use crate::util::{metric_f64, window_all};
use heimdall_core::TelemetrySnapshot;

pub fn evaluate_io_contention(
    config: &DiagnosticConfig,
    history: &[TelemetrySnapshot],
    current: &TelemetrySnapshot,
) -> Option<Diagnosis> {
    if current.psi.io_some_avg10.status != SensorStatus::Supported {
        return None;
    }

    let sustained = window_all(history, config.io_window_samples, |s| {
        metric_f64(&s.psi.io_some_avg10)
            .map(|v| v >= config.psi_io_warn)
            .unwrap_or(false)
    });

    if !sustained {
        return None;
    }

    let io_now = metric_f64(&current.psi.io_some_avg10).unwrap_or(0.0);

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "io_contention".into(),
        title: "io_contention".into(),
        explanation: "io_contention".into(),
        severity: DiagnosisSeverity::Warning,
        observed_at: Utc::now(),
        first_detected_at: Utc::now(),
        last_seen_at: Utc::now(),
        lifecycle_state: heimdall_core::DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence: vec![Evidence {
            kind: "metric".into(),
            reference: "psi.io_some_avg10".into(),
            summary: format!("PSI I/O (some) avg10: {io_now:.2} over the window."),
        }],
        affected_resources: vec!["storage".into(), "io".into()],
        related_processes: vec![],
        confidence: Confidence::Medium,
        recommended_actions: vec![RecommendedAction {
            kind: "open_view".into(),
            label: "Inspect disk activity".into(),
            detail: Some("Pause heavy transfers if safe to do so.".into()),
        }],
        limitations: vec![
            "Heimdall does not yet attribute I/O to individual processes in this rule.".into(),
        ],
    })
}
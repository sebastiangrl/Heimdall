use chrono::Utc;
use heimdall_core::{Confidence, Diagnosis, DiagnosisSeverity, Evidence, RecommendedAction, SensorStatus};
use uuid::Uuid;

use crate::config::DiagnosticConfig;
use crate::util::{metric_f64, metric_u64, window_all};
use heimdall_core::TelemetrySnapshot;

/// VRAM pressure — high utilization of VRAM over time. High GPU compute % alone is not flagged here.
pub fn evaluate_gpu_vram_pressure(
    config: &DiagnosticConfig,
    history: &[TelemetrySnapshot],
    current: &TelemetrySnapshot,
) -> Option<Diagnosis> {
    if current.gpu.vram_total_bytes.status != SensorStatus::Supported
        || current.gpu.vram_used_bytes.status != SensorStatus::Supported
    {
        return None;
    }

    let ratio_window = window_all(history, config.gpu_vram_window_samples, |s| {
        match (metric_u64(&s.gpu.vram_used_bytes), metric_u64(&s.gpu.vram_total_bytes)) {
            (Some(used), Some(total)) if total > 0 => {
                (used as f64 / total as f64) >= config.gpu_vram_ratio_warn
            }
            _ => false,
        }
    });

    if !ratio_window {
        return None;
    }

    let used = metric_u64(&current.gpu.vram_used_bytes).unwrap_or(0);
    let total = metric_u64(&current.gpu.vram_total_bytes).unwrap_or(1);
    let ratio = used as f64 / total as f64;
    let gpu_util = metric_f64(&current.gpu.utilization_percent).unwrap_or(0.0);

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "gpu_vram_pressure".into(),
        title: "gpu_vram_pressure".into(),
        explanation: "gpu_vram_pressure".into(),
        severity: if ratio >= 0.97 {
            DiagnosisSeverity::Critical
        } else {
            DiagnosisSeverity::Warning
        },
        observed_at: Utc::now(),
        first_detected_at: Utc::now(),
        last_seen_at: Utc::now(),
        lifecycle_state: heimdall_core::DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence: vec![
            Evidence {
                kind: "metric".into(),
                reference: "gpu.vram_used_bytes".into(),
                summary: format!("vram_ratio:{:.3}", ratio),
            },
            Evidence {
                kind: "metric".into(),
                reference: "gpu.utilization_percent".into(),
                summary: format!("gpu_util:{:.1}", gpu_util),
            },
        ],
        affected_resources: vec!["gpu".into(), "vram".into()],
        related_processes: vec![],
        confidence: Confidence::High,
        recommended_actions: vec![RecommendedAction {
            kind: "recommend_close_apps".into(),
            label: "review_gpu_workloads".into(),
            detail: Some("close_vram_heavy".into()),
        }],
        limitations: vec!["gpu_high_util_not_causal".into()],
    })
}
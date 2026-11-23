use heimdall_core::{Confidence, Diagnosis, DiagnosisSeverity, Evidence, RecommendedAction, SensorStatus};

use crate::config::DiagnosticConfig;
use crate::helpers::shell_diagnosis;
use crate::util::{metric_f64, window_all};

pub fn evaluate_gpu_thermal(
    config: &DiagnosticConfig,
    history: &[heimdall_core::TelemetrySnapshot],
    current: &heimdall_core::TelemetrySnapshot,
) -> Option<Diagnosis> {
    if current.gpu.temperature_c.status != SensorStatus::Supported {
        return None;
    }
    let util = metric_f64(&current.gpu.utilization_percent).unwrap_or(0.0);
    let sustained_hot = window_all(history, config.thermal_window_samples, |s| {
        metric_f64(&s.gpu.temperature_c)
            .map(|t| t >= config.gpu_temp_warn_c)
            .unwrap_or(false)
    });
    if !sustained_hot {
        return None;
    }
    // High GPU % during gaming is expected; require heat + moderate util OR critical temp.
    let temp = metric_f64(&current.gpu.temperature_c).unwrap_or(0.0);
    let critical = temp >= config.gpu_temp_crit_c;
    if !critical && util < 40.0 {
        return None;
    }

    Some(shell_diagnosis(
        "thermal_gpu",
        if critical {
            DiagnosisSeverity::Critical
        } else {
            DiagnosisSeverity::Warning
        },
        "thermal_gpu",
        vec![
            Evidence {
                kind: "metric".into(),
                reference: "gpu.temperature_c".into(),
                summary: format!("temp_c:{temp:.1}"),
            },
            Evidence {
                kind: "metric".into(),
                reference: "gpu.utilization_percent".into(),
                summary: format!("util:{util:.1}"),
            },
        ],
        vec!["gpu".into(), "thermal".into()],
        vec![],
        Confidence::Medium,
        vec![RecommendedAction {
            kind: "open_view".into(),
            label: "review_thermals".into(),
            detail: None,
        }],
        vec!["gaming_load_not_automatic_fault".into()],
    ))
}
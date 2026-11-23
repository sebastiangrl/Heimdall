use heimdall_core::{DiagnosisSeverity, SensorStatus};

use crate::config::DiagnosticConfig;
use crate::helpers::shell_diagnosis;
use crate::util::{metric_f64, window_all};
use heimdall_core::{Confidence, Diagnosis, Evidence, RecommendedAction};

pub fn evaluate_cpu_thermal(
    config: &DiagnosticConfig,
    history: &[heimdall_core::TelemetrySnapshot],
    current: &heimdall_core::TelemetrySnapshot,
) -> Option<Diagnosis> {
    if current.cpu.temperature_c.status != SensorStatus::Supported {
        return None;
    }
    let sustained = window_all(history, config.thermal_window_samples, |s| {
        metric_f64(&s.cpu.temperature_c)
            .map(|t| t >= config.cpu_temp_warn_c)
            .unwrap_or(false)
    });
    if !sustained {
        return None;
    }
    let temp = metric_f64(&current.cpu.temperature_c).unwrap_or(0.0);
    Some(shell_diagnosis(
        "thermal_cpu",
        if temp >= config.cpu_temp_crit_c {
            DiagnosisSeverity::Critical
        } else {
            DiagnosisSeverity::Warning
        },
        "thermal_cpu",
        vec![Evidence {
            kind: "metric".into(),
            reference: "cpu.temperature_c".into(),
            summary: format!("temp_c:{temp:.1}"),
        }],
        vec!["cpu".into(), "thermal".into()],
        vec![],
        Confidence::Medium,
        vec![RecommendedAction {
            kind: "open_view".into(),
            label: "review_thermals".into(),
            detail: Some("check_cooling".into()),
        }],
        vec!["high_cpu_load_may_raise_temp".into()],
    ))
}
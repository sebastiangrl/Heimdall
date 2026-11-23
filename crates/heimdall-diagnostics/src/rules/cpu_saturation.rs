use chrono::Utc;
use heimdall_core::{
    Confidence, Diagnosis, DiagnosisSeverity, Evidence, ProcessFootprint, RecommendedAction,
};
use uuid::Uuid;

use crate::config::DiagnosticConfig;
use crate::util::{metric_f64, window_all};
use heimdall_core::TelemetrySnapshot;

pub fn evaluate_cpu_saturation(
    config: &DiagnosticConfig,
    history: &[TelemetrySnapshot],
    current: &TelemetrySnapshot,
    logical_cores: u32,
    top_cpu: &[ProcessFootprint],
) -> Option<Diagnosis> {
    let cpu_high = window_all(history, config.cpu_window_samples, |s| {
        metric_f64(&s.cpu.usage_percent_global)
            .map(|v| v >= config.cpu_saturation_percent)
            .unwrap_or(false)
    });

    let load = metric_f64(&current.system.load_avg_1).unwrap_or(0.0);
    let load_high = logical_cores > 0 && load >= logical_cores as f64 * config.load_per_core_warn;

    if !cpu_high && !load_high {
        return None;
    }

    let cpu_now = metric_f64(&current.cpu.usage_percent_global);
    let heavy: Vec<_> = top_cpu
        .iter()
        .filter(|p| {
            p.cpu_percent
                .filter(|v| v.is_finite())
                .is_some_and(|v| v >= config.process_cpu_highlight)
        })
        .take(4)
        .map(|p| heimdall_core::RelatedProcess {
            pid: p.pid,
            name: p.name.clone(),
            cpu_percent: p.cpu_percent.filter(|v| v.is_finite()),
            cpu_machine_percent: p.cpu_machine_percent.filter(|v| v.is_finite()),
            memory_bytes: Some(p.memory_bytes),
        })
        .collect();

    let cpu_txt = metric_f64(&current.cpu.usage_percent_global)
        .map(|v| format!("{v:.0}"))
        .unwrap_or_else(|| "n/d".into());
    let explanation = if heavy.is_empty() {
        "CPU utilization stayed high for the configured interval. No single process exceeded the per-core highlight threshold; the load may be spread or include kernel work."
            .to_string()
    } else {
        let machine = heavy[0]
            .cpu_machine_percent
            .map(|v| format!("{v:.1}% of machine capacity"))
            .unwrap_or_else(|| "machine share unavailable".into());
        format!(
            "Global CPU stayed near {cpu_txt}% with load {load:.2} on {logical_cores} logical cores. {} accounts for {:.1}% of one logical CPU ({machine}) in the latest sample. This is temporal correlation with the saturated sample, not a demonstrated sole cause.",
            heavy[0].name,
            heavy[0].cpu_percent.unwrap_or(0.0)
        )
    };

    let heavy_count = heavy.len();

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "cpu_saturation".into(),
        title: "cpu_saturation".into(),
        explanation: format!("cpu_saturation:{explanation}"),
        severity: if cpu_now.is_some_and(|v| v >= 97.0) {
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
                reference: "cpu.usage_percent_global".into(),
                summary: format!("Global CPU ~{cpu_txt}% of machine capacity."),
            },
            Evidence {
                kind: "metric".into(),
                reference: "system.load_avg_1".into(),
                summary: format!("Load average (1m): {load:.2}."),
            },
        ],
        affected_resources: vec!["cpu".into()],
        related_processes: heavy,
        confidence: if cpu_high && heavy_count > 0 {
            Confidence::High
        } else {
            Confidence::Medium
        },
        recommended_actions: vec![RecommendedAction {
            kind: "recommend_close_apps".into(),
            label: "Review heavy processes".into(),
            detail: Some("Avoid killing processes without explicit approval.".into()),
        }],
        limitations: vec![
            "High CPU during gaming, rendering, or builds may be expected — confirm context.".into(),
        ],
    })
}
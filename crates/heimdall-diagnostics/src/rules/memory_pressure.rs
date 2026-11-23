use chrono::Utc;
use heimdall_core::{
    Confidence, Diagnosis, DiagnosisSeverity, Evidence, ProcessFootprint, RecommendedAction,
    SensorStatus,
};
use uuid::Uuid;

use crate::config::DiagnosticConfig;
use crate::util::{available_memory_ratio, metric_f64, metric_u64, window_all};
use heimdall_core::TelemetrySnapshot;

pub fn evaluate_memory_pressure(
    config: &DiagnosticConfig,
    history: &[TelemetrySnapshot],
    current: &TelemetrySnapshot,
    top_memory: &[ProcessFootprint],
) -> Option<Diagnosis> {
    let sustained_low = window_all(history, config.memory_window_samples, |s| {
        available_memory_ratio(s)
            .map(|r| r < config.memory_available_ratio_warn)
            .unwrap_or(false)
    });

    if !sustained_low {
        return None;
    }

    let swap = metric_u64(&current.memory.swap_used_bytes).unwrap_or(0);
    let swap_growth = swap_increased(history, config.memory_window_samples, config.swap_growth_bytes_min);

    let psi_high = current.psi.memory_some_avg10.status == SensorStatus::Supported
        && metric_f64(&current.psi.memory_some_avg10)
            .map(|v| v >= config.psi_memory_warn)
            .unwrap_or(false);

    let swap_active = swap_growth || swap > config.swap_growth_bytes_min / 4;

    if !swap_active && !psi_high {
        return None;
    }

    let ratio = available_memory_ratio(current)?;
    let avail = metric_u64(&current.memory.available_bytes).unwrap_or(0);

    let mut evidence = vec![
        Evidence {
            kind: "metric".into(),
            reference: "memory.available_bytes".into(),
            summary: format!(
                "Available RAM is {:.1}% of total ({} bytes free).",
                ratio * 100.0,
                avail
            ),
        },
        Evidence {
            kind: "metric".into(),
            reference: "memory.swap_used_bytes".into(),
            summary: format!(
                "Swap in use: {} bytes{}.",
                swap,
                if swap_growth { " (rising)" } else { "" }
            ),
        },
    ];

    if psi_high {
        if let Some(psi) = metric_f64(&current.psi.memory_some_avg10) {
            evidence.push(Evidence {
                kind: "metric".into(),
                reference: "psi.memory_some_avg10".into(),
                summary: format!("PSI memory (some) avg10: {psi:.2}."),
            });
        }
    }

    let related: Vec<_> = top_memory
        .iter()
        .take(3)
        .map(|p| heimdall_core::RelatedProcess {
            pid: p.pid,
            name: p.name.clone(),
            cpu_percent: p.cpu_percent.filter(|v| v.is_finite()),
            cpu_machine_percent: p.cpu_machine_percent.filter(|v| v.is_finite()),
            memory_bytes: Some(p.memory_bytes),
        })
        .collect();

    let process_hint = if related.is_empty() {
        "Identify memory-heavy applications in the Processes view.".to_string()
    } else {
        format!(
            "{} and {} other process(es) use noticeable memory.",
            related.first().map(|p| p.name.as_str()).unwrap_or("unknown"),
            related.len().saturating_sub(1)
        )
    };

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "memory_pressure".into(),
        title: "memory_pressure".into(),
        explanation: format!("memory_pressure:{process_hint}"),
        severity: if ratio < config.memory_available_ratio_warn * 0.5 {
            DiagnosisSeverity::Critical
        } else {
            DiagnosisSeverity::Warning
        },
        observed_at: Utc::now(),
        first_detected_at: Utc::now(),
        last_seen_at: Utc::now(),
        lifecycle_state: heimdall_core::DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence,
        affected_resources: vec!["memory".into(), "swap".into()],
        related_processes: related,
        confidence: if psi_high && swap_active {
            Confidence::High
        } else {
            Confidence::Medium
        },
        recommended_actions: vec![
            RecommendedAction {
                kind: "recommend_close_apps".into(),
                label: "Close unnecessary applications".into(),
                detail: Some("Prefer closing tabs or workloads you can safely stop.".into()),
            },
            RecommendedAction {
                kind: "open_view".into(),
                label: "Open Processes".into(),
                detail: Some("Inspect top memory consumers.".into()),
            },
        ],
        limitations: vec![
            "Correlation does not prove a single application caused the pressure.".into(),
            "Normal cache usage can temporarily lower “available” RAM on Linux.".into(),
        ],
    })
}

fn swap_increased(history: &[TelemetrySnapshot], window: usize, min_delta: u64) -> bool {
    if history.len() < window {
        return false;
    }
    let recent: Vec<_> = history.iter().rev().take(window).collect();
    let first = metric_u64(&recent.last().unwrap().memory.swap_used_bytes).unwrap_or(0);
    let last = metric_u64(&recent.first().unwrap().memory.swap_used_bytes).unwrap_or(0);
    last > first && last.saturating_sub(first) >= min_delta
}
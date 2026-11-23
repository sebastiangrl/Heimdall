use chrono::{DateTime, Utc};
use heimdall_blackbox::TelemetryFrame;

use crate::burst::{frame_global_cpu, BurstStatus};
use crate::config::InvestigationConfig;
use crate::types::{ImpactAssessment, ImpactSeverity, InvestigationLevel};

pub fn assess_impact(
    status: &BurstStatus,
    frames: &[TelemetryFrame],
    activity_profile: &str,
    config: &InvestigationConfig,
) -> ImpactAssessment {
    let start = status.window_start;
    let end = status.window_end;
    let after_end = end + chrono::Duration::seconds(10);

    let psi_cpu = max_in_window(frames, start, end, |f| f.psi_cpu);
    let psi_mem = max_in_window(frames, start, end, |f| f.psi_mem);
    let psi_io = max_in_window(frames, start, end, |f| f.psi_io);

    let baseline = mean_in_window(frames, start - chrono::Duration::seconds(15), start, |f| f.cpu_percent);
    let activity = mean_in_window(frames, start, end, |f| f.cpu_percent);
    let peak = max_in_window(frames, start, end, |f| f.cpu_percent);
    let delta = match (baseline, activity) {
        (Some(b), Some(a)) => Some(a - b),
        _ => None,
    };

    let recovery_observed = frames
        .iter()
        .filter(|f| f.observed_at > end && f.observed_at <= after_end)
        .filter_map(frame_global_cpu)
        .any(|v| baseline.is_some_and(|b| v <= b + 5.0));

    let mut notes = Vec::new();
    let mut degradation = false;
    if psi_cpu.is_some_and(|v| v >= config.psi_cpu_warn) {
        notes.push(format!("PSI CPU some avg10 >= {}", config.psi_cpu_warn));
        degradation = true;
    }
    if psi_mem.is_some_and(|v| v >= config.psi_mem_warn) {
        notes.push(format!("PSI memoria some avg10 >= {}", config.psi_mem_warn));
        degradation = true;
    }
    if psi_io.is_some_and(|v| v >= config.psi_io_warn) {
        notes.push(format!("PSI E/S some avg10 >= {}", config.psi_io_warn));
        degradation = true;
    }
    if peak.is_some_and(|v| v >= config.global_cpu_saturation) {
        notes.push(format!(
            "CPU global alcanzó o superó {} % de la máquina.",
            config.global_cpu_saturation
        ));
        degradation = true;
    }
    if !degradation {
        notes.push(
            "No hay evidencia de degradación del sistema en la ventana; la actividad puede ser legítima."
                .into(),
        );
    }

    let severity = if degradation {
        if peak.is_some_and(|v| v >= 95.0) || psi_cpu.is_some_and(|v| v >= 30.0) {
            ImpactSeverity::Major
        } else {
            ImpactSeverity::Minor
        }
    } else {
        ImpactSeverity::None
    };

    ImpactAssessment {
        severity,
        psi_cpu_some_avg10: psi_cpu,
        psi_mem_some_avg10: psi_mem,
        psi_io_some_avg10: psi_io,
        global_cpu_peak: peak,
        global_cpu_baseline: baseline,
        global_cpu_delta: delta,
        recovery_observed,
        activity_profile: activity_profile.to_string(),
        degradation_confirmed: degradation,
        notes,
    }
}

pub fn classify_level(impact: &ImpactAssessment, global_delta: Option<f64>) -> InvestigationLevel {
    if impact.degradation_confirmed {
        return InvestigationLevel::Incident;
    }
    if global_delta.is_some_and(|d| d >= 3.0) {
        return InvestigationLevel::Anomaly;
    }
    InvestigationLevel::Event
}

fn mean_in_window(
    frames: &[TelemetryFrame],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    pick: fn(&TelemetryFrame) -> Option<f64>,
) -> Option<f64> {
    let vals: Vec<f64> = frames
        .iter()
        .filter(|f| f.observed_at >= from && f.observed_at <= to)
        .filter_map(pick)
        .filter(|v| v.is_finite())
        .collect();
    if vals.is_empty() {
        return None;
    }
    Some(vals.iter().sum::<f64>() / vals.len() as f64)
}

fn max_in_window(
    frames: &[TelemetryFrame],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    pick: fn(&TelemetryFrame) -> Option<f64>,
) -> Option<f64> {
    frames
        .iter()
        .filter(|f| f.observed_at >= from && f.observed_at <= to)
        .filter_map(pick)
        .filter(|v| v.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}
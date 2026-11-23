use chrono::{DateTime, Utc};
use heimdall_blackbox::{HistoricalProcessSample, TelemetryFrame};

use crate::burst::{collect_identity_samples, BurstStatus, frame_global_cpu};
use crate::config::InvestigationConfig;
use crate::types::{
    AlternativeProcess, AttributionConfidence, AttributionEvidence, ProcessIdentity, TimeInterval,
};

pub fn build_attribution(
    status: &BurstStatus,
    frames: &[TelemetryFrame],
    series: &[HistoricalProcessSample],
    config: &InvestigationConfig,
) -> AttributionEvidence {
    let identity = status.identity.clone();
    let start = status
        .hits
        .first()
        .map(|h| h.at)
        .unwrap_or(status.window_start);
    let end = status
        .hits
        .last()
        .map(|h| h.at)
        .unwrap_or(status.window_end);
    let burst_hit_times: Vec<_> = status.hits.iter().map(|h| h.at).collect();

    let core_vals: Vec<f32> = status.hits.iter().map(|h| h.cpu_core).collect();
    let machine_vals: Vec<f32> = status
        .hits
        .iter()
        .filter_map(|h| h.cpu_machine)
        .collect();
    let cpu_core_max = core_vals.iter().copied().reduce(f32::max);
    let cpu_core_mean = mean_f32(&core_vals);
    let cpu_machine_mean = mean_f32(&machine_vals);

    let baseline_start = start - chrono::Duration::seconds(config.baseline_lookback_secs);
    let baseline_global = mean_global_cpu(frames, baseline_start, start);
    let activity_global = mean_global_cpu(frames, start, end);
    let global_delta = match (baseline_global, activity_global) {
        (Some(b), Some(a)) => Some(a - b),
        _ => None,
    };
    let process_contribution = cpu_machine_mean.map(|v| v as f64);

    let alternatives = alternative_processes(frames, &identity, start, end, process_contribution);

    let (samples_available, samples_total) = series_stats(series, &identity);
    let sample_coverage = if samples_total == 0 {
        0.0
    } else {
        samples_available as f64 / samples_total as f64
    };

    let mut rationale = Vec::new();
    let confidence = classify_confidence(
        config,
        global_delta,
        process_contribution,
        sample_coverage,
        samples_available,
        &mut rationale,
    );

    let temporal_overlap = global_delta.is_some() && process_contribution.is_some();

    AttributionEvidence {
        identity,
        activity_interval: TimeInterval { start, end },
        cpu_core_max,
        cpu_core_mean,
        cpu_machine_mean,
        burst_hit_times,
        baseline_global_cpu: baseline_global,
        activity_global_cpu: activity_global,
        global_cpu_delta: global_delta,
        process_contribution_estimate: process_contribution,
        temporal_overlap,
        alternative_processes: alternatives,
        sample_coverage,
        samples_available,
        samples_total,
        exit: None,
        confidence,
        confidence_rationale: rationale,
    }
}

pub fn attach_exit(
    attribution: &mut AttributionEvidence,
    exit: heimdall_blackbox::ProcessExitEvent,
) {
    attribution.exit = Some(exit);
}

fn mean_f32(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f32>() / values.len() as f32)
}

fn mean_global_cpu(
    frames: &[TelemetryFrame],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Option<f64> {
    let mut vals = Vec::new();
    for frame in frames {
        if frame.observed_at < from || frame.observed_at > to {
            continue;
        }
        if let Some(v) = frame_global_cpu(frame) {
            vals.push(v);
        }
    }
    if vals.is_empty() {
        return None;
    }
    Some(vals.iter().sum::<f64>() / vals.len() as f64)
}

fn series_stats(series: &[HistoricalProcessSample], id: &ProcessIdentity) -> (u32, u32) {
    let mut total = 0u32;
    let mut avail = 0u32;
    for s in series {
        if s.boot_id != id.boot_id
            || s.pid != id.pid
            || s.start_time_ticks != id.start_time_ticks
        {
            continue;
        }
        total += 1;
        if s.cpu_core_percent.is_some() {
            avail += 1;
        }
    }
    (avail, total)
}

fn alternative_processes(
    frames: &[TelemetryFrame],
    subject: &ProcessIdentity,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    subject_contrib: Option<f64>,
) -> Vec<AlternativeProcess> {
    let samples = collect_identity_samples(frames);
    let mut by_id: std::collections::HashMap<ProcessIdentity, Vec<f32>> =
        std::collections::HashMap::new();
    for (id, hit) in samples {
        if id == *subject {
            continue;
        }
        if hit.at < start || hit.at > end {
            continue;
        }
        if let Some(m) = hit.cpu_machine {
            by_id.entry(id).or_default().push(m);
        }
    }
    let mut alts: Vec<AlternativeProcess> = by_id
        .into_iter()
        .map(|(identity, vals)| {
            let mean = mean_f32(&vals).map(|v| v as f32);
            let share = match (subject_contrib, mean) {
                (Some(sub), Some(alt)) if sub > 0.01 => Some(alt as f64 / sub),
                _ => None,
            };
            AlternativeProcess {
                identity,
                cpu_machine_mean: mean,
                share_of_process_contribution: share,
            }
        })
        .collect();
    alts.sort_by(|a, b| {
        b.cpu_machine_mean
            .unwrap_or(0.0)
            .partial_cmp(&a.cpu_machine_mean.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    alts.truncate(5);
    alts
}

fn classify_confidence(
    config: &InvestigationConfig,
    global_delta: Option<f64>,
    process_contrib: Option<f64>,
    coverage: f64,
    samples_avail: u32,
    rationale: &mut Vec<String>,
) -> AttributionConfidence {
    let Some(delta) = global_delta else {
        rationale.push("Sin referencia de incremento global.".into());
        return AttributionConfidence::Insufficient;
    };
    let Some(proc_c) = process_contrib else {
        rationale.push("Sin muestras de CPU del proceso en la ventana.".into());
        return AttributionConfidence::Insufficient;
    };
    if delta.abs() < config.attribution_min_global_delta {
        rationale.push(format!(
            "Incremento global {delta:.1} pts < mínimo {} para alta confianza.",
            config.attribution_min_global_delta
        ));
        if proc_c >= config.burst_core_cpu_percent as f64 / 12.0 && coverage >= 0.35 {
            rationale.push("Correlación temporal con CPU del proceso.".into());
            return AttributionConfidence::Medium;
        }
        return AttributionConfidence::Insufficient;
    }
    let explains = proc_c / delta.max(0.01);
    if explains >= config.attribution_dominance_ratio
        && coverage >= config.attribution_min_coverage
        && samples_avail >= 3
    {
        rationale.push(format!(
            "Proceso ~{explains:.0}% del incremento global; cobertura {coverage:.0}%."
        ));
        AttributionConfidence::High
    } else if proc_c > 0.0 && coverage >= 0.25 {
        rationale.push(format!(
            "Contribución no dominante ({explains:.0}%) o cobertura {coverage:.0}%."
        ));
        AttributionConfidence::Medium
    } else {
        rationale.push("Evidencia contradictoria o incompleta.".into());
        AttributionConfidence::Insufficient
    }
}
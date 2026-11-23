use chrono::{DateTime, Utc};
use heimdall_blackbox::{SampleAvailability, TelemetryFrame};

use crate::config::InvestigationConfig;
use crate::types::ProcessIdentity;

#[derive(Debug, Clone)]
pub struct BurstHit {
    pub at: DateTime<Utc>,
    pub cpu_core: f32,
    pub cpu_machine: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct BurstStatus {
    pub identity: ProcessIdentity,
    pub hits: Vec<BurstHit>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub meets_threshold: bool,
}

pub fn collect_identity_samples(frames: &[TelemetryFrame]) -> Vec<(ProcessIdentity, BurstHit)> {
    let mut out = Vec::new();
    for frame in frames {
        for sample in &frame.processes {
            if sample.start_time_ticks == 0 || sample.boot_id.is_empty() {
                continue;
            }
            let core = match (sample.availability, sample.cpu_core_percent) {
                (SampleAvailability::Available, Some(v)) if v.is_finite() => v,
                _ => continue,
            };
            let identity = ProcessIdentity {
                boot_id: sample.boot_id.clone(),
                pid: sample.pid,
                start_time_ticks: sample.start_time_ticks,
                name: sample.name.clone(),
            };
            out.push((
                identity,
                BurstHit {
                    at: sample.observed_at,
                    cpu_core: core,
                    cpu_machine: sample.cpu_machine_percent.filter(|v| v.is_finite()),
                },
            ));
        }
    }
    out
}

pub fn burst_status_for_identity(
    identity: &ProcessIdentity,
    samples: &[(ProcessIdentity, BurstHit)],
    frames: &[TelemetryFrame],
    config: &InvestigationConfig,
    now: DateTime<Utc>,
) -> Option<BurstStatus> {
    if frames.is_empty() {
        return None;
    }
    let window_end = now;
    let window_start = window_end - chrono::Duration::seconds(config.burst_window_secs);
    let hits: Vec<BurstHit> = samples
        .iter()
        .filter(|(id, _)| id == identity)
        .filter(|(_, hit)| hit.at >= window_start && hit.at <= window_end)
        .filter(|(_, hit)| hit.cpu_core >= config.burst_core_cpu_percent)
        .map(|(_, hit)| hit.clone())
        .collect();
    let meets = hits.len() as u32 >= config.burst_min_hits;
    Some(BurstStatus {
        identity: identity.clone(),
        hits,
        window_start,
        window_end,
        meets_threshold: meets,
    })
}

pub fn identities_with_burst(
    frames: &[TelemetryFrame],
    config: &InvestigationConfig,
    now: DateTime<Utc>,
) -> Vec<BurstStatus> {
    let samples = collect_identity_samples(frames);
    let mut keys: Vec<ProcessIdentity> = Vec::new();
    for (id, _) in &samples {
        if !keys.iter().any(|k| k == id) {
            keys.push(id.clone());
        }
    }
    keys.into_iter()
        .filter_map(|id| burst_status_for_identity(&id, &samples, frames, config, now))
        .filter(|s| s.meets_threshold)
        .collect()
}

pub fn frame_global_cpu(frame: &TelemetryFrame) -> Option<f64> {
    frame.cpu_percent.filter(|v| v.is_finite())
}
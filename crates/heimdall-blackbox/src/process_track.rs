use std::collections::{HashSet, VecDeque};

use chrono::{DateTime, Utc};

use crate::types::{
    HistoricalProcessSample, ProcessExitEvent, SampleAvailability,
};

pub const MAX_TRACKED: usize = 24;

/// Proceso visto en la única pasada de este tick.
#[derive(Debug, Clone)]
pub struct LiveProcess {
    pub pid: u32,
    pub start_time_ticks: u64,
    pub name: String,
    pub cpu_core_percent: Option<f32>,
    pub cpu_machine_percent: Option<f32>,
    pub memory_bytes: u64,
    /// False si no se pudo leer `/proc/[pid]/stat`. No autoriza un evento de salida.
    pub identity_confirmed: bool,
    /// Está en el top acotado y debe entrar en el seguimiento si aún no estaba.
    pub follow: bool,
}

#[derive(Debug, Clone)]
struct Tracked {
    boot_id: String,
    pid: u32,
    start_time_ticks: u64,
    name: String,
    last_seen_at: DateTime<Utc>,
    cpu_core_percent: Option<f32>,
    cpu_machine_percent: Option<f32>,
    memory_bytes: Option<u64>,
}

pub struct TrackTick {
    pub samples: Vec<HistoricalProcessSample>,
    pub exits: Vec<ProcessExitEvent>,
}

pub struct ProcessTracker {
    tracked: VecDeque<Tracked>,
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTracker {
    pub fn new() -> Self {
        Self {
            tracked: VecDeque::new(),
        }
    }

    pub fn pids(&self) -> Vec<u32> {
        self.tracked.iter().map(|t| t.pid).collect()
    }

    pub fn observe(
        &mut self,
        now: DateTime<Utc>,
        boot_id: &str,
        present: &HashSet<u32>,
        live: &[LiveProcess],
    ) -> TrackTick {
        let mut exits = Vec::new();
        let mut samples = Vec::new();
        let mut kept: Vec<Tracked> = Vec::new();

        for tracked in self.tracked.drain(..) {
            if tracked.boot_id != boot_id || !present.contains(&tracked.pid) {
                exits.push(exit_event(&tracked, now));
                continue;
            }
            match live.iter().find(|p| p.pid == tracked.pid) {
                Some(p) if p.identity_confirmed && p.start_time_ticks != tracked.start_time_ticks => {
                    exits.push(exit_event(&tracked, now));
                }
                Some(p) if p.identity_confirmed => {
                    let updated = apply_live(&tracked, p, now);
                    samples.push(sample_from(&updated, now));
                    kept.push(updated);
                }
                _ => {
                    samples.push(unavailable_sample(&tracked, now));
                    kept.push(tracked);
                }
            }
        }

        for process in live.iter().filter(|p| p.follow && p.identity_confirmed && p.start_time_ticks > 0)
        {
            let already = kept.iter().any(|t| {
                t.boot_id == boot_id
                    && t.pid == process.pid
                    && t.start_time_ticks == process.start_time_ticks
            });
            if already {
                continue;
            }
            let created = Tracked {
                boot_id: boot_id.to_string(),
                pid: process.pid,
                start_time_ticks: process.start_time_ticks,
                name: process.name.clone(),
                last_seen_at: now,
                cpu_core_percent: finite_f32(process.cpu_core_percent),
                cpu_machine_percent: finite_f32(process.cpu_machine_percent),
                memory_bytes: Some(process.memory_bytes),
            };
            samples.push(sample_from(&created, now));
            kept.push(created);
        }

        if kept.len() > MAX_TRACKED {
            let follow_keys: HashSet<(u32, u64)> = live
                .iter()
                .filter(|p| p.follow && p.identity_confirmed)
                .map(|p| (p.pid, p.start_time_ticks))
                .collect();
            kept.sort_by_key(|t| {
                let followed = follow_keys.contains(&(t.pid, t.start_time_ticks));
                (!followed, std::cmp::Reverse(t.last_seen_at))
            });
            // Salir del cupo no es una finalización: el proceso sigue en la tabla.
            kept.truncate(MAX_TRACKED);
            let survivors: HashSet<(u32, u64)> = kept
                .iter()
                .map(|t| (t.pid, t.start_time_ticks))
                .collect();
            samples.retain(|sample| survivors.contains(&(sample.pid, sample.start_time_ticks)));
        }

        self.tracked = kept.into();
        TrackTick { samples, exits }
    }
}

fn finite_f32(value: Option<f32>) -> Option<f32> {
    value.filter(|v| v.is_finite())
}

fn apply_live(tracked: &Tracked, process: &LiveProcess, now: DateTime<Utc>) -> Tracked {
    Tracked {
        boot_id: tracked.boot_id.clone(),
        pid: tracked.pid,
        start_time_ticks: tracked.start_time_ticks,
        name: process.name.clone(),
        last_seen_at: now,
        cpu_core_percent: finite_f32(process.cpu_core_percent),
        cpu_machine_percent: finite_f32(process.cpu_machine_percent),
        memory_bytes: Some(process.memory_bytes),
    }
}

fn sample_from(tracked: &Tracked, now: DateTime<Utc>) -> HistoricalProcessSample {
    let available = tracked.cpu_core_percent.is_some();
    HistoricalProcessSample {
        boot_id: tracked.boot_id.clone(),
        pid: tracked.pid,
        start_time_ticks: tracked.start_time_ticks,
        name: tracked.name.clone(),
        observed_at: now,
        cpu_core_percent: tracked.cpu_core_percent,
        cpu_machine_percent: tracked.cpu_machine_percent,
        memory_bytes: tracked.memory_bytes,
        availability: if available {
            SampleAvailability::Available
        } else {
            SampleAvailability::Unavailable
        },
    }
}

fn unavailable_sample(tracked: &Tracked, now: DateTime<Utc>) -> HistoricalProcessSample {
    HistoricalProcessSample {
        boot_id: tracked.boot_id.clone(),
        pid: tracked.pid,
        start_time_ticks: tracked.start_time_ticks,
        name: tracked.name.clone(),
        observed_at: now,
        cpu_core_percent: None,
        cpu_machine_percent: None,
        memory_bytes: None,
        availability: SampleAvailability::Unavailable,
    }
}

fn exit_event(tracked: &Tracked, now: DateTime<Utc>) -> ProcessExitEvent {
    ProcessExitEvent {
        boot_id: tracked.boot_id.clone(),
        pid: tracked.pid,
        start_time_ticks: tracked.start_time_ticks,
        name: tracked.name.clone(),
        last_seen_at: tracked.last_seen_at,
        observed_absent_at: now,
        exit_observed_between_samples: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_700_000_000 + secs, 0).unwrap()
    }

    fn live(pid: u32, start: u64, cpu: Option<f32>, follow: bool) -> LiveProcess {
        LiveProcess {
            pid,
            start_time_ticks: start,
            name: format!("p{pid}"),
            cpu_core_percent: cpu,
            cpu_machine_percent: cpu.map(|v| v / 12.0),
            memory_bytes: 10,
            identity_confirmed: true,
            follow,
        }
    }

    #[test]
    fn leaving_the_top_is_not_an_exit() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        present.insert(7);
        tracker.observe(at(0), "boot", &present, &[live(7, 100, Some(80.0), true)]);
        let tick = tracker.observe(at(1), "boot", &present, &[live(7, 100, Some(1.0), false)]);
        assert!(tick.exits.is_empty());
        assert_eq!(tick.samples.len(), 1);
        assert_eq!(tick.samples[0].cpu_core_percent, Some(1.0));
        assert_eq!(tick.samples[0].availability, SampleAvailability::Available);
    }

    #[test]
    fn absence_from_the_process_table_is_an_exit_between_samples() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        present.insert(7);
        tracker.observe(at(0), "boot", &present, &[live(7, 100, Some(40.0), true)]);
        present.remove(&7);
        let tick = tracker.observe(at(2), "boot", &present, &[]);
        assert_eq!(tick.exits.len(), 1);
        assert_eq!(tick.exits[0].start_time_ticks, 100);
        assert_eq!(tick.exits[0].last_seen_at, at(0));
        assert_eq!(tick.exits[0].observed_absent_at, at(2));
        assert!(tick.exits[0].exit_observed_between_samples);
        assert!(tracker.pids().is_empty());
    }

    #[test]
    fn pid_reuse_with_a_different_start_time_closes_the_old_identity() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        present.insert(7);
        tracker.observe(at(0), "boot", &present, &[live(7, 100, Some(50.0), true)]);
        let tick = tracker.observe(at(1), "boot", &present, &[live(7, 250, Some(10.0), true)]);
        assert_eq!(tick.exits.len(), 1);
        assert_eq!(tick.exits[0].start_time_ticks, 100);
        assert!(tick
            .samples
            .iter()
            .any(|s| s.start_time_ticks == 250 && s.pid == 7));
    }

    #[test]
    fn unconfirmed_identity_does_not_emit_an_exit() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        present.insert(7);
        tracker.observe(at(0), "boot", &present, &[live(7, 100, Some(20.0), true)]);
        let unknown = LiveProcess {
            identity_confirmed: false,
            start_time_ticks: 0,
            follow: false,
            ..live(7, 0, None, false)
        };
        let tick = tracker.observe(at(1), "boot", &present, &[unknown]);
        assert!(tick.exits.is_empty());
        assert_eq!(tick.samples[0].availability, SampleAvailability::Unavailable);
        assert!(tick.samples[0].cpu_core_percent.is_none());
    }

    #[test]
    fn a_hot_process_displaces_a_full_tracker_without_an_exit() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        let mut initial = Vec::new();
        for pid in 1..=MAX_TRACKED as u32 {
            present.insert(pid);
            initial.push(live(pid, 10, Some(1.0), true));
        }
        tracker.observe(at(0), "boot", &present, &initial);
        present.insert(99);
        let mut next: Vec<_> = (1..=MAX_TRACKED as u32)
            .map(|pid| live(pid, 10, Some(1.0), false))
            .collect();
        next.push(live(99, 77, Some(100.0), true));
        let tick = tracker.observe(at(1), "boot", &present, &next);
        assert!(tick.exits.is_empty());
        assert!(tick.samples.iter().any(|s| s.pid == 99 && s.start_time_ticks == 77));
        assert!(tracker.pids().contains(&99));
        assert!(tracker.pids().len() <= MAX_TRACKED);
    }

    #[test]
    fn non_finite_cpu_is_unavailable_and_zero_is_kept() {
        let mut tracker = ProcessTracker::new();
        let mut present = HashSet::new();
        present.insert(3);
        let tick = tracker.observe(
            at(0),
            "boot",
            &present,
            &[live(3, 9, Some(f32::NAN), true)],
        );
        assert_eq!(tick.samples[0].availability, SampleAvailability::Unavailable);
        assert!(tick.samples[0].cpu_core_percent.is_none());
        let tick = tracker.observe(at(1), "boot", &present, &[live(3, 9, Some(0.0), true)]);
        assert_eq!(tick.samples[0].cpu_core_percent, Some(0.0));
        assert_eq!(tick.samples[0].availability, SampleAvailability::Available);
    }
}

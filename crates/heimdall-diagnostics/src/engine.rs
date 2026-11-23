use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use heimdall_core::{
    Diagnosis, DiagnosisLifecycleState, HardwareSummary, ProcessFootprint, TelemetrySnapshot,
};

use crate::config::DiagnosticConfig;
use crate::journal::{enrich_diagnoses, JournalHint};
use crate::rules::{
    evaluate_cpu_saturation, evaluate_cpu_thermal, evaluate_gpu_driver_journal,
    evaluate_gpu_thermal, evaluate_gpu_vram_pressure, evaluate_io_contention,
    evaluate_memory_pressure, evaluate_storage_pressure,
};

struct RuleCooldown {
    last_fired: DateTime<Utc>,
}

struct RuleTrack {
    first_seen: DateTime<Utc>,
    consecutive: u32,
}

pub struct DiagnosticEngine {
    config: DiagnosticConfig,
    logical_cores: u32,
    history: VecDeque<TelemetrySnapshot>,
    cooldowns: HashMap<String, RuleCooldown>,
    tracks: HashMap<String, RuleTrack>,
    recently_resolved: HashMap<String, (Diagnosis, DateTime<Utc>)>,
    dismissed: std::collections::HashSet<String>,
    active: Vec<Diagnosis>,
}

impl DiagnosticEngine {
    pub fn new(hardware: &HardwareSummary, config: DiagnosticConfig) -> Self {
        Self {
            config: config.clone(),
            logical_cores: hardware.cpu.logical_cores.max(1),
            history: VecDeque::with_capacity(config.history_capacity),
            cooldowns: HashMap::new(),
            tracks: HashMap::new(),
            recently_resolved: HashMap::new(),
            dismissed: std::collections::HashSet::new(),
            active: Vec::new(),
        }
    }

    pub fn dismiss_rule(&mut self, rule_id: &str) {
        self.dismissed.insert(rule_id.to_string());
        self.active.retain(|d| d.rule_id != rule_id);
        self.tracks.remove(rule_id);
    }

    pub fn active_diagnoses(&self) -> &[Diagnosis] {
        &self.active
    }

    pub fn set_config(&mut self, config: DiagnosticConfig) {
        self.config = config;
    }

    pub fn ingest(&mut self, snapshot: TelemetrySnapshot) {
        self.history.push_back(snapshot);
        while self.history.len() > self.config.history_capacity {
            self.history.pop_front();
        }
    }

    pub fn evaluate(
        &mut self,
        by_cpu: &[ProcessFootprint],
        by_memory: &[ProcessFootprint],
        journal: &[JournalHint],
    ) -> Vec<Diagnosis> {
        let current = match self.history.back() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        };
        let history: Vec<_> = self.history.iter().cloned().collect();
        let now = Utc::now();

        let mut candidates = Vec::new();
        if let Some(mut d) = evaluate_memory_pressure(&self.config, &history, &current, by_memory) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_cpu_saturation(
            &self.config,
            &history,
            &current,
            self.logical_cores,
            by_cpu,
        ) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_storage_pressure(&self.config, &current) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_gpu_vram_pressure(&self.config, &history, &current) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_io_contention(&self.config, &history, &current) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_cpu_thermal(&self.config, &history, &current) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_gpu_thermal(&self.config, &history, &current) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }
        if let Some(mut d) = evaluate_gpu_driver_journal(journal) {
            stamp_lifecycle(&mut d, now, &self.config, &mut self.tracks);
            candidates.push(d);
        }

        enrich_diagnoses(&mut candidates, journal);

        let fired: std::collections::HashSet<_> =
            candidates.iter().map(|c| c.rule_id.as_str()).collect();
        for rule_id in &fired {
            self.recently_resolved.remove(*rule_id);
        }
        self.tracks.retain(|rule_id, _| fired.contains(rule_id.as_str()));

        let mut next_active = Vec::new();
        for d in candidates {
            if self.dismissed.contains(&d.rule_id) {
                continue;
            }
            if let Some(cd) = self.cooldowns.get(&d.rule_id) {
                let within = (now - cd.last_fired).num_seconds() < self.config.rule_cooldown_secs;
                if within {
                    if let Some(prev) = self.active.iter().find(|a| a.rule_id == d.rule_id) {
                        next_active.push(prev.clone());
                    }
                    continue;
                }
            }
            self.cooldowns
                .insert(d.rule_id.clone(), RuleCooldown { last_fired: now });
            next_active.push(d);
        }

        let still_active: std::collections::HashSet<String> =
            next_active.iter().map(|d| d.rule_id.clone()).collect();
        for prev in &self.active {
            if still_active.contains(&prev.rule_id) {
                continue;
            }
            if prev.lifecycle_state == DiagnosisLifecycleState::Resolved {
                continue;
            }
            let mut resolved = prev.clone();
            resolved.lifecycle_state = DiagnosisLifecycleState::Resolved;
            resolved.observed_at = now;
            self.recently_resolved
                .insert(prev.rule_id.clone(), (resolved, now));
        }

        self.recently_resolved.retain(|rule_id, (_, at)| {
            if still_active.contains(rule_id) {
                return false;
            }
            (now - *at).num_seconds() < self.config.resolved_display_secs
        });

        for (resolved, _) in self.recently_resolved.values() {
            if !still_active.contains(&resolved.rule_id) {
                next_active.push(resolved.clone());
            }
        }

        self.active = next_active;
        self.active.clone()
    }

    pub fn step(
        &mut self,
        snapshot: TelemetrySnapshot,
        by_cpu: &[ProcessFootprint],
        by_memory: &[ProcessFootprint],
        journal: &[JournalHint],
    ) -> Vec<Diagnosis> {
        self.ingest(snapshot);
        self.evaluate(by_cpu, by_memory, journal)
    }
}

fn stamp_lifecycle(
    d: &mut Diagnosis,
    now: DateTime<Utc>,
    config: &DiagnosticConfig,
    tracks: &mut HashMap<String, RuleTrack>,
) {
    let track = tracks.entry(d.rule_id.clone()).or_insert(RuleTrack {
        first_seen: now,
        consecutive: 0,
    });
    track.consecutive += 1;
    d.first_detected_at = track.first_seen;
    d.last_seen_at = now;
    d.observed_at = now;
    d.active_seconds = (now - track.first_seen).num_seconds().max(0) as u64;
    d.lifecycle_state = if track.consecutive >= config.persistent_samples {
        DiagnosisLifecycleState::Persistent
    } else {
        DiagnosisLifecycleState::Detected
    };
}
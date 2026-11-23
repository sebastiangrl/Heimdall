use std::collections::HashMap;

use chrono::{DateTime, Utc};
use heimdall_blackbox::{HistoricalProcessSample, ProcessExitEvent, TelemetryFrame};
use uuid::Uuid;

use crate::burst::{burst_status_for_identity, collect_identity_samples, identities_with_burst};
use crate::config::InvestigationConfig;
use crate::evidence::{attach_exit, build_attribution};
use crate::impact::{assess_impact, classify_level};
use crate::types::{
    AttributionConfidence, ImpactSeverity, InvestigationDossier, InvestigationLevel,
    InvestigationState, InvestigationSummary, ProcessIdentity, INVESTIGATION_SCHEMA_VERSION,
};

pub const RULE_INTERMITTENT_BURST: &str = "intermittent_cpu_burst";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct IdentityKey {
    boot_id: String,
    pid: u32,
    start_time_ticks: u64,
}

impl From<&ProcessIdentity> for IdentityKey {
    fn from(id: &ProcessIdentity) -> Self {
        Self {
            boot_id: id.boot_id.clone(),
            pid: id.pid,
            start_time_ticks: id.start_time_ticks,
        }
    }
}

struct OpenInvestigation {
    dossier: InvestigationDossier,
    last_active: DateTime<Utc>,
}

pub struct InvestigationEngine {
    config: InvestigationConfig,
    open: HashMap<IdentityKey, OpenInvestigation>,
    cooldown_until: HashMap<IdentityKey, DateTime<Utc>>,
}

impl InvestigationEngine {
    pub fn new(config: InvestigationConfig) -> Self {
        Self {
            config,
            open: HashMap::new(),
            cooldown_until: HashMap::new(),
        }
    }

    pub fn config(&self) -> &InvestigationConfig {
        &self.config
    }

    pub fn open_count(&self) -> usize {
        self.open.len()
    }

    pub fn restore_open(&mut self, dossier: InvestigationDossier) {
        let key = IdentityKey::from(&dossier.identity);
        self.open.insert(
            key,
            OpenInvestigation {
                dossier,
                last_active: Utc::now(),
            },
        );
    }

    /// Un tick del sampler. Devuelve expedientes que deben persistirse y cierres con post-captura.
    pub fn step(
        &mut self,
        frames: &[TelemetryFrame],
        activity_profile: &str,
        exits: &[ProcessExitEvent],
        now: DateTime<Utc>,
    ) -> InvestigationStepResult {
        let mut persist: Vec<InvestigationDossier> = Vec::new();
        let mut closed_with_post: Vec<(Uuid, usize)> = Vec::new();

        self.cooldown_until
            .retain(|_, until| *until > now);

        let active_bursts = identities_with_burst(frames, &self.config, now);
        let active_keys: std::collections::HashSet<_> = active_bursts
            .iter()
            .map(|s| IdentityKey::from(&s.identity))
            .collect();

        for status in &active_bursts {
            let key = IdentityKey::from(&status.identity);
            if self.cooldown_until.contains_key(&key) {
                continue;
            }
            if self.open.contains_key(&key) {
                let dossier = self.refresh_open_entry(
                    &key,
                    status,
                    frames,
                    activity_profile,
                    exits,
                    now,
                );
                persist.push(dossier);
                continue;
            }
            let dossier = self.open_new(status, frames, activity_profile);
            self.open.insert(
                key,
                OpenInvestigation {
                    dossier: dossier.clone(),
                    last_active: now,
                },
            );
            persist.push(dossier);
        }

        let samples = collect_identity_samples(frames);
        let keys_to_close: Vec<IdentityKey> = self
            .open
            .keys()
            .filter(|k| !active_keys.contains(k))
            .cloned()
            .collect();

        for key in keys_to_close {
            let Some(open) = self.open.get(&key) else {
                continue;
            };
            let identity = open.dossier.identity.clone();
            let still_bursting = burst_status_for_identity(
                &identity,
                &samples,
                frames,
                &self.config,
                now,
            )
            .map(|s| s.meets_threshold)
            .unwrap_or(false);
            if still_bursting {
                continue;
            }
            let inactive_secs = (now - open.last_active).num_seconds();
            if inactive_secs < self.config.burst_quiet_secs {
                continue;
            }
            let mut closed = self.open.remove(&key).unwrap().dossier;
            closed.state = InvestigationState::Closed;
            closed.updated_at = now;
            closed.explanation = finalize_explanation(&closed);
            let post = self.config.post_capture_frames;
            closed_with_post.push((closed.id, post));
            persist.push(closed);
            self.cooldown_until.insert(
                key,
                now + chrono::Duration::seconds(self.config.identity_cooldown_secs),
            );
        }

        self.apply_exits(exits);

        InvestigationStepResult {
            persist,
            closed_with_post,
        }
    }

    fn refresh_open_entry(
        &mut self,
        key: &IdentityKey,
        status: &crate::burst::BurstStatus,
        frames: &[TelemetryFrame],
        activity_profile: &str,
        exits: &[ProcessExitEvent],
        now: DateTime<Utc>,
    ) -> InvestigationDossier {
        let open = self.open.get_mut(key).expect("open entry");
        open.last_active = now;
        let series = merge_series(&open.dossier.process_series, frames, &status.identity);
        let mut attribution = build_attribution(status, frames, &series, &self.config);
        if let Some(exit) = exits.iter().find(|e| {
            e.boot_id == status.identity.boot_id
                && e.pid == status.identity.pid
                && e.start_time_ticks == status.identity.start_time_ticks
        }) {
            attach_exit(&mut attribution, exit.clone());
        }
        let impact = assess_impact(status, frames, activity_profile, &self.config);
        let level = classify_level(&impact, attribution.global_cpu_delta);
        open.dossier.updated_at = now;
        open.dossier.attribution = attribution;
        open.dossier.impact = impact;
        open.dossier.level = level;
        open.dossier.process_series = series;
        open.dossier.explanation = explain_dossier(&open.dossier);
        open.dossier.clone()
    }

    fn open_new(
        &self,
        status: &crate::burst::BurstStatus,
        frames: &[TelemetryFrame],
        activity_profile: &str,
    ) -> InvestigationDossier {
        let now = Utc::now();
        let series = merge_series(&[], frames, &status.identity);
        let attribution = build_attribution(status, frames, &series, &self.config);
        let impact = assess_impact(status, frames, activity_profile, &self.config);
        let level = classify_level(&impact, attribution.global_cpu_delta);
        let mut dossier = InvestigationDossier {
            schema_version: INVESTIGATION_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            state: InvestigationState::Open,
            level,
            rule_id: RULE_INTERMITTENT_BURST.into(),
            identity: status.identity.clone(),
            attribution,
            impact,
            explanation: String::new(),
            process_series: series,
            context_frames: clip_frames(frames, status.window_start),
            post_frames: vec![],
            limitations: vec![
                "Detección por ráfagas intermitentes; no implica malicia.".into(),
                "La atribución no usa Ollama ni nombres conocidos de antemano.".into(),
            ],
        };
        dossier.explanation = explain_dossier(&dossier);
        dossier
    }

    fn apply_exits(&mut self, exits: &[ProcessExitEvent]) {
        for exit in exits {
            let key = IdentityKey {
                boot_id: exit.boot_id.clone(),
                pid: exit.pid,
                start_time_ticks: exit.start_time_ticks,
            };
            if let Some(open) = self.open.get_mut(&key) {
                attach_exit(&mut open.dossier.attribution, exit.clone());
                open.dossier.updated_at = Utc::now();
            }
        }
    }

    pub fn summaries_from_dossiers(dossiers: &[InvestigationDossier]) -> Vec<InvestigationSummary> {
        dossiers
            .iter()
            .map(|d| InvestigationSummary {
                id: d.id,
                created_at: d.created_at,
                updated_at: d.updated_at,
                state: d.state,
                level: d.level,
                identity: d.identity.clone(),
                attribution_confidence: d.attribution.confidence,
                impact_severity: d.impact.severity,
                incomplete: d.state == InvestigationState::Incomplete,
            })
            .collect()
    }
}

pub struct InvestigationStepResult {
    pub persist: Vec<InvestigationDossier>,
    pub closed_with_post: Vec<(Uuid, usize)>,
}

fn clip_frames(frames: &[TelemetryFrame], from: DateTime<Utc>) -> Vec<TelemetryFrame> {
    frames
        .iter()
        .filter(|f| f.observed_at >= from)
        .take(40)
        .cloned()
        .collect()
}

fn merge_series(
    existing: &[HistoricalProcessSample],
    frames: &[TelemetryFrame],
    identity: &ProcessIdentity,
) -> Vec<HistoricalProcessSample> {
    let mut out = existing.to_vec();
    for frame in frames {
        for sample in &frame.processes {
            if sample.boot_id == identity.boot_id
                && sample.pid == identity.pid
                && sample.start_time_ticks == identity.start_time_ticks
            {
                if out.iter().any(|s| s.observed_at == sample.observed_at) {
                    continue;
                }
                out.push(sample.clone());
            }
        }
    }
    out.sort_by_key(|s| s.observed_at);
    out
}

fn explain_dossier(d: &InvestigationDossier) -> String {
    let level = match d.level {
        InvestigationLevel::Event => "evento",
        InvestigationLevel::Anomaly => "anomalía",
        InvestigationLevel::Incident => "incidente (impacto)",
    };
    let attr = match d.attribution.confidence {
        AttributionConfidence::High => "alta",
        AttributionConfidence::Medium => "media",
        AttributionConfidence::Insufficient => "insuficiente",
    };
    let impact = match d.impact.severity {
        ImpactSeverity::None => "sin degradación confirmada",
        ImpactSeverity::Minor => "impacto menor",
        ImpactSeverity::Major => "impacto mayor",
    };
    format!(
        "Investigación {level}: proceso {} (pid {}, starttime {}). Atribución {attr}. {impact}.",
        d.identity.name, d.identity.pid, d.identity.start_time_ticks
    )
}

fn finalize_explanation(d: &InvestigationDossier) -> String {
    let mut base = explain_dossier(d);
    if let Some(exit) = &d.attribution.exit {
        if exit.exit_observed_between_samples {
            base.push_str(" Salida observada entre dos muestras.");
        }
    }
    base
}
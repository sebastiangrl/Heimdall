use chrono::{DateTime, Utc};
use heimdall_core::{ProcessFootprint, TelemetrySnapshot};
use serde::{Deserialize, Serialize};

use crate::baseline::{BaselineTracker, ProfileBaselineSummary};
use crate::baseline_store::BaselineStore;
use crate::classifier::{classify, ActivityAssessment, ActivityProfile};
use crate::store::ProfileStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileState {
    pub profile: ActivityProfile,
    pub confidence: f32,
    pub assessment: ActivityAssessment,
    pub session_started_at: DateTime<Utc>,
    pub profile_since: DateTime<Utc>,
    pub learning: bool,
    pub session_seconds: u64,
    pub baseline: ProfileBaselineSummary,
}

pub struct ProfileEngine {
    store: ProfileStore,
    active: ActivityProfile,
    active_since: DateTime<Utc>,
    session_started: DateTime<Utc>,
    candidate: ActivityProfile,
    candidate_streak: u32,
    last: ActivityAssessment,
    stable_samples: u32,
    baseline: BaselineTracker,
    baseline_store: BaselineStore,
    samples_since_baseline_save: u32,
}

impl ProfileEngine {
    pub fn new() -> Self {
        let store = ProfileStore::open_default().unwrap_or_else(|_| {
            ProfileStore::open(std::path::PathBuf::from("./heimdall-profiles.json"))
                .expect("profile store fallback")
        });
        let now = Utc::now();
        let baseline_store = BaselineStore::open_default().unwrap_or_else(|_| {
            BaselineStore::open(std::path::PathBuf::from("./heimdall-baseline.json"))
                .expect("baseline store fallback")
        });
        let mut baseline = BaselineTracker::new();
        baseline_store.load_into(&mut baseline);
        Self {
            store,
            active: ActivityProfile::General,
            active_since: now,
            session_started: now,
            candidate: ActivityProfile::General,
            candidate_streak: 0,
            last: ActivityAssessment {
                profile: ActivityProfile::General,
                confidence: 0.25,
                signals: vec![],
                dominant_apps: vec![],
            },
            stable_samples: 6,
            baseline,
            baseline_store,
            samples_since_baseline_save: 0,
        }
    }

    pub fn update(
        &mut self,
        snap: &TelemetrySnapshot,
        processes: &[ProcessFootprint],
    ) -> ProfileState {
        let assessment = classify(snap, processes, self.store.overrides());
        self.last = assessment.clone();

        if assessment.profile == self.candidate {
            self.candidate_streak += 1;
        } else {
            self.candidate = assessment.profile;
            self.candidate_streak = 1;
        }

        if self.candidate_streak >= self.stable_samples && self.candidate != self.active {
            self.active = self.candidate;
            self.active_since = Utc::now();
        }

        self.baseline.record(self.active, snap);
        self.samples_since_baseline_save += 1;
        if self.samples_since_baseline_save >= 60 {
            self.samples_since_baseline_save = 0;
            let _ = self.baseline_store.save_from(&self.baseline);
        }

        let now = Utc::now();
        let baseline = self.baseline.compare_current(self.active, snap);
        ProfileState {
            profile: self.active,
            confidence: assessment.confidence,
            assessment,
            session_started_at: self.session_started,
            profile_since: self.active_since,
            learning: baseline.learning,
            session_seconds: (now - self.session_started).num_seconds().max(0) as u64,
            baseline,
        }
    }

    pub fn state(&self) -> ProfileState {
        let now = Utc::now();
        let baseline = self.baseline.summary(self.active);
        ProfileState {
            profile: self.active,
            confidence: self.last.confidence,
            assessment: self.last.clone(),
            session_started_at: self.session_started,
            profile_since: self.active_since,
            learning: baseline.learning,
            session_seconds: (now - self.session_started).num_seconds().max(0) as u64,
            baseline,
        }
    }

    pub fn store_mut(&mut self) -> &mut ProfileStore {
        &mut self.store
    }

    pub fn overrides(&self) -> &[crate::store::ProcessProfileOverride] {
        self.store.overrides()
    }
}

impl Default for ProfileEngine {
    fn default() -> Self {
        Self::new()
    }
}
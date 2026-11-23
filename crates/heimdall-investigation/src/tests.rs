use chrono::{TimeZone, Utc};
use heimdall_blackbox::{HistoricalProcessSample, ProcessExitEvent, SampleAvailability, TelemetryFrame};
use tempfile::tempdir;
use uuid::Uuid;

use crate::{
    burst::identities_with_burst, InvestigationConfig, InvestigationEngine, InvestigationLevel,
    InvestigationState, InvestigationStore, ProcessIdentity,
};

fn t(secs: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_800_000_000 + secs, 0).unwrap()
}

fn frame(at: i64, cpu: f64, samples: Vec<HistoricalProcessSample>) -> TelemetryFrame {
    TelemetryFrame {
        observed_at: t(at),
        cpu_percent: Some(cpu),
        mem_available_bytes: Some(8_000_000_000),
        swap_used_bytes: Some(0),
        gpu_percent: None,
        vram_used_bytes: None,
        cpu_temp_c: None,
        gpu_temp_c: None,
        psi_mem: Some(1.0),
        psi_cpu: Some(2.0),
        psi_io: Some(1.0),
        diagnosis_rules: vec![],
        top_processes: vec![],
        schema_version: 2,
        boot_id: "boot-test".into(),
        processes: samples,
        process_exits: vec![],
    }
}

fn proc_sample(pid: u32, start: u64, at: i64, core: f32) -> HistoricalProcessSample {
    HistoricalProcessSample {
        boot_id: "boot-test".into(),
        pid,
        start_time_ticks: start,
        name: format!("worker{pid}"),
        observed_at: t(at),
        cpu_core_percent: Some(core),
        cpu_machine_percent: Some(core / 12.0),
        memory_bytes: Some(10_000_000),
        availability: SampleAvailability::Available,
    }
}

#[test]
fn burst_detects_intermittent_activity_with_gaps() {
    let mut frames = Vec::new();
    let pid = 4242u32;
    let start = 99_001u64;
    for (at, core) in [(0, 5.0), (3, 80.0), (6, 75.0), (9, 82.0), (12, 78.0), (15, 2.0)] {
        frames.push(frame(
            at,
            20.0,
            vec![proc_sample(pid, start, at, core)],
        ));
    }
    let cfg = InvestigationConfig {
        burst_window_secs: 20,
        burst_min_hits: 4,
        burst_core_cpu_percent: 70.0,
        ..InvestigationConfig::default()
    };
    let bursts = identities_with_burst(&frames, &cfg, t(15));
    assert_eq!(bursts.len(), 1);
    assert!(bursts[0].hits.len() >= 4);
}

#[test]
fn sleeping_process_does_not_burst() {
    let frames = (0..8)
        .map(|i| frame(i * 2, 10.0, vec![proc_sample(1, 10, i * 2, 1.0)]))
        .collect::<Vec<_>>();
    let cfg = InvestigationConfig::default();
    assert!(identities_with_burst(&frames, &cfg, t(20)).is_empty());
}

#[test]
fn two_simultaneous_processes_get_separate_bursts() {
    let mut frames = Vec::new();
    for at in [2, 5, 8, 11] {
        frames.push(frame(
            at,
            40.0,
            vec![
                proc_sample(10, 100, at, 85.0),
                proc_sample(11, 200, at, 90.0),
            ],
        ));
    }
    let cfg = InvestigationConfig::default();
    let bursts = identities_with_burst(&frames, &cfg, t(12));
    assert_eq!(bursts.len(), 2);
}

#[test]
fn exit_after_leaving_top_is_recorded() {
    let mut engine = InvestigationEngine::new(InvestigationConfig {
        burst_min_hits: 2,
        burst_quiet_secs: 1,
        ..InvestigationConfig::default()
    });
    let pid = 77;
    let start = 555u64;
    let mut frames = vec![
        frame(0, 30.0, vec![proc_sample(pid, start, 0, 90.0)]),
        frame(1, 30.0, vec![proc_sample(pid, start, 1, 95.0)]),
    ];
    let _ = engine.step(&frames, "general", &[], t(1));
    let exit = ProcessExitEvent {
        boot_id: "boot-test".into(),
        pid,
        start_time_ticks: start,
        name: format!("worker{pid}"),
        last_seen_at: t(1),
        observed_absent_at: t(2),
        exit_observed_between_samples: true,
    };
    frames.push(frame(3, 10.0, vec![]));
    let result = engine.step(&frames, "general", &[exit], t(4));
    assert!(result
        .persist
        .iter()
        .any(|d| d.attribution.exit.is_some()));
}

#[test]
fn pid_reuse_is_different_identity() {
    let mut engine = InvestigationEngine::new(InvestigationConfig {
        burst_min_hits: 2,
        ..Default::default()
    });
    let frames_a = vec![
        frame(0, 20.0, vec![proc_sample(5, 100, 0, 90.0)]),
        frame(1, 20.0, vec![proc_sample(5, 100, 1, 90.0)]),
    ];
    let _ = engine.step(&frames_a, "general", &[], t(1));
    let frames_b = vec![
        frame(2, 20.0, vec![proc_sample(5, 999, 2, 90.0)]),
        frame(3, 20.0, vec![proc_sample(5, 999, 3, 90.0)]),
    ];
    let r = engine.step(&frames_b, "general", &[], t(3));
    assert!(!r.persist.is_empty());
    assert_eq!(r.persist.last().unwrap().identity.start_time_ticks, 999);
}

#[test]
fn unavailable_samples_reduce_coverage_not_invent_confidence() {
    let samples = vec![
        HistoricalProcessSample {
            availability: SampleAvailability::Unavailable,
            cpu_core_percent: None,
            ..proc_sample(1, 1, 0, 0.0)
        };
        4
    ];
    let frames = vec![frame(0, 10.0, samples)];
    let mut engine = InvestigationEngine::new(InvestigationConfig {
        burst_min_hits: 1,
        burst_core_cpu_percent: 10.0,
        ..Default::default()
    });
    let r = engine.step(&frames, "general", &[], t(0));
    if let Some(d) = r.persist.first() {
        assert!(d.attribution.sample_coverage < 0.5);
    }
}

#[test]
fn store_survives_restart_and_ring_loss() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("bb.db");
    let id = {
        let store = InvestigationStore::open(db.clone()).unwrap();
        let dossier = crate::types::InvestigationDossier {
            schema_version: crate::INVESTIGATION_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            state: InvestigationState::Closed,
            level: InvestigationLevel::Event,
            rule_id: "intermittent_cpu_burst".into(),
            identity: ProcessIdentity {
                boot_id: "b".into(),
                pid: 1,
                start_time_ticks: 2,
                name: "x".into(),
            },
            attribution: crate::evidence::build_attribution(
                &identities_with_burst(
                    &[frame(0, 10.0, vec![proc_sample(1, 2, 0, 80.0)])],
                    &InvestigationConfig {
                        burst_min_hits: 1,
                        ..Default::default()
                    },
                    t(0),
                )[0],
                &[frame(0, 10.0, vec![proc_sample(1, 2, 0, 80.0)])],
                &[],
                &InvestigationConfig::default(),
            ),
            impact: crate::impact::assess_impact(
                &identities_with_burst(
                    &[frame(0, 10.0, vec![proc_sample(1, 2, 0, 80.0)])],
                    &InvestigationConfig {
                        burst_min_hits: 1,
                        ..Default::default()
                    },
                    t(0),
                )[0],
                &[frame(0, 10.0, vec![proc_sample(1, 2, 0, 80.0)])],
                "general",
                &InvestigationConfig::default(),
            ),
            explanation: "test".into(),
            process_series: vec![proc_sample(1, 2, 0, 80.0)],
            context_frames: vec![],
            post_frames: vec![],
            limitations: vec![],
        };
        let id = dossier.id;
        store.upsert(&dossier).unwrap();
        id
    };
    let store2 = InvestigationStore::open(db).unwrap();
    let loaded = store2.get(id).unwrap().expect("loaded");
    assert_eq!(loaded.id, id);
    assert_eq!(loaded.process_series.len(), 1);
}

#[test]
fn repeat_event_after_cooldown_allowed() {
    let mut engine = InvestigationEngine::new(InvestigationConfig {
        burst_min_hits: 2,
        burst_quiet_secs: 0,
        identity_cooldown_secs: 1,
        ..Default::default()
    });
    let pid = 9;
    let start = 1u64;
    let burst = |at| {
        vec![
            frame(at, 50.0, vec![proc_sample(pid, start, at, 90.0)]),
            frame(at + 1, 50.0, vec![proc_sample(pid, start, at + 1, 90.0)]),
        ]
    };
    let _ = engine.step(&burst(0), "general", &[], t(1));
    let close = engine.step(&[frame(5, 10.0, vec![])], "general", &[], t(6));
    assert!(!close.closed_with_post.is_empty());
    let _ = engine.step(&burst(7), "general", &[], t(8));
    let later = engine.step(&burst(25), "general", &[], t(30));
    assert!(!later.persist.is_empty());
}
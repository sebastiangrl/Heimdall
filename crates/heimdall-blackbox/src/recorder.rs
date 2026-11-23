use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

use chrono::Utc;
use heimdall_core::{Diagnosis, DiagnosisLifecycleState, DiagnosisSeverity, ProcessFootprint, TelemetrySnapshot};
use uuid::Uuid;

use crate::config::BlackBoxConfig;
use crate::db::BlackBoxDb;
use crate::process_track::{LiveProcess, ProcessTracker};
use crate::types::{
    BlackBoxStatus, IncidentRecord, IncidentSeverity, IncidentSummary, ProcessExitEvent,
    ProcessReconstruction, TelemetryFrame, FRAME_SCHEMA_VERSION,
};

struct PostCapture {
    incident_id: Uuid,
    remaining: usize,
}

pub struct BlackBoxRecorder {
    session_id: Uuid,
    ring: VecDeque<TelemetryFrame>,
    ring_capacity: usize,
    db: BlackBoxDb,
    db_path: PathBuf,
    last_checkpoint: Instant,
    last_incident_rules: std::collections::HashSet<String>,
    post_captures: Vec<PostCapture>,
    max_concurrent_post: usize,
    post_capture_frames: usize,
    checkpoint_secs: u64,
    max_incidents: u32,
    tracker: ProcessTracker,
}

impl BlackBoxRecorder {
    pub fn open_default() -> Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall");
        let db_path = data_dir.join("blackbox.db");
        Self::open(db_path)
    }

    pub fn open(db_path: PathBuf) -> Result<Self, String> {
        Self::open_with_config(db_path, BlackBoxConfig::from_env())
    }

    pub fn open_with_config(db_path: PathBuf, config: BlackBoxConfig) -> Result<Self, String> {
        let db = BlackBoxDb::open(&db_path).map_err(|e| e.to_string())?;
        let cap = config.ring_capacity;
        let mut ring = VecDeque::with_capacity(cap);
        if let Ok(Some(json)) = db.load_latest_ring_checkpoint() {
            if let Ok(frames) = serde_json::from_str::<Vec<TelemetryFrame>>(&json) {
                for f in frames.into_iter().rev().take(cap) {
                    ring.push_front(f);
                }
            }
        }
        Ok(Self {
            session_id: Uuid::new_v4(),
            ring,
            ring_capacity: cap,
            db,
            db_path,
            last_checkpoint: Instant::now(),
            last_incident_rules: std::collections::HashSet::new(),
            post_captures: Vec::new(),
            max_concurrent_post: 6,
            post_capture_frames: config.post_capture_frames,
            checkpoint_secs: config.checkpoint_secs,
            max_incidents: config.max_incidents,
            tracker: ProcessTracker::new(),
        })
    }

    pub fn status(&self) -> BlackBoxStatus {
        let incidents = self.db.count_incidents().unwrap_or(0);
        BlackBoxStatus {
            recording: true,
            frames_in_memory: self.ring.len(),
            incidents_stored: incidents,
            last_checkpoint_at: Some(Utc::now()),
            db_path: self.db_path.display().to_string(),
            session_id: self.session_id,
        }
    }

    pub fn list_incidents(&self, limit: usize) -> Vec<IncidentRecord> {
        self.db.list_incidents(limit).unwrap_or_default()
    }

    pub fn list_incident_summaries(&self, limit: usize) -> Vec<IncidentSummary> {
        self.list_incidents(limit)
            .into_iter()
            .map(|r| IncidentSummary {
                id: r.id,
                detected_at: r.detected_at,
                severity: r.severity,
                title_key: r.title_key,
                rule_id: r.rule_id,
                incomplete: r.incomplete,
                pre_samples: r.pre_window.len(),
                post_samples: r.post_window.len(),
            })
            .collect()
    }

    pub fn export_technical(&self, id: Uuid) -> Result<String, String> {
        let record = self
            .get_incident(id)
            .ok_or_else(|| "incidente no encontrado".to_string())?;
        crate::export::export_incident_technical_json(&record)
    }

    pub fn export_human(&self, id: Uuid) -> Result<String, String> {
        let record = self
            .get_incident(id)
            .ok_or_else(|| "incidente no encontrado".to_string())?;
        crate::export::export_incident_human_json(&record)
    }

    pub fn get_incident(&self, id: Uuid) -> Option<IncidentRecord> {
        self.db.get_incident(id).ok().flatten()
    }

    pub fn tracked_pids(&self) -> Vec<u32> {
        self.tracker.pids()
    }

    pub fn recent_exits(&self, limit: usize) -> Vec<ProcessExitEvent> {
        let mut out = Vec::new();
        for frame in self.ring.iter().rev() {
            for event in frame.process_exits.iter().rev() {
                out.push(event.clone());
                if out.len() >= limit {
                    return out;
                }
            }
        }
        out
    }

    pub fn reconstruct(
        &self,
        boot_id: &str,
        pid: u32,
        start_time_ticks: u64,
    ) -> ProcessReconstruction {
        let mut samples = Vec::new();
        let mut exit = None;
        let mut name = String::new();
        let mut legacy = false;
        for frame in &self.ring {
            for sample in &frame.processes {
                if sample.pid == pid
                    && sample.boot_id == boot_id
                    && sample.start_time_ticks == start_time_ticks
                {
                    name = sample.name.clone();
                    samples.push(sample.clone());
                }
            }
            for event in &frame.process_exits {
                if event.pid == pid
                    && event.boot_id == boot_id
                    && event.start_time_ticks == start_time_ticks
                {
                    name = event.name.clone();
                    exit = Some(event.clone());
                }
            }
            if samples.is_empty() && start_time_ticks == 0 && boot_id.is_empty() {
                for label in &frame.top_processes {
                    if let Some((legacy_name, legacy_pid)) = label.rsplit_once(':') {
                        if legacy_pid.parse::<u32>().ok() == Some(pid) {
                            legacy = true;
                            name = legacy_name.to_string();
                        }
                    }
                }
            }
        }
        ProcessReconstruction {
            boot_id: boot_id.to_string(),
            pid,
            start_time_ticks,
            name,
            samples,
            exit,
            legacy,
        }
    }

    pub fn ingest(
        &mut self,
        snap: &TelemetrySnapshot,
        processes: &[ProcessFootprint],
        diagnoses: &[Diagnosis],
    ) {
        self.ingest_observed(snap, processes, diagnoses, &HashSet::new(), &[]);
    }

    pub fn ingest_observed(
        &mut self,
        snap: &TelemetrySnapshot,
        processes: &[ProcessFootprint],
        diagnoses: &[Diagnosis],
        present: &HashSet<u32>,
        live: &[LiveProcess],
    ) {
        let tick = self
            .tracker
            .observe(Utc::now(), &snap.boot_id, present, live);
        let frame = frame_from_snapshot(snap, processes, diagnoses, &tick.samples, &tick.exits);
        self.ring.push_back(frame.clone());
        while self.ring.len() > self.ring_capacity {
            self.ring.pop_front();
        }

        let firing: std::collections::HashSet<_> = diagnoses
            .iter()
            .filter(|d| {
                d.lifecycle_state == DiagnosisLifecycleState::Persistent
                    && matches!(d.severity, DiagnosisSeverity::Critical | DiagnosisSeverity::Warning)
            })
            .map(|d| d.rule_id.clone())
            .collect();
        for rule_id in &firing {
            if !self.last_incident_rules.contains(rule_id) {
                if let Some(d) = diagnoses.iter().find(|x| x.rule_id == *rule_id) {
                    self.capture_incident(d);
                    self.last_incident_rules.insert(rule_id.clone());
                }
            }
        }
        let resolved_or_gone: Vec<_> = self
            .last_incident_rules
            .iter()
            .filter(|rule| !firing.contains(*rule))
            .cloned()
            .collect();
        for rule in resolved_or_gone {
            self.last_incident_rules.remove(&rule);
        }

        let mut finished_posts = Vec::new();
        let mut post_updates = Vec::new();
        for post in &mut self.post_captures {
            if post.remaining == 0 {
                finished_posts.push(post.incident_id);
                continue;
            }
            let last = post.remaining == 1;
            post.remaining -= 1;
            post_updates.push((post.incident_id, last));
            if last {
                finished_posts.push(post.incident_id);
            }
        }
        for (incident_id, last) in post_updates {
            if let Some(mut record) = self.get_incident(incident_id) {
                record.post_window.push(frame.clone());
                if last {
                    record.incomplete = false;
                }
                let _ = self.db.update_incident(&record);
            }
        }
        self.post_captures
            .retain(|p| !finished_posts.contains(&p.incident_id));

        if self.last_checkpoint.elapsed().as_secs() >= self.checkpoint_secs {
            self.last_checkpoint = Instant::now();
            if let Ok(json) = serde_json::to_string(&self.ring.iter().cloned().collect::<Vec<_>>()) {
                let _ = self.db.save_ring_checkpoint(&json);
            }
        }
    }

    fn capture_incident(&mut self, diagnosis: &Diagnosis) {
        let pre: Vec<_> = self.ring.iter().cloned().collect();
        let severity = match diagnosis.severity {
            DiagnosisSeverity::Critical => IncidentSeverity::Critical,
            DiagnosisSeverity::Warning => IncidentSeverity::Warning,
            DiagnosisSeverity::Info => IncidentSeverity::Info,
        };
        let record = IncidentRecord {
            id: Uuid::new_v4(),
            detected_at: Utc::now(),
            severity,
            kind: "diagnosis".into(),
            title_key: diagnosis.rule_id.clone(),
            summary: diagnosis.explanation.clone(),
            rule_id: Some(diagnosis.rule_id.clone()),
            pre_window: pre,
            post_window: vec![],
            incomplete: true,
            limitations: vec![
                "No se garantiza contexto tras kernel panic o corte de energía.".into(),
            ],
        };
        let _ = self.db.insert_incident(&record);
        let _ = self.db.trim_incidents(self.max_incidents);
        self.enqueue_post(PostCapture {
            incident_id: record.id,
            remaining: self.post_capture_frames,
        });
    }

    /// Incidente desde journald (OOM, NVRM, etc.); no inventa causa si solo hay el log.
    pub fn capture_journal_event(
        &mut self,
        title_key: &str,
        message: &str,
        severity: IncidentSeverity,
    ) {
        let pre: Vec<_> = self.ring.iter().cloned().collect();
        let record = IncidentRecord {
            id: Uuid::new_v4(),
            detected_at: Utc::now(),
            severity,
            kind: "journal".into(),
            title_key: title_key.into(),
            summary: message.chars().take(800).collect(),
            rule_id: None,
            pre_window: pre,
            post_window: vec![],
            incomplete: true,
            limitations: vec![
                "Origen: línea de journal del sistema; no sustituye un análisis forense completo.".into(),
                "La desaparición de un proceso por sí sola no demuestra fallo.".into(),
            ],
        };
        let _ = self.db.insert_incident(&record);
        let _ = self.db.trim_incidents(self.max_incidents);
        self.enqueue_post(PostCapture {
            incident_id: record.id,
            remaining: self.post_capture_frames,
        });
    }

    fn enqueue_post(&mut self, capture: PostCapture) {
        if self.post_captures.len() >= self.max_concurrent_post {
            self.post_captures.remove(0);
        }
        self.post_captures.push(capture);
    }

    pub fn ring_frames(&self) -> Vec<TelemetryFrame> {
        self.ring.iter().cloned().collect()
    }

    pub fn last_frame_exits(&self) -> Vec<ProcessExitEvent> {
        self.ring
            .back()
            .map(|f| f.process_exits.clone())
            .unwrap_or_default()
    }
}

fn frame_from_snapshot(
    snap: &TelemetrySnapshot,
    processes: &[ProcessFootprint],
    diagnoses: &[Diagnosis],
    observed: &[crate::types::HistoricalProcessSample],
    exits: &[crate::types::ProcessExitEvent],
) -> TelemetryFrame {
    use heimdall_core::SensorStatus;
    let val_f64 = |m: &heimdall_core::MetricValue<f64>| {
        if m.status == SensorStatus::Supported {
            m.value
        } else {
            None
        }
    };
    let val_u64 = |m: &heimdall_core::MetricValue<u64>| {
        if m.status == SensorStatus::Supported {
            m.value
        } else {
            None
        }
    };
    TelemetryFrame {
        observed_at: snap.observed_at,
        cpu_percent: val_f64(&snap.cpu.usage_percent_global),
        mem_available_bytes: val_u64(&snap.memory.available_bytes),
        swap_used_bytes: val_u64(&snap.memory.swap_used_bytes),
        gpu_percent: val_f64(&snap.gpu.utilization_percent),
        vram_used_bytes: val_u64(&snap.gpu.vram_used_bytes),
        cpu_temp_c: val_f64(&snap.cpu.temperature_c),
        gpu_temp_c: val_f64(&snap.gpu.temperature_c),
        psi_mem: val_f64(&snap.psi.memory_some_avg10),
        psi_cpu: val_f64(&snap.psi.cpu_some_avg10),
        psi_io: val_f64(&snap.psi.io_some_avg10),
        diagnosis_rules: diagnoses.iter().map(|d| d.rule_id.clone()).collect(),
        top_processes: if observed.is_empty() {
            processes
                .iter()
                .take(5)
                .map(|p| format!("{}:{}", p.name, p.pid))
                .collect()
        } else {
            observed
                .iter()
                .take(8)
                .map(|p| format!("{}:{}", p.name, p.pid))
                .collect()
        },
        schema_version: if observed.is_empty() && exits.is_empty() {
            1
        } else {
            FRAME_SCHEMA_VERSION
        },
        boot_id: snap.boot_id.clone(),
        processes: observed.to_vec(),
        process_exits: exits.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn stores_incident_on_persistent_diagnosis() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let mut rec = BlackBoxRecorder::open(path).unwrap();
        let snap = TelemetrySnapshot {
            observed_at: Utc::now(),
            session_id: Uuid::new_v4(),
            boot_id: "boot-test".into(),
            cpu: heimdall_core::CpuSnapshot {
                usage_percent_global: heimdall_core::MetricValue::supported(10.0, heimdall_core::Unit::Percent, "t"),
                usage_per_core: vec![],
                frequency_mhz_avg: heimdall_core::MetricValue::unsupported("t"),
                temperature_c: heimdall_core::MetricValue::unavailable("t"),
            },
            memory: heimdall_core::MemorySnapshot {
                used_bytes: heimdall_core::MetricValue::supported(1, heimdall_core::Unit::Bytes, "t"),
                available_bytes: heimdall_core::MetricValue::supported(1, heimdall_core::Unit::Bytes, "t"),
                total_bytes: heimdall_core::MetricValue::supported(2, heimdall_core::Unit::Bytes, "t"),
                swap_used_bytes: heimdall_core::MetricValue::supported(1, heimdall_core::Unit::Bytes, "t"),
            },
            gpu: heimdall_core::GpuSnapshot {
                name: None,
                active_provider: None,
                hint_code: None,
                utilization_percent: heimdall_core::MetricValue::unavailable("t"),
                vram_used_bytes: heimdall_core::MetricValue::unavailable("t"),
                vram_total_bytes: heimdall_core::MetricValue::unavailable("t"),
                temperature_c: heimdall_core::MetricValue::unavailable("t"),
                power_watts: heimdall_core::MetricValue::unavailable("t"),
                core_clock_mhz: heimdall_core::MetricValue::unavailable("t"),
                mem_clock_mhz: heimdall_core::MetricValue::unavailable("t"),
                fan_speed_percent: heimdall_core::MetricValue::unavailable("t"),
            },
            storage: vec![],
            network: heimdall_core::NetworkSnapshot {
                rx_bytes_per_sec: heimdall_core::MetricValue::supported(0.0, heimdall_core::Unit::BytesPerSecond, "t"),
                tx_bytes_per_sec: heimdall_core::MetricValue::supported(0.0, heimdall_core::Unit::BytesPerSecond, "t"),
            },
            system: heimdall_core::SystemSnapshot {
                uptime_seconds: heimdall_core::MetricValue::supported(1, heimdall_core::Unit::Seconds, "t"),
                load_avg_1: heimdall_core::MetricValue::supported(0.1, heimdall_core::Unit::None, "t"),
                load_avg_5: heimdall_core::MetricValue::supported(0.1, heimdall_core::Unit::None, "t"),
                load_avg_15: heimdall_core::MetricValue::supported(0.1, heimdall_core::Unit::None, "t"),
            },
            psi: heimdall_core::PsiSnapshot {
                cpu_some_avg10: heimdall_core::MetricValue::supported(0.0, heimdall_core::Unit::Percent, "t"),
                memory_some_avg10: heimdall_core::MetricValue::supported(0.0, heimdall_core::Unit::Percent, "t"),
                io_some_avg10: heimdall_core::MetricValue::supported(0.0, heimdall_core::Unit::Percent, "t"),
            },
        };
        let diag = Diagnosis {
            id: Uuid::new_v4(),
            rule_id: "memory_pressure".into(),
            title: "memory_pressure".into(),
            explanation: "test".into(),
            severity: DiagnosisSeverity::Warning,
            observed_at: Utc::now(),
            first_detected_at: Utc::now(),
            last_seen_at: Utc::now(),
            lifecycle_state: DiagnosisLifecycleState::Persistent,
            active_seconds: 60,
            evidence: vec![],
            affected_resources: vec![],
            related_processes: vec![],
            confidence: heimdall_core::Confidence::High,
            recommended_actions: vec![],
            limitations: vec![],
        };
        rec.ingest(&snap, &[], &[diag]);
        assert_eq!(rec.list_incidents(10).len(), 1);
    }

    #[test]
    fn journal_event_creates_incident() {
        let dir = tempdir().unwrap();
        let mut rec = BlackBoxRecorder::open(dir.path().join("j.db")).unwrap();
        rec.capture_journal_event(
            "journal_oom",
            "kernel: Out of memory: Killed process 9999 (test)",
            IncidentSeverity::Critical,
        );
        assert_eq!(rec.list_incidents(5).len(), 1);
        assert_eq!(rec.list_incidents(5)[0].title_key, "journal_oom");
    }

    #[test]
    fn legacy_incident_payload_still_deserializes() {
        let old = r#"{
            "id":"11111111-1111-1111-1111-111111111111",
            "detected_at":"2024-01-01T00:00:00Z",
            "severity":"warning",
            "kind":"diagnosis",
            "title_key":"cpu_saturation",
            "summary":"old",
            "rule_id":"cpu_saturation",
            "pre_window":[{
                "observed_at":"2024-01-01T00:00:00Z",
                "cpu_percent":90.0,
                "mem_available_bytes":1,
                "swap_used_bytes":0,
                "gpu_percent":null,
                "vram_used_bytes":null,
                "cpu_temp_c":null,
                "gpu_temp_c":null,
                "psi_mem":null,
                "diagnosis_rules":[],
                "top_processes":["oldproc:42"]
            }],
            "post_window":[],
            "incomplete":false,
            "limitations":[]
        }"#;
        let record: IncidentRecord = serde_json::from_str(old).unwrap();
        assert_eq!(record.pre_window[0].top_processes, vec!["oldproc:42".to_string()]);
        assert!(record.pre_window[0].processes.is_empty());
        assert_eq!(record.pre_window[0].schema_version, 0);
        assert_eq!(record.pre_window[0].cpu_percent, Some(90.0));
    }
}
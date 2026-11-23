use heimdall_blackbox::{BlackBoxRecorder, LiveProcess};
use heimdall_core::{Diagnosis, HardwareSummary, ProcessFootprint, TelemetrySnapshot};
use heimdall_blackbox::IncidentSeverity;
use heimdall_diagnostics::{
    config_for_profile, DiagnosticConfig, DiagnosticEngine, JournalHint, JournalHintKind,
};
use chrono::Utc;
use heimdall_intelligence::{
    ask as ollama_ask, build_context, probe as ollama_probe, AssistantReply, IntelligenceSettings,
    IntelligenceStore, OllamaStatus, PendingAssistantRequest,
};
use serde::{Deserialize, Serialize};
use heimdall_linux::{
    discover_hardware, footprint_from, JournalCache, JournalSignal, JournalSignalKind, LinuxSampler,
    ProcDetail,
};
use heimdall_investigation::{
    InvestigationConfig, InvestigationDossier, InvestigationEngine, InvestigationStore,
    InvestigationSummary, dossier_context_json,
};
use heimdall_profiles::{ActivityProfile, ProfileEngine, ProfileState, ProcessProfileOverride};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSample {
    pub by_cpu: Vec<ProcessFootprint>,
    pub by_memory: Vec<ProcessFootprint>,
}

pub struct TelemetryService {
    sampler: LinuxSampler,
    hardware: HardwareSummary,
    base_diagnostic_config: DiagnosticConfig,
    diagnostics: DiagnosticEngine,
    blackbox: BlackBoxRecorder,
    journal: JournalCache,
    seen_journal: HashSet<String>,
    profiles: ProfileEngine,
    last_profile: ProfileState,
    intelligence: IntelligenceStore,
    pending_assistant: Option<PendingAssistantRequest>,
    completed_assistant: Option<AssistantReply>,
    ready_assistant: Option<AssistantHttp>,
    cached_processes: ProcessSample,
    last_snapshot: Option<TelemetrySnapshot>,
    last_sample_at: Option<Instant>,
    investigations: InvestigationEngine,
    investigation_store: InvestigationStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantQueueStatus {
    pub pending: bool,
    pub question_preview: Option<String>,
    pub queued_at: Option<chrono::DateTime<Utc>>,
}

impl TelemetryService {
    pub fn new() -> Self {
        let hardware = discover_hardware();
        let base_diagnostic_config = DiagnosticConfig::default();
        let blackbox = BlackBoxRecorder::open_default().unwrap_or_else(|e| {
            tracing::warn!("blackbox unavailable: {e}");
            BlackBoxRecorder::open(std::path::PathBuf::from("./heimdall-blackbox.db"))
                .expect("fallback blackbox db")
        });
        let profiles = ProfileEngine::new();
        let last_profile = profiles.state();
        let intelligence = IntelligenceStore::open_default().unwrap_or_else(|e| {
            tracing::warn!("intelligence settings unavailable: {e}");
            IntelligenceStore::open(std::path::PathBuf::from("./heimdall-intelligence.json"))
                .expect("intelligence settings fallback")
        });
        let mut investigations = InvestigationEngine::new(InvestigationConfig::from_env());
        let investigation_store = InvestigationStore::open_default().unwrap_or_else(|e| {
            tracing::warn!("investigation store unavailable: {e}");
            InvestigationStore::open(std::path::PathBuf::from("./heimdall-blackbox.db"))
                .expect("investigation store fallback")
        });
        let _ = investigation_store.load_open_into_engine(&mut investigations);
        Self {
            sampler: LinuxSampler::new(),
            base_diagnostic_config: base_diagnostic_config.clone(),
            diagnostics: DiagnosticEngine::new(&hardware, base_diagnostic_config),
            hardware,
            blackbox,
            journal: JournalCache::new(Duration::from_secs(30)),
            seen_journal: HashSet::new(),
            profiles,
            last_profile,
            intelligence,
            pending_assistant: None,
            completed_assistant: None,
            ready_assistant: None,
            cached_processes: ProcessSample::default(),
            last_snapshot: None,
            last_sample_at: None,
            investigations,
            investigation_store,
        }
    }

    pub fn activity_profile(&self) -> &ProfileState {
        &self.last_profile
    }

    pub fn set_process_profile(
        &mut self,
        pattern: String,
        profile: ActivityProfile,
    ) -> Result<(), String> {
        self.profiles.store_mut().set_override(pattern, profile)
    }

    pub fn clear_process_profile(&mut self, pattern: &str) -> Result<(), String> {
        self.profiles.store_mut().remove_override(pattern)
    }

    pub fn process_profile_overrides(&self) -> Vec<ProcessProfileOverride> {
        self.profiles.overrides().to_vec()
    }

    fn ingest_journal_into_blackbox(&mut self, signals: &[JournalSignal]) {
        for s in signals {
            let key = format!("{:?}:{}", s.kind, s.message);
            if !self.seen_journal.insert(key) {
                continue;
            }
            let (title, severity) = match s.kind {
                JournalSignalKind::Oom => ("journal_oom", IncidentSeverity::Critical),
                JournalSignalKind::Nvrm => ("journal_nvrm", IncidentSeverity::Warning),
                JournalSignalKind::IoError => ("journal_io", IncidentSeverity::Warning),
                JournalSignalKind::Other => continue,
            };
            self.blackbox
                .capture_journal_event(title, &s.message, severity);
        }
    }

    fn journal_hints_from(&self, signals: &[JournalSignal]) -> Vec<JournalHint> {
        signals
            .iter()
            .map(|s| JournalHint {
                kind: match s.kind {
                    JournalSignalKind::Oom => JournalHintKind::Oom,
                    JournalSignalKind::Nvrm => JournalHintKind::Nvrm,
                    JournalSignalKind::IoError => JournalHintKind::IoError,
                    JournalSignalKind::Other => JournalHintKind::Oom,
                },
                message: s.message.clone(),
            })
            .collect()
    }

    pub fn hardware_summary(&self) -> &HardwareSummary {
        &self.hardware
    }

    pub fn snapshot(&mut self) -> TelemetrySnapshot {
        self.sampler.sample()
    }

    pub fn blackbox_status(&self) -> heimdall_blackbox::BlackBoxStatus {
        self.blackbox.status()
    }

    pub fn list_incidents(&self, limit: usize) -> Vec<heimdall_blackbox::IncidentSummary> {
        self.blackbox.list_incident_summaries(limit)
    }

    pub fn export_incident_technical(&self, id: uuid::Uuid) -> Result<String, String> {
        self.blackbox.export_technical(id)
    }

    pub fn export_incident_human(&self, id: uuid::Uuid) -> Result<String, String> {
        self.blackbox.export_human(id)
    }

    pub fn get_incident(&self, id: uuid::Uuid) -> Option<heimdall_blackbox::IncidentRecord> {
        self.blackbox.get_incident(id)
    }

    pub fn recent_process_exits(&self, limit: usize) -> Vec<heimdall_blackbox::ProcessExitEvent> {
        self.blackbox.recent_exits(limit)
    }

    pub fn reconstruct_process(
        &self,
        boot_id: &str,
        pid: u32,
        start_time_ticks: u64,
    ) -> heimdall_blackbox::ProcessReconstruction {
        self.blackbox.reconstruct(boot_id, pid, start_time_ticks)
    }

    pub fn list_investigations(&self, limit: usize) -> Vec<InvestigationSummary> {
        self.investigation_store
            .summaries(limit)
            .unwrap_or_default()
    }

    pub fn get_investigation(&self, id: uuid::Uuid) -> Option<InvestigationDossier> {
        self.investigation_store.get(id).ok().flatten()
    }

    pub fn export_investigation_json(&self, id: uuid::Uuid) -> Result<String, String> {
        self.investigation_store.export_json(id)
    }

    pub fn prepare_explain_investigation(
        &mut self,
        investigation_id: uuid::Uuid,
        model: Option<String>,
        question: String,
    ) -> Result<PreparedAssistant, String> {
        let dossier = self
            .get_investigation(investigation_id)
            .ok_or_else(|| "investigación no encontrada".to_string())?;
        let settings = self.intelligence.settings().clone();
        let resolved_model = model
            .filter(|s| !s.trim().is_empty())
            .or(settings.preferred_model.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "llama3.2".into());
        let ctx = dossier_context_json(&dossier);
        let prompt = if question.trim().is_empty() {
            "Explica este expediente en lenguaje claro, citando solo las evidencias incluidas.".into()
        } else {
            question
        };
        Ok(PreparedAssistant::Call(AssistantHttp {
            base_url: settings.ollama_base_url,
            model: resolved_model,
            context: ctx,
            question: prompt,
        }))
    }

    pub fn process_refresh_count(&self) -> u64 {
        self.sampler.process_refresh_count()
    }

    pub fn snapshot_and_diagnose(
        &mut self,
    ) -> (TelemetrySnapshot, ProcessSample, Vec<Diagnosis>) {
        let snapshot = self.sampler.sample();
        let present = self.sampler.present_pids().clone();
        let also = self.blackbox.tracked_pids();
        let details = self.sampler.enrichment(&also);
        let processes = ranked_sample(&details, snapshot.boot_id.as_str());
        let live: Vec<LiveProcess> = details.iter().cloned().map(live_from_detail).collect();
        let diag_cpu = head(&processes.by_cpu, 12);
        let diag_mem = head(&processes.by_memory, 12);
        let raw_journal = self.journal.signals().to_vec();
        self.ingest_journal_into_blackbox(&raw_journal);
        let journal = self.journal_hints_from(&raw_journal);
        self.last_profile = self.profiles.update(&snapshot, diag_mem);
        let diag_cfg = config_for_profile(
            self.last_profile.profile,
            &self.base_diagnostic_config,
        );
        self.diagnostics.set_config(diag_cfg);
        let diagnoses = self.diagnostics.step(
            snapshot.clone(),
            diag_cpu,
            diag_mem,
            &journal,
        );
        self.blackbox
            .ingest_observed(&snapshot, &processes.by_memory, &diagnoses, &present, &live);
        let frames = self.blackbox.ring_frames();
        let exits = self.blackbox.last_frame_exits();
        let profile_label = serde_json::to_string(&self.last_profile.profile)
            .unwrap_or_else(|_| "general".into());
        let inv = self.investigations.step(
            &frames,
            profile_label.trim_matches('"'),
            &exits,
            snapshot.observed_at,
        );
        if let Err(e) = self
            .investigation_store
            .apply_step(&inv.persist, &inv.closed_with_post)
        {
            tracing::warn!("investigation persist: {e}");
        }
        if let Some(last) = frames.last() {
            let _ = self.investigation_store.append_post_frames(last);
        }
        self.cached_processes = processes.clone();
        self.last_snapshot = Some(snapshot.clone());
        self.last_sample_at = Some(Instant::now());
        self.maybe_flush_pending_assistant(&snapshot);
        (snapshot, processes, diagnoses)
    }

    /// Relee el diagnóstico ya calculado en el último tick. No vuelve a recorrer `/proc`.
    pub fn run_diagnostics(&mut self) -> Vec<Diagnosis> {
        self.active_diagnoses().to_vec()
    }

    pub fn active_diagnoses(&self) -> &[Diagnosis] {
        self.diagnostics.active_diagnoses()
    }

    pub fn dismiss_diagnosis(&mut self, rule_id: &str) {
        self.diagnostics.dismiss_rule(rule_id);
    }

    /// Devuelve la muestra del último tick. Si tiene más de un segundo, ese tick es el suyo.
    pub fn process_sample(&mut self) -> ProcessSample {
        let stale = self
            .last_sample_at
            .map(|t| t.elapsed() >= Duration::from_millis(1000))
            .unwrap_or(true);
        if stale {
            let _ = self.snapshot_and_diagnose();
        }
        self.cached_processes.clone()
    }

    pub fn assistant_block_heavy_load(&self, snap: &TelemetrySnapshot) -> bool {
        use heimdall_core::SensorStatus;
        use heimdall_profiles::ActivityProfile;
        let gpu_hot = snap
            .gpu
            .utilization_percent
            .value
            .unwrap_or(0.0)
            > 55.0
            && snap.gpu.utilization_percent.status == SensorStatus::Supported;
        self.last_profile.profile == ActivityProfile::Gaming || gpu_hot
    }

    pub fn intelligence_settings(&self) -> IntelligenceSettings {
        self.intelligence.settings().clone()
    }

    pub fn set_intelligence_settings(&mut self, settings: IntelligenceSettings) -> Result<(), String> {
        self.intelligence.set_settings(settings)
    }

    pub fn ollama_status(&self) -> OllamaStatus {
        let base = self.intelligence.settings().ollama_base_url.as_str();
        ollama_probe(Some(base))
    }

    pub fn assistant_queue_status(&self) -> AssistantQueueStatus {
        match &self.pending_assistant {
            None => AssistantQueueStatus {
                pending: false,
                question_preview: None,
                queued_at: None,
            },
            Some(p) => AssistantQueueStatus {
                pending: true,
                question_preview: Some(truncate_preview(&p.question, 120)),
                queued_at: Some(p.queued_at),
            },
        }
    }

    pub fn take_completed_assistant(&mut self) -> Option<AssistantReply> {
        self.completed_assistant.take()
    }

    pub fn ask_assistant(&mut self, model: Option<String>, question: String) -> AssistantReply {
        match self.prepare_assistant(model, question) {
            PreparedAssistant::Finished(reply) => reply,
            PreparedAssistant::Call(request) => complete_assistant(request),
        }
    }

    /// Decide la cola o deja listo el HTTP. No llama a Ollama.
    pub fn prepare_assistant(
        &mut self,
        model: Option<String>,
        question: String,
    ) -> PreparedAssistant {
        let block = self
            .last_snapshot
            .as_ref()
            .is_some_and(|snap| self.assistant_block_heavy_load(snap));
        let settings = self.intelligence.settings().clone();
        let resolved_model = model
            .filter(|s| !s.trim().is_empty())
            .or(settings.preferred_model.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "llama3.2".into());
        let ctx = self.assistant_context_json();

        if block && settings.defer_when_blocked {
            self.pending_assistant = Some(PendingAssistantRequest {
                question: question.clone(),
                model: resolved_model.clone(),
                queued_at: Utc::now(),
            });
            return PreparedAssistant::Finished(AssistantReply {
                text: String::new(),
                model: resolved_model,
                blocked_reason: Some(
                    "Pregunta en cola: se enviará a Ollama cuando baje la carga o salgas de gaming."
                        .into(),
                ),
                queued: true,
            });
        }

        if block {
            return PreparedAssistant::Finished(AssistantReply {
                text: String::new(),
                model: resolved_model,
                blocked_reason: Some(
                    "Inferencia pospuesta: carga alta de GPU o perfil gaming activo.".into(),
                ),
                queued: false,
            });
        }

        PreparedAssistant::Call(AssistantHttp {
            base_url: settings.ollama_base_url,
            model: resolved_model,
            context: ctx,
            question,
        })
    }

    pub fn ollama_base_url(&self) -> String {
        self.intelligence.settings().ollama_base_url.clone()
    }

    fn assistant_context_json(&self) -> String {
        let host = self.hardware.hostname.clone();
        let diagnoses_json =
            serde_json::to_string(&self.active_diagnoses()).unwrap_or_else(|_| "[]".into());
        let profile_json =
            serde_json::to_string(self.activity_profile()).unwrap_or_else(|_| "{}".into());
        let incidents = self.blackbox_status().incidents_stored;
        build_context(&host, &diagnoses_json, &profile_json, incidents)
    }

    fn maybe_flush_pending_assistant(&mut self, snap: &TelemetrySnapshot) {
        let pending = match self.pending_assistant.clone() {
            Some(p) => p,
            None => return,
        };
        if pending.expired() {
            self.pending_assistant = None;
            self.completed_assistant = Some(AssistantReply {
                text: String::new(),
                model: pending.model,
                blocked_reason: Some(
                    "La pregunta en cola expiró (más de una hora sin ventana de inferencia).".into(),
                ),
                queued: false,
            });
            return;
        }
        if self.assistant_block_heavy_load(snap) {
            return;
        }
        let settings = self.intelligence.settings().clone();
        let ctx = self.assistant_context_json();
        self.pending_assistant = None;
        self.ready_assistant = Some(AssistantHttp {
            base_url: settings.ollama_base_url,
            model: pending.model,
            context: ctx,
            question: pending.question,
        });
    }

    /// Saca el HTTP de Ollama preparado por el tick. El llamador lo ejecuta sin el candado.
    pub fn take_ready_assistant(&mut self) -> Option<AssistantHttp> {
        self.ready_assistant.take()
    }

    pub fn store_completed_assistant(&mut self, reply: AssistantReply) {
        self.completed_assistant = Some(reply);
    }
}

pub struct AssistantHttp {
    pub base_url: String,
    pub model: String,
    pub context: String,
    pub question: String,
}

pub enum PreparedAssistant {
    Finished(AssistantReply),
    Call(AssistantHttp),
}

pub fn complete_assistant(request: AssistantHttp) -> AssistantReply {
    ollama_ask(
        Some(request.base_url.as_str()),
        &request.model,
        &request.context,
        &request.question,
        false,
    )
}

pub fn probe_ollama(base_url: &str) -> OllamaStatus {
    ollama_probe(Some(base_url))
}

fn head<T>(items: &[T], limit: usize) -> &[T] {
    let n = items.len().min(limit);
    &items[..n]
}

fn ranked_sample(details: &[ProcDetail], boot_id: &str) -> ProcessSample {
    let mut rows: Vec<ProcessFootprint> = details
        .iter()
        .map(|detail| footprint_from(detail, boot_id))
        .collect();
    let mut by_memory = rows.clone();
    rows.sort_by(|a, b| cmp_cpu_desc(a.cpu_percent, b.cpu_percent));
    rows.truncate(20);
    by_memory.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    by_memory.truncate(20);
    ProcessSample {
        by_cpu: rows,
        by_memory,
    }
}

fn cmp_cpu_desc(a: Option<f32>, b: Option<f32>) -> std::cmp::Ordering {
    match (a.filter(|v| v.is_finite()), b.filter(|v| v.is_finite())) {
        (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn live_from_detail(detail: ProcDetail) -> LiveProcess {
    LiveProcess {
        pid: detail.pid,
        start_time_ticks: detail.start_time_ticks,
        name: detail.name,
        cpu_core_percent: detail.cpu_core_percent.filter(|v| v.is_finite()),
        cpu_machine_percent: detail.cpu_machine_percent.filter(|v| v.is_finite()),
        memory_bytes: detail.memory_bytes,
        identity_confirmed: detail.identity_confirmed,
        follow: detail.follow,
    }
}

fn truncate_preview(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TelemetryService;

    #[test]
    fn one_tick_refreshes_processes_once_and_cache_does_not() {
        let mut service = TelemetryService::new();
        let before = service.process_refresh_count();
        let _ = service.snapshot_and_diagnose();
        assert_eq!(service.process_refresh_count(), before + 1);
        let _ = service.process_sample();
        assert_eq!(service.process_refresh_count(), before + 1);
        let _ = service.run_diagnostics();
        assert_eq!(service.process_refresh_count(), before + 1);
    }
}
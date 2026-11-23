//! Monitor en segundo plano (Fase 6): muestreo + socket Unix.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use heimdall_core::{Diagnosis, HardwareSummary, TelemetrySnapshot};
use heimdall_ipc::{
    default_socket_path, handle_cached_request, peer_is_owner, runtime_dir, IpcRequest, IpcResponse,
    IPC_PROTOCOL_VERSION,
};
use heimdall_profiles::ProfileState;
use heimdall_telemetry::{
    complete_assistant, probe_ollama, PreparedAssistant, ProcessSample, TelemetryService,
};
use uuid::Uuid;

struct DaemonCache {
    snapshot: Option<TelemetrySnapshot>,
    diagnoses: Vec<Diagnosis>,
    profile: ProfileState,
    processes: ProcessSample,
    hardware: HardwareSummary,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "heimdall_daemon=info".into()),
        )
        .init();

    let socket_path = default_socket_path();
    prepare_socket_dir(&socket_path);

    let service = Arc::new(Mutex::new(TelemetryService::new()));
    let hardware = service.lock().expect("lock").hardware_summary().clone();
    let profile = service.lock().expect("lock").activity_profile().clone();
    let cache = Arc::new(Mutex::new(DaemonCache {
        snapshot: None,
        diagnoses: vec![],
        profile,
        processes: ProcessSample::default(),
        hardware,
    }));

    let sample_service = service.clone();
    let sample_cache = cache.clone();
    thread::spawn(move || {
        loop {
            let pending = if let Ok(mut svc) = sample_service.lock() {
                let (snap, processes, diagnoses) = svc.snapshot_and_diagnose();
                let profile = svc.activity_profile().clone();
                let pending = svc.take_ready_assistant();
                if let Ok(mut c) = sample_cache.lock() {
                    c.snapshot = Some(snap);
                    c.processes = processes;
                    c.diagnoses = diagnoses;
                    c.profile = profile;
                }
                pending
            } else {
                None
            };
            if let Some(request) = pending {
                let reply = complete_assistant(request);
                if let Ok(mut svc) = sample_service.lock() {
                    svc.store_completed_assistant(reply);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).expect("bind unix socket");
    tracing::info!("heimdall-daemon listening on {}", socket_path.display());

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let cache = cache.clone();
                let service = service.clone();
                thread::spawn(move || handle_connection(s, cache, service));
            }
            Err(e) => tracing::warn!("accept error: {e}"),
        }
    }
}

fn prepare_socket_dir(socket_path: &std::path::Path) {
    let dir = runtime_dir();
    std::fs::create_dir_all(&dir).expect("create runtime dir");
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&dir).expect("dir metadata").permissions();
        perms.set_mode(0o700);
        let _ = std::fs::set_permissions(&dir, perms);
    }
    let _ = socket_path.parent().map(std::fs::create_dir_all);
}

fn handle_connection(
    stream: UnixStream,
    cache: Arc<Mutex<DaemonCache>>,
    service: Arc<Mutex<TelemetryService>>,
) {
    if !peer_is_owner(&stream) {
        tracing::warn!("ipc peer uid mismatch — conexión rechazada");
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let req: IpcRequest = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            let _ = write_response(
                stream,
                IpcResponse {
                    id: 0,
                    ok: false,
                    data: None,
                    error: Some(e.to_string()),
                    protocol_version: IPC_PROTOCOL_VERSION,
                },
            );
            return;
        }
    };

    let mut resp = match req.method.as_str() {
        "process_sample" => {
            let guard = cache.lock().expect("cache lock");
            match serde_json::to_value(&guard.processes) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "hardware_summary" => {
            let guard = cache.lock().expect("cache lock");
            match serde_json::to_value(&guard.hardware) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "run_diagnostics" | "active_diagnoses" => {
            let guard = cache.lock().expect("cache lock");
            match serde_json::to_value(&guard.diagnoses) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "blackbox_status" | "list_incidents" | "get_incident" | "export_incident_technical"
        | "export_incident_human" | "dismiss_diagnosis" | "set_process_profile"
        | "clear_process_profile" | "list_profile_overrides" | "recent_process_exits"
        | "reconstruct_process" | "get_intelligence_settings" | "set_intelligence_settings"
        | "list_investigations" | "get_investigation" | "export_investigation"
        | "assistant_queue_status" | "take_completed_assistant" => {
            dispatch_service(&req, &service, Some(&cache))
        }
        "ollama_status" => ollama_status_unlocked(&req, &service),
        "ask_assistant" => ask_unlocked(&req, &service),
        "explain_investigation" => explain_investigation_unlocked(&req, &service),
        _ => {
            let guard = cache.lock().expect("cache lock");
            handle_cached_request(
                &req.method,
                guard.snapshot.as_ref(),
                Some(&guard.diagnoses),
                Some(&guard.profile),
            )
        }
    };
    resp.id = req.id;
    let _ = write_response(stream, resp);
}

fn dispatch_service(
    req: &IpcRequest,
    service: &Arc<Mutex<TelemetryService>>,
    cache: Option<&Arc<Mutex<DaemonCache>>>,
) -> IpcResponse {
    let Ok(mut svc) = service.lock() else {
        return IpcResponse {
            id: req.id,
            ok: false,
            data: None,
            error: Some("daemon ocupado".into()),
            protocol_version: IPC_PROTOCOL_VERSION,
        };
    };
    match req.method.as_str() {
        "blackbox_status" => match serde_json::to_value(svc.blackbox_status()) {
            Ok(v) => ok_resp(req.id, v),
            Err(e) => err_resp(req.id, &e.to_string()),
        },
        "list_incidents" => {
            let limit = req.params.limit.unwrap_or(20);
            match serde_json::to_value(svc.list_incidents(limit)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "get_incident" => {
            let Some(id) = req.params.incident_id.as_deref() else {
                return err_resp(req.id, "falta incidentId");
            };
            let Ok(uuid) = Uuid::parse_str(id) else {
                return err_resp(req.id, "incidentId inválido");
            };
            match serde_json::to_value(svc.get_incident(uuid)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "export_incident_technical" => {
            let Some(id) = req.params.incident_id.as_deref() else {
                return err_resp(req.id, "falta incidentId");
            };
            let Ok(uuid) = Uuid::parse_str(id) else {
                return err_resp(req.id, "incidentId inválido");
            };
            match svc.export_incident_technical(uuid) {
                Ok(text) => ok_resp(req.id, serde_json::Value::String(text)),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "export_incident_human" => {
            let Some(id) = req.params.incident_id.as_deref() else {
                return err_resp(req.id, "falta incidentId");
            };
            let Ok(uuid) = Uuid::parse_str(id) else {
                return err_resp(req.id, "incidentId inválido");
            };
            match svc.export_incident_human(uuid) {
                Ok(text) => ok_resp(req.id, serde_json::Value::String(text)),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "dismiss_diagnosis" => {
            let Some(rule_id) = req.params.rule_id.as_deref() else {
                return err_resp(req.id, "falta ruleId");
            };
            svc.dismiss_diagnosis(rule_id);
            if let Some(cache) = cache {
                if let Ok(mut c) = cache.lock() {
                    c.diagnoses = svc.active_diagnoses().to_vec();
                }
            }
            ok_resp(req.id, serde_json::json!({ "dismissed": rule_id }))
        }
        "set_process_profile" => {
            let Some(pattern) = req.params.pattern.clone() else {
                return err_resp(req.id, "falta pattern");
            };
            let Some(profile) = req.params.profile.as_deref() else {
                return err_resp(req.id, "falta profile");
            };
            let Ok(profile) = serde_json::from_value::<heimdall_profiles::ActivityProfile>(
                serde_json::Value::String(profile.to_string()),
            ) else {
                return err_resp(req.id, "profile inválido");
            };
            match svc.set_process_profile(pattern, profile) {
                Ok(()) => ok_resp(req.id, serde_json::json!({ "ok": true })),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "clear_process_profile" => {
            let Some(pattern) = req.params.pattern.as_deref() else {
                return err_resp(req.id, "falta pattern");
            };
            match svc.clear_process_profile(pattern) {
                Ok(()) => ok_resp(req.id, serde_json::json!({ "ok": true })),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "list_profile_overrides" => match serde_json::to_value(svc.process_profile_overrides()) {
            Ok(v) => ok_resp(req.id, v),
            Err(e) => err_resp(req.id, &e.to_string()),
        },
        "recent_process_exits" => {
            let limit = req.params.limit.unwrap_or(20);
            match serde_json::to_value(svc.recent_process_exits(limit)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "list_investigations" => {
            let limit = req.params.limit.unwrap_or(50);
            match serde_json::to_value(svc.list_investigations(limit)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "get_investigation" => {
            let Some(id) = req.params.investigation_id.as_deref() else {
                return err_resp(req.id, "falta investigationId");
            };
            let Ok(uuid) = Uuid::parse_str(id) else {
                return err_resp(req.id, "investigationId inválido");
            };
            match serde_json::to_value(svc.get_investigation(uuid)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "export_investigation" => {
            let Some(id) = req.params.investigation_id.as_deref() else {
                return err_resp(req.id, "falta investigationId");
            };
            let Ok(uuid) = Uuid::parse_str(id) else {
                return err_resp(req.id, "investigationId inválido");
            };
            match svc.export_investigation_json(uuid) {
                Ok(json) => ok_resp(req.id, serde_json::json!({ "json": json })),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "reconstruct_process" => {
            let Some(boot_id) = req.params.boot_id.clone() else {
                return err_resp(req.id, "falta bootId");
            };
            let Some(pid) = req.params.pid else {
                return err_resp(req.id, "falta pid");
            };
            let Some(start) = req.params.start_time_ticks else {
                return err_resp(req.id, "falta startTimeTicks");
            };
            match serde_json::to_value(svc.reconstruct_process(&boot_id, pid, start)) {
                Ok(v) => ok_resp(req.id, v),
                Err(e) => err_resp(req.id, &e.to_string()),
            }
        }
        "get_intelligence_settings" => match serde_json::to_value(svc.intelligence_settings()) {
            Ok(v) => ok_resp(req.id, v),
            Err(e) => err_resp(req.id, &e.to_string()),
        },
        "set_intelligence_settings" => {
            let Some(payload) = req.params.payload.as_deref() else {
                return err_resp(req.id, "falta payload");
            };
            let Ok(settings) = serde_json::from_str(payload) else {
                return err_resp(req.id, "settings inválidos");
            };
            match svc.set_intelligence_settings(settings) {
                Ok(()) => ok_resp(req.id, serde_json::json!({ "ok": true })),
                Err(e) => err_resp(req.id, &e),
            }
        }
        "assistant_queue_status" => match serde_json::to_value(svc.assistant_queue_status()) {
            Ok(v) => ok_resp(req.id, v),
            Err(e) => err_resp(req.id, &e.to_string()),
        },
        "take_completed_assistant" => match serde_json::to_value(svc.take_completed_assistant()) {
            Ok(v) => ok_resp(req.id, v),
            Err(e) => err_resp(req.id, &e.to_string()),
        },
        _ => err_resp(req.id, "método de servicio desconocido"),
    }
}

fn ollama_status_unlocked(
    req: &IpcRequest,
    service: &Arc<Mutex<TelemetryService>>,
) -> IpcResponse {
    let base = {
        let Ok(svc) = service.lock() else {
            return err_resp(req.id, "daemon ocupado");
        };
        svc.ollama_base_url()
    };
    match serde_json::to_value(probe_ollama(&base)) {
        Ok(v) => ok_resp(req.id, v),
        Err(e) => err_resp(req.id, &e.to_string()),
    }
}

fn explain_investigation_unlocked(
    req: &IpcRequest,
    service: &Arc<Mutex<TelemetryService>>,
) -> IpcResponse {
    let Some(id_str) = req.params.investigation_id.clone() else {
        return err_resp(req.id, "falta investigationId");
    };
    let Ok(id) = Uuid::parse_str(&id_str) else {
        return err_resp(req.id, "investigationId inválido");
    };
    let question = req
        .params
        .question
        .clone()
        .unwrap_or_default();
    let prepared = {
        let Ok(mut svc) = service.lock() else {
            return err_resp(req.id, "daemon ocupado");
        };
        match svc.prepare_explain_investigation(id, req.params.model.clone(), question) {
            Ok(p) => p,
            Err(e) => return err_resp(req.id, &e),
        }
    };
    let reply = match prepared {
        PreparedAssistant::Finished(reply) => reply,
        PreparedAssistant::Call(request) => complete_assistant(request),
    };
    match serde_json::to_value(reply) {
        Ok(v) => ok_resp(req.id, v),
        Err(e) => err_resp(req.id, &e.to_string()),
    }
}

fn ask_unlocked(req: &IpcRequest, service: &Arc<Mutex<TelemetryService>>) -> IpcResponse {
    let Some(question) = req.params.question.clone() else {
        return err_resp(req.id, "falta question");
    };
    let prepared = {
        let Ok(mut svc) = service.lock() else {
            return err_resp(req.id, "daemon ocupado");
        };
        svc.prepare_assistant(req.params.model.clone(), question)
    };
    let reply = match prepared {
        PreparedAssistant::Finished(reply) => reply,
        PreparedAssistant::Call(request) => complete_assistant(request),
    };
    match serde_json::to_value(reply) {
        Ok(v) => ok_resp(req.id, v),
        Err(e) => err_resp(req.id, &e.to_string()),
    }
}

fn ok_resp(id: u64, data: serde_json::Value) -> IpcResponse {
    IpcResponse {
        id,
        ok: true,
        data: Some(data),
        error: None,
        protocol_version: IPC_PROTOCOL_VERSION,
    }
}

fn err_resp(id: u64, message: &str) -> IpcResponse {
    IpcResponse {
        id,
        ok: false,
        data: None,
        error: Some(message.into()),
        protocol_version: IPC_PROTOCOL_VERSION,
    }
}

fn write_response(mut stream: UnixStream, resp: IpcResponse) -> std::io::Result<()> {
    let line = serde_json::to_string(&resp).expect("serialize response");
    writeln!(stream, "{line}")?;
    stream.flush()
}
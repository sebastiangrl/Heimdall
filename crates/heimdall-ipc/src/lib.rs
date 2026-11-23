//! Protocolo IPC Heimdall (línea JSON por conexión Unix). Fase 6.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use heimdall_blackbox::{
    BlackBoxStatus, IncidentRecord, IncidentSummary, ProcessExitEvent, ProcessReconstruction,
};
use heimdall_core::{Diagnosis, HardwareSummary, TelemetrySnapshot};
use heimdall_profiles::ProfileState;
use heimdall_telemetry::ProcessSample;
use serde::{Deserialize, Serialize};

pub const IPC_PROTOCOL_VERSION: u32 = 1;

pub fn default_socket_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("heimdall")
        .join("heimdall.sock")
}

pub fn runtime_dir() -> PathBuf {
    default_socket_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IpcParams {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub incident_id: Option<String>,
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub start_time_ticks: Option<u64>,
    #[serde(default)]
    pub boot_id: Option<String>,
    #[serde(default)]
    pub question: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub payload: Option<String>,
    #[serde(default)]
    pub investigation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: IpcParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcResponse {
    pub id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub protocol_version: u32,
}

/// Solo acepta conexiones del mismo UID (Linux).
pub fn peer_is_owner(stream: &UnixStream) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();
        unsafe {
            let mut cred: libc::ucred = MaybeUninit::zeroed().assume_init();
            let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            let rc = libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &mut cred as *mut _ as *mut libc::c_void,
                &mut len,
            );
            rc == 0 && cred.uid == libc::getuid()
        }
    }
    #[cfg(not(unix))]
    {
        let _ = stream;
        true
    }
}

pub fn ping(socket_path: &Path) -> Result<(), String> {
    let resp = call(socket_path, "ping", 1, IpcParams::default(), Duration::from_millis(800))?;
    if resp.ok {
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "ping falló".into()))
    }
}

pub fn fetch_snapshot(socket_path: &Path) -> Result<TelemetrySnapshot, String> {
    decode_data(call(socket_path, "snapshot", 2, IpcParams::default(), Duration::from_secs(2))?)
}

pub fn fetch_active_diagnoses(socket_path: &Path) -> Result<Vec<Diagnosis>, String> {
    decode_data(call(
        socket_path,
        "active_diagnoses",
        3,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_activity_profile(socket_path: &Path) -> Result<ProfileState, String> {
    decode_data(call(
        socket_path,
        "activity_profile",
        4,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_process_sample(socket_path: &Path) -> Result<ProcessSample, String> {
    decode_data(call(
        socket_path,
        "process_sample",
        5,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_run_diagnostics(socket_path: &Path) -> Result<Vec<Diagnosis>, String> {
    decode_data(call(
        socket_path,
        "run_diagnostics",
        11,
        IpcParams::default(),
        Duration::from_secs(5),
    )?)
}

pub fn fetch_blackbox_status(socket_path: &Path) -> Result<BlackBoxStatus, String> {
    decode_data(call(
        socket_path,
        "blackbox_status",
        6,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_list_incidents(socket_path: &Path, limit: usize) -> Result<Vec<IncidentSummary>, String> {
    decode_data(call(
        socket_path,
        "list_incidents",
        7,
        IpcParams {
            limit: Some(limit),
            ..Default::default()
        },
        Duration::from_secs(3),
    )?)
}

pub fn fetch_incident(socket_path: &Path, id: &str) -> Result<Option<IncidentRecord>, String> {
    decode_data(call(
        socket_path,
        "get_incident",
        8,
        IpcParams {
            incident_id: Some(id.into()),
            ..Default::default()
        },
        Duration::from_secs(3),
    )?)
}

pub fn fetch_export_technical(socket_path: &Path, id: &str) -> Result<String, String> {
    decode_data(call(
        socket_path,
        "export_incident_technical",
        9,
        IpcParams {
            incident_id: Some(id.into()),
            ..Default::default()
        },
        Duration::from_secs(5),
    )?)
}

pub fn fetch_export_human(socket_path: &Path, id: &str) -> Result<String, String> {
    decode_data(call(
        socket_path,
        "export_incident_human",
        10,
        IpcParams {
            incident_id: Some(id.into()),
            ..Default::default()
        },
        Duration::from_secs(5),
    )?)
}

pub fn fetch_hardware(socket_path: &Path) -> Result<HardwareSummary, String> {
    decode_data(call(
        socket_path,
        "hardware_summary",
        30,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_set_process_profile(
    socket_path: &Path,
    pattern: &str,
    profile: &str,
) -> Result<(), String> {
    let resp = call(
        socket_path,
        "set_process_profile",
        31,
        IpcParams {
            pattern: Some(pattern.into()),
            profile: Some(profile.into()),
            ..Default::default()
        },
        Duration::from_secs(2),
    )?;
    if resp.ok {
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "set_process_profile falló".into()))
    }
}

pub fn fetch_list_profile_overrides(socket_path: &Path) -> Result<Vec<heimdall_profiles::ProcessProfileOverride>, String> {
    decode_data(call(
        socket_path,
        "list_profile_overrides",
        32,
        IpcParams::default(),
        Duration::from_secs(2),
    )?)
}

pub fn fetch_recent_exits(socket_path: &Path, limit: usize) -> Result<Vec<ProcessExitEvent>, String> {
    decode_data(call(
        socket_path,
        "recent_process_exits",
        33,
        IpcParams {
            limit: Some(limit),
            ..Default::default()
        },
        Duration::from_secs(2),
    )?)
}

pub fn fetch_list_investigations(
    socket_path: &Path,
    limit: usize,
) -> Result<Vec<heimdall_investigation::InvestigationSummary>, String> {
    decode_data(call(
        socket_path,
        "list_investigations",
        35,
        IpcParams {
            limit: Some(limit),
            ..Default::default()
        },
        Duration::from_secs(2),
    )?)
}

#[derive(serde::Deserialize)]
struct ExportInvestigationBody {
    json: String,
}

pub fn fetch_export_investigation(socket_path: &Path, investigation_id: &str) -> Result<String, String> {
    let body: ExportInvestigationBody = decode_data(call(
        socket_path,
        "export_investigation",
        37,
        IpcParams {
            investigation_id: Some(investigation_id.into()),
            ..Default::default()
        },
        Duration::from_secs(5),
    )?)?;
    Ok(body.json)
}

pub fn fetch_explain_investigation(
    socket_path: &Path,
    investigation_id: &str,
    question: Option<String>,
    model: Option<String>,
) -> Result<heimdall_intelligence::AssistantReply, String> {
    decode_data(call(
        socket_path,
        "explain_investigation",
        38,
        IpcParams {
            investigation_id: Some(investigation_id.into()),
            question,
            model,
            ..Default::default()
        },
        Duration::from_secs(60),
    )?)
}

pub fn fetch_get_investigation(
    socket_path: &Path,
    investigation_id: &str,
) -> Result<Option<heimdall_investigation::InvestigationDossier>, String> {
    decode_data(call(
        socket_path,
        "get_investigation",
        36,
        IpcParams {
            investigation_id: Some(investigation_id.into()),
            ..Default::default()
        },
        Duration::from_secs(2),
    )?)
}

pub fn fetch_reconstruct_process(
    socket_path: &Path,
    boot_id: &str,
    pid: u32,
    start_time_ticks: u64,
) -> Result<ProcessReconstruction, String> {
    decode_data(call(
        socket_path,
        "reconstruct_process",
        34,
        IpcParams {
            boot_id: Some(boot_id.into()),
            pid: Some(pid),
            start_time_ticks: Some(start_time_ticks),
            ..Default::default()
        },
        Duration::from_secs(2),
    )?)
}

pub fn handle_cached_request(
    method: &str,
    snapshot: Option<&TelemetrySnapshot>,
    diagnoses: Option<&[Diagnosis]>,
    profile: Option<&ProfileState>,
) -> IpcResponse {
    let id = 0;
    match method {
        "ping" => ok(id, serde_json::json!({ "message": "pong" })),
        "snapshot" => match snapshot {
            Some(s) => ok(id, serde_json::to_value(s).unwrap_or(serde_json::Value::Null)),
            None => err(id, "sin muestra aún"),
        },
        "active_diagnoses" => ok(
            id,
            serde_json::to_value(diagnoses.unwrap_or(&[])).unwrap_or(serde_json::json!([])),
        ),
        "activity_profile" => match profile {
            Some(p) => ok(id, serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
            None => err(id, "sin perfil"),
        },
        _ => err(id, &format!("método no en caché: {method}")),
    }
}

fn ok(id: u64, data: serde_json::Value) -> IpcResponse {
    IpcResponse {
        id,
        ok: true,
        data: Some(data),
        error: None,
        protocol_version: IPC_PROTOCOL_VERSION,
    }
}

fn err(id: u64, message: &str) -> IpcResponse {
    IpcResponse {
        id,
        ok: false,
        data: None,
        error: Some(message.into()),
        protocol_version: IPC_PROTOCOL_VERSION,
    }
}

fn decode_data<T: for<'de> Deserialize<'de>>(resp: IpcResponse) -> Result<T, String> {
    if !resp.ok {
        return Err(resp.error.unwrap_or_else(|| "ipc error".into()));
    }
  let data = resp.data.ok_or_else(|| "sin datos".to_string())?;
    serde_json::from_value(data).map_err(|e| e.to_string())
}

pub fn call(
    socket_path: &Path,
    method: &str,
    id: u64,
    params: IpcParams,
    timeout: Duration,
) -> Result<IpcResponse, String> {
    let mut stream = UnixStream::connect(socket_path).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let req = IpcRequest {
        id,
        method: method.into(),
        params,
    };
    let line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    writeln!(stream, "{line}").map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut resp_line = String::new();
    reader.read_line(&mut resp_line).map_err(|e| e.to_string())?;
    serde_json::from_str(resp_line.trim()).map_err(|e| e.to_string())
}
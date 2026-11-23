use heimdall_blackbox::{BlackBoxStatus, IncidentRecord, IncidentSummary};
use heimdall_core::{Diagnosis, HardwareSummary, TelemetrySnapshot};
use uuid::Uuid;
use heimdall_intelligence::{AssistantReply, IntelligenceSettings, OllamaStatus};
mod ipc_proxy;

use heimdall_ipc::{
    fetch_activity_profile, fetch_active_diagnoses, fetch_blackbox_status, fetch_export_human,
    fetch_export_technical, fetch_export_investigation, fetch_explain_investigation,
    fetch_get_investigation, fetch_hardware, fetch_incident, fetch_list_incidents,
    fetch_list_investigations, fetch_list_profile_overrides, fetch_process_sample,
    fetch_run_diagnostics, fetch_set_process_profile, fetch_snapshot, IpcParams,
};
use heimdall_investigation::{InvestigationDossier, InvestigationSummary};
use ipc_proxy::{daemon_call, daemon_reachable, dismiss_via_daemon, socket_path};
use heimdall_telemetry::ProcessSample;
use heimdall_telemetry::{AssistantQueueStatus, TelemetryService};
use heimdall_profiles::{ActivityProfile, ProcessProfileOverride, ProfileState};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent, State};
use tauri::WindowEvent;

const APP_ICON: tauri::image::Image<'_> = tauri::include_image!("icons/128x128.png");

fn apply_window_icon(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_icon(APP_ICON.clone());
    }
}

struct AppState {
    /// Solo existe cuando el daemon no responde. Un único escritor de telemetría.
    embedded: Mutex<Option<TelemetryService>>,
}

fn with_active<T>(
    state: &AppState,
    daemon_call: impl FnOnce() -> Result<T, String>,
    embedded_call: impl FnOnce(&mut TelemetryService) -> Result<T, String>,
) -> Result<T, String> {
    let mut slot = state.embedded.lock().map_err(|e| e.to_string())?;
    if daemon_reachable() {
        *slot = None;
        drop(slot);
        return daemon_call();
    }
    if slot.is_none() {
        *slot = Some(TelemetryService::new());
    }
    embedded_call(slot.as_mut().expect("servicio embebido"))
}

fn daemon_connected() -> bool {
    daemon_reachable()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitoringStatus {
    mode: &'static str,
    sampling_interval_ms: u64,
    daemon_connected: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonStatus {
    running: bool,
    embedded: bool,
    message: String,
}

#[tauri::command]
fn get_hardware_summary(state: State<'_, AppState>) -> Result<HardwareSummary, String> {
    with_active(
        &state,
        || fetch_hardware(socket_path().as_path()),
        |service| Ok(service.hardware_summary().clone()),
    )
}

#[tauri::command]
fn get_telemetry_snapshot(state: State<'_, AppState>) -> Result<TelemetrySnapshot, String> {
    with_active(
        &state,
        || fetch_snapshot(socket_path().as_path()),
        |service| {
            let snap = service.snapshot_and_diagnose().0;
            if let Some(request) = service.take_ready_assistant() {
                let reply = heimdall_telemetry::complete_assistant(request);
                service.store_completed_assistant(reply);
            }
            Ok(snap)
        },
    )
}

#[tauri::command]
fn get_active_diagnoses(state: State<'_, AppState>) -> Result<Vec<Diagnosis>, String> {
    with_active(
        &state,
        || fetch_active_diagnoses(socket_path().as_path()),
        |service| Ok(service.active_diagnoses().to_vec()),
    )
}

#[tauri::command]
fn run_diagnostics(state: State<'_, AppState>) -> Result<Vec<Diagnosis>, String> {
    with_active(
        &state,
        || fetch_run_diagnostics(socket_path().as_path()),
        |service| Ok(service.run_diagnostics()),
    )
}

#[tauri::command]
fn get_monitoring_status() -> MonitoringStatus {
    let connected = daemon_connected();
    MonitoringStatus {
        mode: if connected { "daemon" } else { "embedded" },
        sampling_interval_ms: 1_000,
        daemon_connected: connected,
    }
}

#[tauri::command]
fn get_daemon_status() -> DaemonStatus {
    let connected = daemon_connected();
    DaemonStatus {
        running: connected,
        embedded: !connected,
        message: if connected {
            "Daemon activo en socket Unix; la UI lee telemetría remota.".into()
        } else {
            "Monitor integrado en la app. Arranca `heimdall-daemon` para segundo plano.".into()
        },
    }
}

fn ipc_json<T: serde::de::DeserializeOwned>(
    method: &str,
    params: IpcParams,
    timeout: Duration,
) -> Result<T, String> {
    let resp = daemon_call(method, 60, params, timeout)?;
    let data = resp.data.ok_or_else(|| "sin datos".to_string())?;
    serde_json::from_value(data).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_blackbox_status(state: State<'_, AppState>) -> Result<BlackBoxStatus, String> {
    with_active(
        &state,
        || fetch_blackbox_status(socket_path().as_path()),
        |service| Ok(service.blackbox_status()),
    )
}

#[tauri::command]
fn list_incidents(state: State<'_, AppState>, limit: usize) -> Result<Vec<IncidentSummary>, String> {
    with_active(
        &state,
        move || fetch_list_incidents(socket_path().as_path(), limit),
        |service| Ok(service.list_incidents(limit)),
    )
}

#[tauri::command]
fn export_incident_technical(state: State<'_, AppState>, id: String) -> Result<String, String> {
    with_active(
        &state,
        {
            let id = id.clone();
            move || fetch_export_technical(socket_path().as_path(), &id)
        },
        move |service| {
            let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
            service.export_incident_technical(uuid)
        },
    )
}

#[tauri::command]
fn export_incident_human(state: State<'_, AppState>, id: String) -> Result<String, String> {
    with_active(
        &state,
        {
            let id = id.clone();
            move || fetch_export_human(socket_path().as_path(), &id)
        },
        move |service| {
            let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
            service.export_incident_human(uuid)
        },
    )
}

#[tauri::command]
fn get_incident(state: State<'_, AppState>, id: String) -> Result<Option<IncidentRecord>, String> {
    with_active(
        &state,
        {
            let id = id.clone();
            move || fetch_incident(socket_path().as_path(), &id)
        },
        move |service| {
            let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
            Ok(service.get_incident(uuid))
        },
    )
}

#[tauri::command]
fn get_process_sample(state: State<'_, AppState>) -> Result<ProcessSample, String> {
    with_active(
        &state,
        || fetch_process_sample(socket_path().as_path()),
        |service| {
            let sample = service.process_sample();
            if let Some(request) = service.take_ready_assistant() {
                let reply = heimdall_telemetry::complete_assistant(request);
                service.store_completed_assistant(reply);
            }
            Ok(sample)
        },
    )
}

#[tauri::command]
fn dismiss_diagnosis(state: State<'_, AppState>, rule_id: String) -> Result<(), String> {
    with_active(
        &state,
        {
            let rule_id = rule_id.clone();
            move || dismiss_via_daemon(&rule_id)
        },
        move |service| {
            service.dismiss_diagnosis(&rule_id);
            Ok(())
        },
    )
}

#[tauri::command]
fn get_activity_profile(state: State<'_, AppState>) -> Result<ProfileState, String> {
    with_active(
        &state,
        || fetch_activity_profile(socket_path().as_path()),
        |service| Ok(service.activity_profile().clone()),
    )
}

#[tauri::command]
fn list_profile_overrides(state: State<'_, AppState>) -> Result<Vec<ProcessProfileOverride>, String> {
    with_active(
        &state,
        || fetch_list_profile_overrides(socket_path().as_path()),
        |service| Ok(service.process_profile_overrides()),
    )
}

#[tauri::command]
fn get_ollama_status(state: State<'_, AppState>) -> Result<OllamaStatus, String> {
    with_active(
        &state,
        || ipc_json("ollama_status", IpcParams::default(), Duration::from_secs(5)),
        |service| Ok(service.ollama_status()),
    )
}

#[tauri::command]
fn get_intelligence_settings(state: State<'_, AppState>) -> Result<IntelligenceSettings, String> {
    with_active(
        &state,
        || ipc_json("get_intelligence_settings", IpcParams::default(), Duration::from_secs(2)),
        |service| Ok(service.intelligence_settings()),
    )
}

#[tauri::command]
fn set_intelligence_settings(
    state: State<'_, AppState>,
    settings: IntelligenceSettings,
) -> Result<(), String> {
    let payload = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    with_active(
        &state,
        {
            let payload = payload.clone();
            move || {
                daemon_call(
                    "set_intelligence_settings",
                    61,
                    IpcParams {
                        payload: Some(payload),
                        ..Default::default()
                    },
                    Duration::from_secs(2),
                )?;
                Ok(())
            }
        },
        move |service| service.set_intelligence_settings(settings),
    )
}

#[tauri::command]
fn get_assistant_queue_status(state: State<'_, AppState>) -> Result<AssistantQueueStatus, String> {
    with_active(
        &state,
        || ipc_json("assistant_queue_status", IpcParams::default(), Duration::from_secs(2)),
        |service| Ok(service.assistant_queue_status()),
    )
}

#[tauri::command]
fn take_completed_assistant(state: State<'_, AppState>) -> Result<Option<AssistantReply>, String> {
    with_active(
        &state,
        || ipc_json("take_completed_assistant", IpcParams::default(), Duration::from_secs(2)),
        |service| Ok(service.take_completed_assistant()),
    )
}

#[tauri::command]
fn ask_assistant(
    state: State<'_, AppState>,
    question: String,
    model: Option<String>,
) -> Result<AssistantReply, String> {
    with_active(
        &state,
        {
            let question = question.clone();
            let model = model.clone();
            move || {
                ipc_json(
                    "ask_assistant",
                    IpcParams {
                        question: Some(question),
                        model,
                        ..Default::default()
                    },
                    Duration::from_secs(50),
                )
            }
        },
        move |service| Ok(service.ask_assistant(model, question)),
    )
}

#[tauri::command]
fn list_investigations(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<InvestigationSummary>, String> {
    with_active(
        &state,
        move || fetch_list_investigations(socket_path().as_path(), limit),
        |service| Ok(service.list_investigations(limit)),
    )
}

#[tauri::command]
fn get_investigation(
    state: State<'_, AppState>,
    investigation_id: String,
) -> Result<Option<InvestigationDossier>, String> {
    let Ok(uuid) = Uuid::parse_str(&investigation_id) else {
        return Err("investigationId inválido".into());
    };
    with_active(
        &state,
        {
            let investigation_id = investigation_id.clone();
            move || fetch_get_investigation(socket_path().as_path(), &investigation_id)
        },
        move |service| Ok(service.get_investigation(uuid)),
    )
}

#[tauri::command]
fn export_investigation(
    state: State<'_, AppState>,
    investigation_id: String,
) -> Result<String, String> {
    with_active(
        &state,
        {
            let investigation_id = investigation_id.clone();
            move || fetch_export_investigation(socket_path().as_path(), &investigation_id)
        },
        {
            let investigation_id = investigation_id.clone();
            move |service| {
                let uuid = Uuid::parse_str(&investigation_id).map_err(|e| e.to_string())?;
                service.export_investigation_json(uuid)
            }
        },
    )
}

#[tauri::command]
fn explain_investigation(
    state: State<'_, AppState>,
    investigation_id: String,
    question: Option<String>,
    model: Option<String>,
) -> Result<AssistantReply, String> {
    let Ok(uuid) = Uuid::parse_str(&investigation_id) else {
        return Err("investigationId inválido".into());
    };
    with_active(
        &state,
        {
            let investigation_id = investigation_id.clone();
            let question = question.clone();
            let model = model.clone();
            move || {
                fetch_explain_investigation(
                    socket_path().as_path(),
                    &investigation_id,
                    question,
                    model,
                )
            }
        },
        {
            let question = question.clone();
            let model = model.clone();
            move |service| {
                match service.prepare_explain_investigation(
                    uuid,
                    model,
                    question.unwrap_or_default(),
                )? {
                    heimdall_telemetry::PreparedAssistant::Finished(reply) => Ok(reply),
                    heimdall_telemetry::PreparedAssistant::Call(request) => {
                        Ok(heimdall_telemetry::complete_assistant(request))
                    }
                }
            }
        },
    )
}

#[tauri::command]
fn set_process_profile_override(
    state: State<'_, AppState>,
    pattern: String,
    profile: ActivityProfile,
) -> Result<(), String> {
    let label = match serde_json::to_value(&profile) {
        Ok(serde_json::Value::String(value)) => value,
        _ => return Err("perfil inválido".into()),
    };
    with_active(
        &state,
        {
            let pattern = pattern.clone();
            let label = label.clone();
            move || fetch_set_process_profile(socket_path().as_path(), &pattern, &label)
        },
        move |service| service.set_process_profile(pattern, profile),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    std::env::set_var("GTK_APPLICATION_ID", "com.asgard.heimdall");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "heimdall_desktop=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            apply_window_icon(app.handle());
            if let Some(window) = app.get_webview_window("main") {
                use tauri::window::Color;
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
            }

            #[cfg(desktop)]
            {
                let show_i = MenuItem::with_id(app, "tray-show", "Abrir Heimdall", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "tray-quit", "Salir", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
                let _tray = TrayIconBuilder::new()
                    .icon(APP_ICON.clone())
                    .tooltip("Heimdall")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "tray-show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "tray-quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(move |tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .manage(AppState {
            embedded: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_hardware_summary,
            get_telemetry_snapshot,
            get_active_diagnoses,
            run_diagnostics,
            get_monitoring_status,
            get_daemon_status,
            get_blackbox_status,
            list_incidents,
            get_incident,
            export_incident_technical,
            export_incident_human,
            get_activity_profile,
            list_profile_overrides,
            set_process_profile_override,
            get_ollama_status,
            ask_assistant,
            get_intelligence_settings,
            set_intelligence_settings,
            get_assistant_queue_status,
            take_completed_assistant,
            get_process_sample,
            dismiss_diagnosis,
            list_investigations,
            get_investigation,
            export_investigation,
            explain_investigation,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Heimdall")
        .run(|app, event| {
            if matches!(event, RunEvent::Ready) {
                apply_window_icon(app);
            }
        });
}
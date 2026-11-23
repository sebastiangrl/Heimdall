use heimdall_ipc::{call, default_socket_path, ping, IpcParams, IpcResponse};
use std::time::Duration;

pub fn daemon_reachable() -> bool {
    ping(&default_socket_path()).is_ok()
}

pub fn socket_path() -> std::path::PathBuf {
    default_socket_path()
}

pub fn daemon_call(method: &str, id: u64, params: IpcParams, timeout: Duration) -> Result<IpcResponse, String> {
    let resp = call(&default_socket_path(), method, id, params, timeout)?;
    if resp.ok {
        Ok(resp)
    } else {
        Err(resp.error.unwrap_or_else(|| format!("{method} falló")))
    }
}

pub fn dismiss_via_daemon(rule_id: &str) -> Result<(), String> {
    let path = default_socket_path();
    let resp = call(
        &path,
        "dismiss_diagnosis",
        20,
        IpcParams {
            rule_id: Some(rule_id.into()),
            ..Default::default()
        },
        std::time::Duration::from_secs(2),
    )?;
    if resp.ok {
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "dismiss falló".into()))
    }
}
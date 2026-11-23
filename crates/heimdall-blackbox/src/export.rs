use crate::types::IncidentRecord;

pub fn export_incident_technical_json(record: &IncidentRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| e.to_string())
}

/// Informe con nombres de proceso parcialmente redactados.
pub fn export_incident_human_json(record: &IncidentRecord) -> Result<String, String> {
    let mut copy = record.clone();
    for frame in copy.pre_window.iter_mut().chain(copy.post_window.iter_mut()) {
        frame.top_processes = frame
            .top_processes
            .iter()
            .map(|p| redact_process_label(p))
            .collect();
    }
    #[derive(serde::Serialize)]
    struct HumanReport<'a> {
        id: &'a str,
        cuando: &'a str,
        gravedad: &'a str,
        titulo_clave: &'a str,
        resumen: &'a str,
        muestras_previas: usize,
        muestras_posteriores: usize,
        limitaciones: &'a [String],
        nota: &'a str,
    }
    copy.summary = scrub_paths(&copy.summary);
    let report = HumanReport {
        id: &copy.id.to_string(),
        cuando: &copy.detected_at.to_rfc3339(),
        gravedad: &format!("{:?}", copy.severity),
        titulo_clave: &copy.title_key,
        resumen: &copy.summary,
        muestras_previas: copy.pre_window.len(),
        muestras_posteriores: copy.post_window.len(),
        limitaciones: &copy.limitations,
        nota: "Exportación humana sin serie temporal completa; use el informe técnico para gráficas.",
    };
    serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
}

/// Redacta rutas tipo /home/usuario/... en textos de journal.
fn scrub_paths(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 6 < bytes.len() {
            let slice = &text[i..];
            if slice.starts_with("/home/") || slice.starts_with("/run/user/") {
                let end = slice
                    .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                    .unwrap_or(slice.len());
                out.push_str("[ruta redactada]");
                i += end;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

#[cfg(test)]
mod scrub_tests {
    use super::scrub_paths;

    #[test]
    fn redacts_home_paths() {
        let s = scrub_paths("error at /home/alice/secret/file");
        assert!(!s.contains("/home/alice"));
        assert!(s.contains("[ruta redactada]"));
    }
}

fn redact_process_label(label: &str) -> String {
    let (name, pid) = label.rsplit_once(':').unwrap_or((label, ""));
    let visible: String = name.chars().take(3).collect();
    if pid.is_empty() {
        format!("{visible}***")
    } else {
        format!("{visible}***:{pid}")
    }
}
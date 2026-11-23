use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_HOST: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub available: bool,
    pub base_url: String,
    pub models: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantReply {
    pub text: String,
    pub model: String,
    pub blocked_reason: Option<String>,
    pub queued: bool,
}

pub fn probe(base_url: Option<&str>) -> OllamaStatus {
    let base = base_url.unwrap_or(DEFAULT_HOST);
    let url = format!("{base}/api/tags");
    let agent = ureq::AgentBuilder::new().timeout_read(Duration::from_secs(3)).build();
    match agent.get(&url).call() {
        Ok(resp) => match resp.into_json::<TagsResponse>() {
            Ok(tags) => {
                let models: Vec<String> = tags
                    .models
                    .iter()
                    .map(|m| m.name.clone())
                    .collect();
                let empty = models.is_empty();
                OllamaStatus {
                    available: !empty,
                    base_url: base.into(),
                    models,
                    message: if empty {
                        "Ollama responde pero no hay modelos instalados.".into()
                    } else {
                        "Ollama disponible.".into()
                    },
                }
            }
            Err(e) => OllamaStatus {
                available: false,
                base_url: base.into(),
                models: vec![],
                message: format!("Respuesta inválida de Ollama: {e}"),
            },
        },
        Err(ureq::Error::Status(404, _)) => OllamaStatus {
            available: false,
            base_url: base.into(),
            models: vec![],
            message: "Ollama no encontrado en el puerto local.".into(),
        },
        Err(e) => OllamaStatus {
            available: false,
            base_url: base.into(),
            models: vec![],
            message: format!("Ollama no disponible: {e}"),
        },
    }
}

pub fn ask(
    base_url: Option<&str>,
    model: &str,
    system_context_json: &str,
    user_question: &str,
    block_heavy_load: bool,
) -> AssistantReply {
    if block_heavy_load {
        return AssistantReply {
            text: String::new(),
            model: model.into(),
            blocked_reason: Some(
                "Inferencia pospuesta: carga alta de GPU o perfil gaming activo.".into(),
            ),
            queued: false,
        };
    }

    let prompt = format!(
        "Eres Heimdall, asistente local del sistema. Responde en español.\n\
         Usa SOLO el contexto JSON; si falta evidencia, dilo.\n\
         No ejecutes comandos ni inventes métricas.\n\n\
         CONTEXTO:\n{system_context_json}\n\n\
         PREGUNTA:\n{user_question}\n"
    );

    let body = GenerateRequest {
        model: model.into(),
        prompt,
        stream: false,
    };

    let base = base_url.unwrap_or(DEFAULT_HOST);
    let url = format!("{base}/api/generate");
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(45))
        .build();

    match agent.post(&url).send_json(body) {
        Ok(resp) => match resp.into_json::<GenerateResponse>() {
            Ok(gen) => AssistantReply {
                text: gen.response.trim().to_string(),
                model: model.into(),
                blocked_reason: None,
                queued: false,
            },
            Err(e) => AssistantReply {
                text: String::new(),
                model: model.into(),
                blocked_reason: Some(format!("Error leyendo respuesta: {e}")),
                queued: false,
            },
        },
        Err(e) => AssistantReply {
            text: String::new(),
            model: model.into(),
            blocked_reason: Some(format!("No se pudo contactar Ollama: {e}")),
            queued: false,
        },
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

pub fn build_context(
    hostname: &str,
    diagnoses_json: &str,
    profile_json: &str,
    blackbox_count: u64,
) -> String {
    serde_json::json!({
        "hostname": hostname,
        "diagnoses": serde_json::from_str::<serde_json::Value>(diagnoses_json).unwrap_or(serde_json::Value::Null),
        "activity_profile": serde_json::from_str::<serde_json::Value>(profile_json).unwrap_or(serde_json::Value::Null),
        "blackbox_incidents": blackbox_count,
    })
    .to_string()
}
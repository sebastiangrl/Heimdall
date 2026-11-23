use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntelligenceSettings {
    pub ollama_base_url: String,
    pub preferred_model: Option<String>,
    /// Si la GPU está caliente o el perfil es gaming, encolar en lugar de fallar en seco.
    pub defer_when_blocked: bool,
}

impl Default for IntelligenceSettings {
    fn default() -> Self {
        Self {
            ollama_base_url: DEFAULT_OLLAMA_URL.into(),
            preferred_model: None,
            defer_when_blocked: true,
        }
    }
}

pub struct IntelligenceStore {
    path: PathBuf,
    settings: IntelligenceSettings,
}

impl IntelligenceStore {
    pub fn open_default() -> Result<Self, String> {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall")
            .join("intelligence.json");
        Self::open(path)
    }

    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let settings = if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            IntelligenceSettings::default()
        };
        Ok(Self { path, settings })
    }

    pub fn settings(&self) -> &IntelligenceSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: IntelligenceSettings) -> Result<(), String> {
        if settings.ollama_base_url.trim().is_empty() {
            return Err("La URL de Ollama no puede estar vacía.".into());
        }
        self.settings = settings;
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(&self.settings).map_err(|e| e.to_string())?;
        fs::write(&self.path, raw).map_err(|e| e.to_string())
    }
}
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::classifier::ActivityProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessProfileOverride {
    pub pattern: String,
    pub profile: ActivityProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoreFile {
  pub overrides: Vec<ProcessProfileOverride>,
}

pub struct ProfileStore {
    path: PathBuf,
    data: StoreFile,
}

impl ProfileStore {
    pub fn open_default() -> Result<Self, String> {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall")
            .join("profiles.json");
        Self::open(path)
    }

    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            StoreFile::default()
        };
        Ok(Self { path, data })
    }

    pub fn overrides(&self) -> &[ProcessProfileOverride] {
        &self.data.overrides
    }

    pub fn set_override(&mut self, pattern: String, profile: ActivityProfile) -> Result<(), String> {
        self.data
            .overrides
            .retain(|o| o.pattern.to_lowercase() != pattern.to_lowercase());
        self.data.overrides.push(ProcessProfileOverride { pattern, profile });
        self.save()
    }

    pub fn remove_override(&mut self, pattern: &str) -> Result<(), String> {
        let key = pattern.to_lowercase();
        self.data
            .overrides
            .retain(|o| o.pattern.to_lowercase() != key);
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.data).map_err(|e| e.to_string())?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }
}
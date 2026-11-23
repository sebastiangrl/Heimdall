use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::baseline::{BaselineSample, BaselineTracker};
use crate::classifier::ActivityProfile;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedBaseline {
    buckets: HashMap<ActivityProfile, Vec<BaselineSample>>,
}

pub struct BaselineStore {
    path: PathBuf,
}

impl BaselineStore {
    pub fn open_default() -> Result<Self, String> {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall")
            .join("baseline.json");
        Self::open(path)
    }

    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        Ok(Self { path })
    }

    pub fn load_into(&self, tracker: &mut BaselineTracker) {
        let raw = fs::read_to_string(&self.path).ok();
        let Some(raw) = raw else {
            return;
        };
        let data: PersistedBaseline = serde_json::from_str(&raw).unwrap_or_default();
        tracker.load_buckets(data.buckets);
    }

    pub fn save_from(&self, tracker: &BaselineTracker) -> Result<(), String> {
        let data = PersistedBaseline {
            buckets: tracker.export_buckets(),
        };
        let raw = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        fs::write(&self.path, raw).map_err(|e| e.to_string())
    }
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub fast_interval_ms: u64,
    pub slow_interval_ms: u64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            fast_interval_ms: 1_000,
            slow_interval_ms: 5_000,
        }
    }
}
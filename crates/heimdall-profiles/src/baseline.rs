use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use heimdall_core::{SensorStatus, TelemetrySnapshot};
use serde::{Deserialize, Serialize};

use crate::classifier::ActivityProfile;

const MAX_SAMPLES: usize = 2_000;
const MIN_FOR_BASELINE: usize = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BaselineSample {
    at: DateTime<Utc>,
    mem_used_ratio: f64,
    cpu_percent: f64,
    gpu_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileBaselineSummary {
    pub profile: ActivityProfile,
    pub sample_count: usize,
    pub learning: bool,
    pub mem_used_ratio_typical_min: f64,
    pub mem_used_ratio_typical_max: f64,
    pub cpu_percent_typical: f64,
    pub gpu_percent_typical: Option<f64>,
    pub observation: Option<String>,
}

pub struct BaselineTracker {
    buckets: std::collections::HashMap<ActivityProfile, VecDeque<BaselineSample>>,
}

impl Default for BaselineTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BaselineTracker {
    pub fn new() -> Self {
        Self {
            buckets: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn load_buckets(
        &mut self,
        buckets: std::collections::HashMap<ActivityProfile, Vec<BaselineSample>>,
    ) {
        for (profile, mut samples) in buckets {
            if samples.len() > MAX_SAMPLES {
                let skip = samples.len() - MAX_SAMPLES;
                samples = samples.split_off(skip);
            }
            let q = self.buckets.entry(profile).or_default();
            for s in samples {
                q.push_back(s);
            }
        }
    }

    pub(crate) fn export_buckets(
        &self,
    ) -> std::collections::HashMap<ActivityProfile, Vec<BaselineSample>> {
        self.buckets
            .iter()
            .map(|(k, dq)| {
                let v: Vec<_> = dq.iter().cloned().collect();
                (*k, v)
            })
            .collect()
    }

    pub fn record(&mut self, profile: ActivityProfile, snap: &TelemetrySnapshot) {
        let mem_used = snap.memory.used_bytes.value.unwrap_or(0);
        let mem_total = snap.memory.total_bytes.value.unwrap_or(1).max(1);
        let mem_ratio = mem_used as f64 / mem_total as f64;
        if !mem_ratio.is_finite() {
            return;
        }
        let Some(cpu) = supported_finite(&snap.cpu.usage_percent_global) else {
            return;
        };
        let gpu = supported_finite(&snap.gpu.utilization_percent);
        let q = self.buckets.entry(profile).or_default();
        q.push_back(BaselineSample {
            at: Utc::now(),
            mem_used_ratio: mem_ratio,
            cpu_percent: cpu,
            gpu_percent: gpu,
        });
        while q.len() > MAX_SAMPLES {
            q.pop_front();
        }
    }

    pub fn summary(&self, profile: ActivityProfile) -> ProfileBaselineSummary {
        let Some(samples) = self.buckets.get(&profile) else {
            return empty_summary(profile);
        };
        let n = samples.len();
        if n < MIN_FOR_BASELINE {
            return ProfileBaselineSummary {
                profile,
                sample_count: n,
                learning: true,
                mem_used_ratio_typical_min: 0.0,
                mem_used_ratio_typical_max: 0.0,
                cpu_percent_typical: 0.0,
                gpu_percent_typical: None,
                observation: None,
            };
        }
        let mut mem: Vec<f64> = samples
            .iter()
            .map(|s| s.mem_used_ratio)
            .filter(|v| v.is_finite())
            .collect();
        let mut cpu: Vec<f64> = samples
            .iter()
            .map(|s| s.cpu_percent)
            .filter(|v| v.is_finite())
            .collect();
        if mem.is_empty() || cpu.is_empty() {
            return ProfileBaselineSummary {
                profile,
                sample_count: n,
                learning: true,
                mem_used_ratio_typical_min: 0.0,
                mem_used_ratio_typical_max: 0.0,
                cpu_percent_typical: 0.0,
                gpu_percent_typical: None,
                observation: None,
            };
        }
        mem.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        cpu.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p25 = |v: &[f64]| v[v.len() / 4];
        let p75 = |v: &[f64]| v[(v.len() * 3) / 4];
        let mem_min = p25(&mem);
        let mem_max = p75(&mem);
        let cpu_typ = cpu.iter().sum::<f64>() / cpu.len() as f64;
        let gpu_typ = {
            let g: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.gpu_percent)
                .filter(|v| v.is_finite())
                .collect();
            if g.is_empty() {
                None
            } else {
                Some(g.iter().sum::<f64>() / g.len() as f64)
            }
        };
        let mut summary = ProfileBaselineSummary {
            profile,
            sample_count: n,
            learning: false,
            mem_used_ratio_typical_min: mem_min,
            mem_used_ratio_typical_max: mem_max,
            cpu_percent_typical: cpu_typ,
            gpu_percent_typical: gpu_typ,
            observation: None,
        };
        sanitize_summary(&mut summary);
        summary
    }

    pub fn compare_current(
        &self,
        profile: ActivityProfile,
        snap: &TelemetrySnapshot,
    ) -> ProfileBaselineSummary {
        let mut s = self.summary(profile);
        if s.learning {
            return s;
        }
        let used = snap.memory.used_bytes.value.unwrap_or(0);
        let total = snap.memory.total_bytes.value.unwrap_or(1).max(1);
        let ratio = used as f64 / total as f64;
        if ratio > s.mem_used_ratio_typical_max + 0.12 {
            s.observation = Some(format!(
                "Memoria ~{:.0}% supera tu banda habitual ({:.0}–{:.0}%) en este perfil.",
                ratio * 100.0,
                s.mem_used_ratio_typical_min * 100.0,
                s.mem_used_ratio_typical_max * 100.0
            ));
        }
        s
    }
}

fn sanitize_summary(summary: &mut ProfileBaselineSummary) {
    let finite = |v: f64| v.is_finite();
    if !finite(summary.mem_used_ratio_typical_min)
        || !finite(summary.mem_used_ratio_typical_max)
        || !finite(summary.cpu_percent_typical)
        || summary.mem_used_ratio_typical_min > summary.mem_used_ratio_typical_max
    {
        summary.learning = true;
        summary.mem_used_ratio_typical_min = 0.0;
        summary.mem_used_ratio_typical_max = 0.0;
        summary.cpu_percent_typical = 0.0;
        summary.gpu_percent_typical = None;
    }
    if let Some(g) = summary.gpu_percent_typical {
        if !g.is_finite() {
            summary.gpu_percent_typical = None;
        }
    }
}

fn supported_finite(metric: &heimdall_core::MetricValue<f64>) -> Option<f64> {
    if metric.status != SensorStatus::Supported {
        return None;
    }
    metric.value.filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::ActivityProfile;
    use chrono::Utc;

    #[test]
    fn baseline_summary_json_uses_camel_case() {
        let mut tracker = BaselineTracker::new();
        let q = tracker.buckets.entry(ActivityProfile::General).or_default();
        for i in 0..30 {
            q.push_back(BaselineSample {
                at: Utc::now(),
                mem_used_ratio: 0.4 + (i as f64) * 0.01,
                cpu_percent: 12.0,
                gpu_percent: None,
            });
        }
        let summary = tracker.summary(ActivityProfile::General);
        let json = serde_json::to_value(&summary).expect("json");
        assert!(json.get("memUsedRatioTypicalMin").is_some());
        assert!(json.get("cpuPercentTypical").is_some());
        assert!(!summary.learning);
    }

    #[test]
    fn summary_does_not_panic_on_non_finite_samples() {
        let mut tracker = BaselineTracker::new();
        let q = tracker.buckets.entry(ActivityProfile::General).or_default();
        for _ in 0..30 {
            q.push_back(BaselineSample {
                at: Utc::now(),
                mem_used_ratio: f64::NAN,
                cpu_percent: f64::NAN,
                gpu_percent: Some(f64::INFINITY),
            });
        }
        let summary = tracker.summary(ActivityProfile::General);
        assert!(summary.learning);
        assert!(summary.cpu_percent_typical.is_finite());
    }
}

fn empty_summary(profile: ActivityProfile) -> ProfileBaselineSummary {
    ProfileBaselineSummary {
        profile,
        sample_count: 0,
        learning: true,
        mem_used_ratio_typical_min: 0.0,
        mem_used_ratio_typical_max: 0.0,
        cpu_percent_typical: 0.0,
        gpu_percent_typical: None,
        observation: None,
    }
}
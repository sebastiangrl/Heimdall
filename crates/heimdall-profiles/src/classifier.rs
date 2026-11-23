use heimdall_core::{ProcessFootprint, SensorStatus, TelemetrySnapshot};
use serde::{Deserialize, Serialize};

use crate::store::ProcessProfileOverride;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityProfile {
    Gaming,
    ContentCreation,
    Development,
    General,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSignal {
    pub kind: String,
    pub detail: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityAssessment {
    pub profile: ActivityProfile,
    pub confidence: f32,
    pub signals: Vec<ProfileSignal>,
    pub dominant_apps: Vec<String>,
}

pub fn classify(
    snap: &TelemetrySnapshot,
    processes: &[ProcessFootprint],
    overrides: &[ProcessProfileOverride],
) -> ActivityAssessment {
    let cpu = metric_f64(&snap.cpu.usage_percent_global).unwrap_or(0.0);
    let gpu = metric_f64(&snap.gpu.utilization_percent).unwrap_or(0.0);
    let mem_used = snap.memory.used_bytes.value.unwrap_or(0);
    let mem_total = snap.memory.total_bytes.value.unwrap_or(1).max(1);
    let mem_ratio = mem_used as f64 / mem_total as f64;

    let mut scores = std::collections::HashMap::<ActivityProfile, f32>::new();
    let mut signals = Vec::new();

    for p in overrides {
        if processes.iter().any(|pr| name_matches(&pr.name, &p.pattern)) {
            *scores.entry(p.profile).or_insert(0.0) += 2.5;
            signals.push(ProfileSignal {
                kind: "user_override".into(),
                detail: format!(
                    "{} → {}",
                    p.pattern,
                    crate::profile_label(p.profile)
                ),
                weight: 2.5,
            });
        }
    }

    for proc in processes.iter().take(12) {
        let name = proc.name.to_lowercase();
        if let Some(profile) = builtin_process_profile(&name) {
            *scores.entry(profile).or_insert(0.0) += 1.8;
            signals.push(ProfileSignal {
                kind: "application".into(),
                detail: proc.name.clone(),
                weight: 1.8,
            });
        }
    }

    if gpu >= 55.0 && cpu >= 25.0 {
        *scores.entry(ActivityProfile::Gaming).or_insert(0.0) += 1.2;
        signals.push(ProfileSignal {
            kind: "telemetry".into(),
            detail: format!("GPU ~{gpu:.0}% y CPU ~{cpu:.0}%"),
            weight: 1.2,
        });
    }

    if cpu >= 70.0 && gpu < 40.0 {
        *scores.entry(ActivityProfile::Development).or_insert(0.0) += 0.8;
        signals.push(ProfileSignal {
            kind: "telemetry".into(),
            detail: format!("CPU alta ({cpu:.0}%) sin GPU dominante"),
            weight: 0.8,
        });
    }

    if mem_ratio > 0.55 && (cpu > 40.0 || gpu > 30.0) {
        *scores.entry(ActivityProfile::ContentCreation).or_insert(0.0) += 0.6;
        signals.push(ProfileSignal {
            kind: "telemetry".into(),
            detail: format!("Memoria ~{:.0}% con carga activa", mem_ratio * 100.0),
            weight: 0.6,
        });
    }

    let net_rx = metric_f64(&snap.network.rx_bytes_per_sec).unwrap_or(0.0);
    let net_tx = metric_f64(&snap.network.tx_bytes_per_sec).unwrap_or(0.0);
    let net_mbps = (net_rx + net_tx) / (1024.0 * 1024.0);
    if net_mbps > 2.5 {
        *scores.entry(ActivityProfile::ContentCreation).or_insert(0.0) += 0.5;
        signals.push(ProfileSignal {
            kind: "network".into(),
            detail: format!("Tráfico de red ~{net_mbps:.1} MiB/s"),
            weight: 0.5,
        });
    }

    let disk_pressure = snap.storage.iter().any(|d| {
        let total = d.total_bytes.value.unwrap_or(1).max(1);
        let used = d.used_bytes.value.unwrap_or(0);
        (used as f64 / total as f64) > 0.88
    });
    if disk_pressure {
        signals.push(ProfileSignal {
            kind: "storage".into(),
            detail: "Algún volumen con poco espacio libre".into(),
            weight: 0.4,
        });
    }

    if cpu < 8.0 && gpu < 5.0 && processes.len() < 80 {
        *scores.entry(ActivityProfile::Idle).or_insert(0.0) += 2.0;
        signals.push(ProfileSignal {
            kind: "telemetry".into(),
            detail: "Baja actividad de CPU y GPU".into(),
            weight: 2.0,
        });
    }

    *scores.entry(ActivityProfile::General).or_insert(0.0) += 0.3;

    let (profile, score) = scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((ActivityProfile::General, 0.3));

    let confidence = (score / 4.5).min(1.0).max(0.25);

    let dominant_apps: Vec<_> = processes
        .iter()
        .take(5)
        .map(|p| p.name.clone())
        .collect();

    ActivityAssessment {
        profile,
        confidence,
        signals: signals.into_iter().take(8).collect(),
        dominant_apps,
    }
}

fn builtin_process_profile(name: &str) -> Option<ActivityProfile> {
    const GAMING: &[&str] = &[
        "steam", "heroic", "lutris", "gamescope", "wine", "proton", "egl", "vulkan",
        "cs2", "dota", "minecraft", "factorio", "borderlands",
    ];
    const DEV: &[&str] = &[
        "cursor", "code", "codium", "helix", "nvim", "vim", "emacs", "jetbrains",
        "idea", "rust-analyzer", "cargo", "node", "npm", "pnpm", "docker", "podman",
        "konsole", "kitty", "alacritty", "ghostty", "wezterm",
    ];
    const CREATE: &[&str] = &[
        "blender", "davinci", "resolve", "kdenlive", "obs", "gimp", "inkscape",
        "audacity", "reaper", "figma",
    ];

    if GAMING.iter().any(|k| name.contains(k)) {
        return Some(ActivityProfile::Gaming);
    }
    if DEV.iter().any(|k| name.contains(k)) {
        return Some(ActivityProfile::Development);
    }
    if CREATE.iter().any(|k| name.contains(k)) {
        return Some(ActivityProfile::ContentCreation);
    }
    None
}

fn name_matches(process_name: &str, pattern: &str) -> bool {
    process_name.to_lowercase().contains(&pattern.to_lowercase())
}

fn metric_f64(m: &heimdall_core::MetricValue<f64>) -> Option<f64> {
    if m.status == SensorStatus::Supported {
        m.value
    } else {
        None
    }
}
use chrono::Utc;
use heimdall_core::{
    CpuSnapshot, GpuSnapshot, MemorySnapshot, MetricValue, NetworkSnapshot, PsiSnapshot,
    SystemSnapshot, TelemetrySnapshot, Unit,
};
use uuid::Uuid;

use crate::classifier::{classify, ActivityProfile};
use crate::classifier::ActivityAssessment;
use crate::store::ProcessProfileOverride;

fn base_snap() -> TelemetrySnapshot {
    TelemetrySnapshot {
        observed_at: Utc::now(),
        session_id: Uuid::new_v4(),
        boot_id: "boot".into(),
        cpu: CpuSnapshot {
            usage_percent_global: MetricValue::supported(12.0, Unit::Percent, "t"),
            usage_per_core: vec![],
            frequency_mhz_avg: MetricValue::unsupported("t"),
            temperature_c: MetricValue::unavailable("t"),
        },
        memory: MemorySnapshot {
            used_bytes: MetricValue::supported(4_000_000_000, Unit::Bytes, "t"),
            available_bytes: MetricValue::supported(12_000_000_000, Unit::Bytes, "t"),
            total_bytes: MetricValue::supported(16_000_000_000, Unit::Bytes, "t"),
            swap_used_bytes: MetricValue::supported(0, Unit::Bytes, "t"),
        },
        gpu: GpuSnapshot {
            name: None,
            active_provider: None,
            hint_code: None,
            utilization_percent: MetricValue::supported(75.0, Unit::Percent, "t"),
            vram_used_bytes: MetricValue::unavailable("t"),
            vram_total_bytes: MetricValue::unavailable("t"),
            temperature_c: MetricValue::unavailable("t"),
            power_watts: MetricValue::unavailable("t"),
            core_clock_mhz: MetricValue::unavailable("t"),
            mem_clock_mhz: MetricValue::unavailable("t"),
            fan_speed_percent: MetricValue::unavailable("t"),
        },
        storage: vec![],
        network: NetworkSnapshot {
            rx_bytes_per_sec: MetricValue::supported(0.0, Unit::BytesPerSecond, "t"),
            tx_bytes_per_sec: MetricValue::supported(0.0, Unit::BytesPerSecond, "t"),
        },
        system: SystemSnapshot {
            uptime_seconds: MetricValue::supported(100, Unit::Seconds, "t"),
            load_avg_1: MetricValue::supported(1.0, Unit::None, "t"),
            load_avg_5: MetricValue::supported(1.0, Unit::None, "t"),
            load_avg_15: MetricValue::supported(1.0, Unit::None, "t"),
        },
        psi: PsiSnapshot {
            cpu_some_avg10: MetricValue::supported(0.0, Unit::Percent, "t"),
            memory_some_avg10: MetricValue::supported(0.0, Unit::Percent, "t"),
            io_some_avg10: MetricValue::supported(0.0, Unit::Percent, "t"),
        },
    }
}

#[test]
fn steam_suggests_gaming() {
    let snap = base_snap();
    let procs = vec![heimdall_core::ProcessFootprint {
        pid: 1,
        name: "steam".into(),
        cpu_percent: Some(12.0),
        cpu_machine_percent: Some(1.0),
        memory_bytes: 1_000_000,
        boot_id: "boot".into(),
        start_time_ticks: 10,
    }];
    let a: ActivityAssessment = classify(&snap, &procs, &[]);
    assert_eq!(a.profile, ActivityProfile::Gaming);
}

#[test]
fn cursor_suggests_development() {
    let mut snap = base_snap();
    snap.cpu.usage_percent_global = MetricValue::supported(45.0, Unit::Percent, "t");
    snap.gpu.utilization_percent = MetricValue::supported(5.0, Unit::Percent, "t");
    let procs = vec![heimdall_core::ProcessFootprint {
        pid: 2,
        name: "cursor".into(),
        cpu_percent: Some(30.0),
        cpu_machine_percent: Some(2.5),
        memory_bytes: 2_000_000_000,
        boot_id: "boot".into(),
        start_time_ticks: 11,
    }];
    let a = classify(&snap, &procs, &[]);
    assert_eq!(a.profile, ActivityProfile::Development);
}

#[test]
fn user_override_wins() {
    let snap = base_snap();
    let procs = vec![heimdall_core::ProcessFootprint {
        pid: 3,
        name: "mi-juego".into(),
        cpu_percent: Some(50.0),
        cpu_machine_percent: Some(4.0),
        memory_bytes: 1_000_000,
        boot_id: "boot".into(),
        start_time_ticks: 12,
    }];
    let over = vec![ProcessProfileOverride {
        pattern: "mi-juego".into(),
        profile: ActivityProfile::Gaming,
    }];
    let a = classify(&snap, &procs, &over);
    assert_eq!(a.profile, ActivityProfile::Gaming);
}
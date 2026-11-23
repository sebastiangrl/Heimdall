use chrono::Utc;
use heimdall_core::{
    CpuCoreInfo, CpuSnapshot, HardwareSummary, MemoryInfo, MemorySnapshot, MetricValue, PsiSnapshot,
    StorageDeviceSnapshot, SystemSnapshot, TelemetrySnapshot, Unit,
};
use uuid::Uuid;

use crate::{DiagnosticConfig, DiagnosticEngine};

fn metric_u64(value: u64) -> MetricValue<u64> {
    MetricValue::supported(value, Unit::Bytes, "test")
}

fn metric_f64(value: f64) -> MetricValue<f64> {
    MetricValue::supported(value, Unit::Percent, "test")
}

fn snapshot_mem(available: u64, total: u64, swap: u64) -> TelemetrySnapshot {
    TelemetrySnapshot {
        observed_at: Utc::now(),
        session_id: Uuid::new_v4(),
        boot_id: "boot".into(),
        cpu: CpuSnapshot {
            usage_percent_global: metric_f64(20.0),
            usage_per_core: vec![],
            frequency_mhz_avg: MetricValue::unsupported("test"),
            temperature_c: MetricValue::unavailable("test"),
        },
        memory: MemorySnapshot {
            used_bytes: metric_u64(total.saturating_sub(available)),
            available_bytes: metric_u64(available),
            total_bytes: metric_u64(total),
            swap_used_bytes: metric_u64(swap),
        },
        gpu: heimdall_core::GpuSnapshot {
            name: None,
            active_provider: None,
            hint_code: None,
            utilization_percent: MetricValue::unavailable("nvml"),
            vram_used_bytes: MetricValue::unavailable("nvml"),
            vram_total_bytes: MetricValue::unavailable("nvml"),
            temperature_c: MetricValue::unavailable("nvml"),
            power_watts: MetricValue::unavailable("nvml"),
            core_clock_mhz: MetricValue::unavailable("nvml"),
            mem_clock_mhz: MetricValue::unavailable("nvml"),
            fan_speed_percent: MetricValue::unavailable("nvml"),
        },
        storage: vec![],
        network: heimdall_core::NetworkSnapshot {
            rx_bytes_per_sec: metric_f64(0.0),
            tx_bytes_per_sec: metric_f64(0.0),
        },
        system: SystemSnapshot {
            uptime_seconds: metric_u64(100),
            load_avg_1: metric_f64(1.0),
            load_avg_5: metric_f64(1.0),
            load_avg_15: metric_f64(1.0),
        },
        psi: PsiSnapshot {
            cpu_some_avg10: metric_f64(0.0),
            memory_some_avg10: metric_f64(12.0),
            io_some_avg10: metric_f64(0.0),
        },
    }
}

fn test_hardware() -> HardwareSummary {
    HardwareSummary {
        hostname: "test".into(),
        os_name: "Linux".into(),
        os_version: String::new(),
        kernel_version: String::new(),
        cpu: CpuCoreInfo {
            name: "Test CPU".into(),
            logical_cores: 8,
            physical_cores: Some(4),
            base_frequency_mhz: Some(3000.0),
        },
        memory: MemoryInfo {
            total_bytes: 16_000_000_000,
        },
        gpus: vec![],
        disks: vec![],
        discovered_at: Utc::now(),
    }
}

#[test]
fn memory_pressure_fires_with_low_available_and_swap_growth() {
    let mut config = DiagnosticConfig::default();
    config.memory_window_samples = 5;
    config.swap_growth_bytes_min = 64 * 1024 * 1024;

    let mut engine = DiagnosticEngine::new(&test_hardware(), config);
    let total = 16_000_000_000u64;
    for i in 0..6 {
        let swap = 200_000_000u64 + i * 80_000_000;
        let avail = 1_000_000_000u64;
        let snap = snapshot_mem(avail, total, swap);
        engine.step(snap, &[], &[], &[]);
    }

    let active = engine.active_diagnoses();
    assert!(
        active.iter().any(|d| d.rule_id == "memory_pressure"),
        "expected memory_pressure diagnosis"
    );
}

#[test]
fn cpu_saturation_requires_sustained_high_cpu() {
    let mut config = DiagnosticConfig::default();
    config.cpu_window_samples = 4;
    config.cpu_saturation_percent = 90.0;

    let mut engine = DiagnosticEngine::new(&test_hardware(), config);
    let total = 16_000_000_000u64;

    for _ in 0..3 {
        let mut snap = snapshot_mem(8_000_000_000, total, 0);
        snap.cpu.usage_percent_global = metric_f64(95.0);
        engine.step(snap, &[], &[], &[]);
    }
    assert!(!engine
        .active_diagnoses()
        .iter()
        .any(|d| d.rule_id == "cpu_saturation"));

        let mut snap = snapshot_mem(8_000_000_000, total, 0);
        snap.cpu.usage_percent_global = metric_f64(95.0);
        engine.step(snap, &[], &[], &[]);

    assert!(engine
        .active_diagnoses()
        .iter()
        .any(|d| d.rule_id == "cpu_saturation"));
}

#[test]
fn high_gpu_util_without_vram_pressure_does_not_alert() {
    let mut config = DiagnosticConfig::default();
    config.gpu_vram_window_samples = 4;
    let mut engine = DiagnosticEngine::new(&test_hardware(), config);
    let total = 16_000_000_000u64;
    for _ in 0..5 {
        let mut snap = snapshot_mem(8_000_000_000, total, 0);
        snap.gpu.utilization_percent = MetricValue::supported(98.0, Unit::Percent, "test");
        snap.gpu.vram_used_bytes = MetricValue::supported(2_000_000_000, Unit::Bytes, "test");
        snap.gpu.vram_total_bytes = MetricValue::supported(6_000_000_000, Unit::Bytes, "test");
        engine.step(snap, &[], &[], &[]);
    }
    assert!(!engine
        .active_diagnoses()
        .iter()
        .any(|d| d.rule_id == "gpu_vram_pressure"));
}

#[test]
fn storage_pressure_on_almost_full_disk() {
    let engine_cfg = DiagnosticConfig::default();
    let mut engine = DiagnosticEngine::new(&test_hardware(), engine_cfg);

    let total = 16_000_000_000u64;
    let mut snap = snapshot_mem(8_000_000_000, total, 0);
    snap.storage = vec![StorageDeviceSnapshot {
        name: "sdb".into(),
        mount_point: Some("/mnt/datos".into()),
        used_bytes: metric_u64(930_000_000_000),
        total_bytes: metric_u64(1_000_000_000_000),
        read_bytes_per_sec: MetricValue::unsupported("test"),
        write_bytes_per_sec: MetricValue::unsupported("test"),
    }];
    engine.step(snap, &[], &[], &[]);

    assert!(engine
        .active_diagnoses()
        .iter()
        .any(|d| d.rule_id == "storage_pressure"));
}

#[test]
fn nvrm_journal_triggers_gpu_driver_rule() {
    use crate::{JournalHint, JournalHintKind};

    let engine = DiagnosticEngine::new(&test_hardware(), DiagnosticConfig::default());
    let mut engine = engine;
    let total = 16_000_000_000u64;
    let snap = snapshot_mem(8_000_000_000, total, 0);
    let hints = vec![JournalHint {
        kind: JournalHintKind::Nvrm,
        message: "kernel: NVRM: Xid (PCI:0000:01:00): 31".into(),
    }];
    engine.step(snap, &[], &[], &hints);
    assert!(engine
        .active_diagnoses()
        .iter()
        .any(|d| d.rule_id == "gpu_driver"));
}
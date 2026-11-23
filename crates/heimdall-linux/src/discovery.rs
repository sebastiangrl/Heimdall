use std::fs;

use chrono::Utc;
use heimdall_core::{CpuCoreInfo, DiskInfo, GpuInfo, HardwareSummary, MemoryInfo};
use sysinfo::{Disks, System};

pub fn discover_hardware() -> HardwareSummary {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let os_name = System::name().unwrap_or_else(|| "Linux".into());
    let os_version = System::os_version().unwrap_or_default();
    let kernel_version = System::kernel_version().unwrap_or_default();

    let cpu = sys
        .cpus()
        .first()
        .map(|c| CpuCoreInfo {
            name: c.brand().to_string(),
            logical_cores: sys.cpus().len() as u32,
            physical_cores: None,
            base_frequency_mhz: Some(c.frequency() as f64),
        })
        .unwrap_or(CpuCoreInfo {
            name: "Unknown CPU".into(),
            logical_cores: 0,
            physical_cores: None,
            base_frequency_mhz: None,
        });

    let memory = MemoryInfo {
        total_bytes: sys.total_memory(),
    };

    let gpus = discover_gpus();
    let disks = discover_disks();

    HardwareSummary {
        hostname,
        os_name,
        os_version,
        kernel_version,
        cpu,
        memory,
        gpus,
        disks,
        discovered_at: Utc::now(),
    }
}

fn discover_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    if let Ok(out) = std::process::Command::new("lspci").args(["-nn"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga") || lower.contains("3d controller") {
                    let vendor = if lower.contains("nvidia") {
                        "NVIDIA"
                    } else if lower.contains("amd") || lower.contains("ati") {
                        "AMD"
                    } else if lower.contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    };
                    gpus.push(GpuInfo {
                        name: line.trim().to_string(),
                        vendor: vendor.into(),
                        vram_bytes: None,
                        driver_note: if vendor == "NVIDIA" {
                            Some("NVML unavailable without working NVIDIA driver".into())
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }
    gpus
}

fn discover_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|d| {
            let mount = d.mount_point().to_string_lossy().to_string();
            DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: if mount.is_empty() {
                    None
                } else {
                    Some(mount)
                },
                total_bytes: Some(d.total_space()),
                filesystem: Some(d.file_system().to_string_lossy().to_string()),
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn read_os_release_pretty() -> Option<String> {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
}
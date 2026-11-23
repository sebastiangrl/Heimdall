use std::collections::HashSet;

use chrono::Utc;
use heimdall_core::{
    finite_f64, CoreUsage, CpuSnapshot, MetricValue, StorageDeviceSnapshot, SystemSnapshot,
    TelemetrySnapshot, Unit,
};
use sysinfo::{CpuRefreshKind, Disks, ProcessRefreshKind, ProcessesToUpdate, System};
use uuid::Uuid;

use crate::cpu_temp::read_cpu_temperature_c;
use crate::gpu::GpuCollector;
use crate::network::NetworkCounterState;
use crate::proc_stat::{read_boot_id, read_start_ticks};
use crate::processes::{finite_cpu, rank_cpu, rank_memory, ProcDetail, RawProc};
use crate::psi::read_psi_snapshot;

/// Cuántos procesos se ofrecen a la UI y al diagnóstico desde la misma pasada.
pub const UI_PROCESS_LIMIT: usize = 20;
/// Subconjunto que la caja negra sigue aunque salga del top momentáneo.
pub const FOLLOW_CPU: usize = 8;
pub const FOLLOW_MEM: usize = 5;

pub struct LinuxSampler {
    sys: System,
    disks: Disks,
    network: NetworkCounterState,
    gpu: GpuCollector,
    session_id: Uuid,
    boot_id: String,
    logical_cores: u32,
    /// La primera lectura de CPU no tiene delta. No se publica como 0 %.
    cpu_primed: bool,
    process_refreshes: u64,
    present_pids: HashSet<u32>,
    rows: Vec<RawProc>,
}

impl Default for LinuxSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxSampler {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        let logical_cores = (sys.cpus().len() as u32).max(1);
        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            network: NetworkCounterState::default(),
            gpu: GpuCollector::new(),
            session_id: Uuid::new_v4(),
            boot_id: read_boot_id(),
            logical_cores,
            cpu_primed: false,
            process_refreshes: 0,
            present_pids: HashSet::new(),
            rows: Vec::new(),
        }
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn boot_id(&self) -> &str {
        &self.boot_id
    }

    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    /// Veces que este sampler ha recorrido la tabla de procesos. Una por `sample`.
    pub fn process_refresh_count(&self) -> u64 {
        self.process_refreshes
    }

    pub fn present_pids(&self) -> &HashSet<u32> {
        &self.present_pids
    }

    pub fn sample(&mut self) -> TelemetrySnapshot {
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.sys.refresh_memory();
        self.refresh_processes_once();

        let global_raw = self.sys.global_cpu_usage() as f64;
        let global_cpu = if self.cpu_primed {
            metric_percent(global_raw, "sysinfo:cpu")
        } else {
            MetricValue::unavailable("sysinfo:cpu")
        };

        let per_core: Vec<CoreUsage> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| {
                let usage = if self.cpu_primed {
                    metric_percent(cpu.cpu_usage() as f64, "sysinfo:cpu")
                } else {
                    MetricValue::unavailable("sysinfo:cpu")
                };
                CoreUsage {
                    core: i as u32,
                    usage_percent: usage,
                }
            })
            .collect();
        self.cpu_primed = true;

        let freq_avg = if self.sys.cpus().is_empty() {
            MetricValue::unavailable("sysinfo:cpufreq")
        } else {
            let sum: u64 = self.sys.cpus().iter().map(|c| c.frequency()).sum();
            let avg = sum as f64 / self.sys.cpus().len() as f64;
            match finite_f64(avg) {
                Some(v) => MetricValue::supported(v, Unit::Mhz, "sysinfo:cpufreq"),
                None => MetricValue::unavailable("sysinfo:cpufreq"),
            }
        };

        let cpu = CpuSnapshot {
            usage_percent_global: global_cpu,
            usage_per_core: per_core,
            frequency_mhz_avg: freq_avg,
            temperature_c: read_cpu_temperature_c(),
        };

        let memory = heimdall_core::MemorySnapshot {
            used_bytes: MetricValue::supported(self.sys.used_memory(), Unit::Bytes, "sysinfo:mem"),
            available_bytes: MetricValue::supported(
                self.sys.available_memory(),
                Unit::Bytes,
                "sysinfo:mem",
            ),
            total_bytes: MetricValue::supported(self.sys.total_memory(), Unit::Bytes, "sysinfo:mem"),
            swap_used_bytes: MetricValue::supported(
                self.sys.used_swap(),
                Unit::Bytes,
                "sysinfo:swap",
            ),
        };

        self.disks.refresh(true);
        let storage: Vec<StorageDeviceSnapshot> = self
            .disks
            .list()
            .iter()
            .map(|d| StorageDeviceSnapshot {
                name: d.name().to_string_lossy().to_string(),
                mount_point: {
                    let m = d.mount_point().to_string_lossy();
                    if m.is_empty() {
                        None
                    } else {
                        Some(m.to_string())
                    }
                },
                used_bytes: MetricValue::supported(
                    d.total_space().saturating_sub(d.available_space()),
                    Unit::Bytes,
                    "sysinfo:disk",
                ),
                total_bytes: MetricValue::supported(d.total_space(), Unit::Bytes, "sysinfo:disk"),
                read_bytes_per_sec: MetricValue::unsupported("sysinfo:disk:io"),
                write_bytes_per_sec: MetricValue::unsupported("sysinfo:disk:io"),
            })
            .collect();

        let uptime = System::uptime();
        let load = System::load_average();

        let system = SystemSnapshot {
            uptime_seconds: MetricValue::supported(uptime, Unit::Seconds, "sysinfo:uptime"),
            load_avg_1: metric_plain(load.one, "sysinfo:load"),
            load_avg_5: metric_plain(load.five, "sysinfo:load"),
            load_avg_15: metric_plain(load.fifteen, "sysinfo:load"),
        };

        if let Some(n) = self.sys.cpus().len().checked_mul(1) {
            if n > 0 {
                self.logical_cores = n as u32;
            }
        }

        TelemetrySnapshot {
            observed_at: Utc::now(),
            session_id: self.session_id,
            boot_id: self.boot_id.clone(),
            cpu,
            memory,
            gpu: self.gpu.sample(),
            storage,
            network: self.network.sample(),
            system,
            psi: read_psi_snapshot(),
        }
    }

    /// Una sola actualización de `/proc`. CPU y memoria salen de esta tabla.
    fn refresh_processes_once(&mut self) {
        self.process_refreshes += 1;
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
        let primed = self.cpu_primed;
        let cores = self.logical_cores.max(1);
        self.present_pids.clear();
        self.rows.clear();
        for (pid, proc_) in self.sys.processes() {
            let pid_u = pid.as_u32();
            self.present_pids.insert(pid_u);
            let (core, machine) = finite_cpu(proc_.cpu_usage(), primed, cores);
            self.rows.push(RawProc {
                pid: pid_u,
                name: proc_.name().to_string_lossy().to_string(),
                cpu_core_percent: core,
                cpu_machine_percent: machine,
                // sysinfo: memoria residente (RSS), no exclusiva ni VRAM.
                memory_bytes: proc_.memory(),
            });
        }
    }

    /// Completa identidad solo para el top y para PIDs que ya se siguen. No recorre `/proc` otra vez.
    pub fn enrichment(&self, also_follow: &[u32]) -> Vec<ProcDetail> {
        let cpu_top = rank_cpu(&self.rows, UI_PROCESS_LIMIT);
        let mem_top = rank_memory(&self.rows, UI_PROCESS_LIMIT);
        let mut follow = HashSet::new();
        for pid in rank_cpu(&self.rows, FOLLOW_CPU) {
            follow.insert(pid);
        }
        for pid in rank_memory(&self.rows, FOLLOW_MEM) {
            follow.insert(pid);
        }
        let mut want = follow.clone();
        for pid in cpu_top.iter().chain(mem_top.iter()) {
            want.insert(*pid);
        }
        for pid in also_follow {
            if self.present_pids.contains(pid) {
                want.insert(*pid);
            }
        }

        let mut out = Vec::with_capacity(want.len());
        for row in &self.rows {
            if !want.contains(&row.pid) {
                continue;
            }
            let start = read_start_ticks(row.pid);
            out.push(ProcDetail {
                pid: row.pid,
                name: row.name.clone(),
                cpu_core_percent: row.cpu_core_percent,
                cpu_machine_percent: row.cpu_machine_percent,
                memory_bytes: row.memory_bytes,
                start_time_ticks: start.unwrap_or(0),
                identity_confirmed: start.is_some(),
                follow: follow.contains(&row.pid),
            });
        }
        out
    }
}

fn metric_percent(value: f64, source: &str) -> MetricValue<f64> {
    match finite_f64(value) {
        Some(v) => MetricValue::supported(v, Unit::Percent, source),
        None => MetricValue::unavailable(source),
    }
}

fn metric_plain(value: f64, source: &str) -> MetricValue<f64> {
    match finite_f64(value) {
        Some(v) => MetricValue::supported(v, Unit::None, source),
        None => MetricValue::unavailable(source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn sample_returns_supported_cpu_memory() {
        let mut sampler = LinuxSampler::new();
        let _ = sampler.sample();
        std::thread::sleep(Duration::from_millis(250));
        let snap = sampler.sample();
        assert_eq!(
            snap.cpu.usage_percent_global.status,
            heimdall_core::SensorStatus::Supported
        );
        assert_eq!(
            snap.memory.total_bytes.status,
            heimdall_core::SensorStatus::Supported
        );
        assert!(!snap.boot_id.is_empty());
        assert_ne!(snap.session_id.to_string(), snap.boot_id);
    }

    #[test]
    #[ignore = "live host: sysinfo RSS must match /proc VmRSS"]
    fn process_memory_matches_proc_vmrss_for_top_pids() {
        use std::fs;
        use sysinfo::{ProcessesToUpdate, System};
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let mut ranked: Vec<_> = sys
            .processes()
            .iter()
            .map(|(pid, proc_)| (proc_.memory(), pid.as_u32()))
            .collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0));
        for (mem_bytes, pid) in ranked.into_iter().take(4) {
            let status = fs::read_to_string(format!("/proc/{pid}/status")).expect("status");
            let vmrss_kb: u64 = status
                .lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let proc_bytes = vmrss_kb * 1024;
            assert_eq!(
                mem_bytes, proc_bytes,
                "PID {pid}: sysinfo={mem_bytes} proc={proc_bytes}"
            );
        }
    }

    #[test]
    fn each_sample_refreshes_the_process_table_once() {
        let mut sampler = LinuxSampler::new();
        assert_eq!(sampler.process_refresh_count(), 0);
        let _ = sampler.sample();
        assert_eq!(sampler.process_refresh_count(), 1);
        let _ = sampler.enrichment(&[]);
        let _ = sampler.present_pids();
        assert_eq!(sampler.process_refresh_count(), 1);
        let _ = sampler.sample();
        assert_eq!(sampler.process_refresh_count(), 2);
    }
}

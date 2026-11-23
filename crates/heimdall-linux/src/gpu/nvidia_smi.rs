use std::process::{Command, Stdio};
use std::time::Duration;

use heimdall_core::{MetricValue, Unit};

const QUERY: &str = concat!(
    "name,utilization.gpu,temperature.gpu,memory.used,memory.total,",
    "power.draw,clocks.current.graphics,clocks.current.memory,fan.speed"
);

#[derive(Debug, Clone, Default)]
pub struct SmiReading {
    pub name: Option<String>,
    pub util: Option<f64>,
    pub temp_c: Option<f64>,
    pub mem_used_mib: Option<u64>,
    pub mem_total_mib: Option<u64>,
    pub power_w: Option<f64>,
    pub clock_graphics_mhz: Option<f64>,
    pub clock_mem_mhz: Option<f64>,
    pub fan_percent: Option<f64>,
    pub error: Option<String>,
}

pub fn query_nvidia_smi(timeout: Duration) -> SmiReading {
    let child = match Command::new("nvidia-smi")
        .args([
            &format!("--query-gpu={QUERY}"),
            "--format=csv,noheader,nounits",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return SmiReading {
                error: Some(format!("spawn: {e}")),
                ..Default::default()
            };
        }
    };

    let output = match wait_with_timeout(child, timeout) {
        Ok(o) => o,
        Err(e) => {
            return SmiReading {
                error: Some(e),
                ..Default::default()
            };
        }
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return SmiReading {
            error: Some(if err.is_empty() {
                format!("nvidia-smi exit {}", output.status)
            } else {
                err
            }),
            ..Default::default()
        };
    }

    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    parse_csv_line(&line)
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let start = std::time::Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            let out = child.wait_with_output().map_err(|e| e.to_string())?;
            return Ok(std::process::Output {
                status,
                stdout: out.stdout,
                stderr: out.stderr,
            });
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err("nvidia-smi timeout".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn parse_csv_line(line: &str) -> SmiReading {
    if line.is_empty() {
        return SmiReading {
            error: Some("empty nvidia-smi output".into()),
            ..Default::default()
        };
    }

    let parts: Vec<&str> = line.split(", ").collect();
    if parts.len() < 9 {
        return SmiReading {
            error: Some(format!("unexpected csv fields: {}", parts.len())),
            ..Default::default()
        };
    }

    let parse_f64 = |s: &str| -> Option<f64> {
        let t = s.trim();
        if t.eq_ignore_ascii_case("N/A") || t.is_empty() {
            None
        } else {
            t.parse().ok()
        }
    };
    let parse_u64_mib = |s: &str| -> Option<u64> {
        parse_f64(s).map(|m| (m * 1024.0 * 1024.0) as u64)
    };

    SmiReading {
        name: Some(parts[0].trim().to_string()),
        util: parse_f64(parts[1]),
        temp_c: parse_f64(parts[2]),
        mem_used_mib: parse_u64_mib(parts[3]),
        mem_total_mib: parse_u64_mib(parts[4]),
        power_w: parse_f64(parts[5]),
        clock_graphics_mhz: parse_f64(parts[6]),
        clock_mem_mhz: parse_f64(parts[7]),
        fan_percent: parse_f64(parts[8]),
        error: None,
    }
}

pub fn smi_to_metrics(read: &SmiReading, source: &str) -> heimdall_core::GpuSnapshot {
    use heimdall_core::GpuSnapshot;

    let mk_f = |v: Option<f64>, unit: Unit| -> MetricValue<f64> {
        v.map(|x| MetricValue::supported(x, unit, source))
            .unwrap_or_else(|| MetricValue::unavailable(source))
    };
    let mk_u = |v: Option<u64>, unit: Unit| -> MetricValue<u64> {
        v.map(|x| MetricValue::supported(x, unit, source))
            .unwrap_or_else(|| MetricValue::unavailable(source))
    };

    GpuSnapshot {
        name: read.name.clone(),
        active_provider: Some("nvidia-smi".into()),
        hint_code: None,
        utilization_percent: mk_f(read.util, Unit::Percent),
        vram_used_bytes: mk_u(read.mem_used_mib, Unit::Bytes),
        vram_total_bytes: mk_u(read.mem_total_mib, Unit::Bytes),
        temperature_c: mk_f(read.temp_c, Unit::Celsius),
        power_watts: mk_f(read.power_w, Unit::Watts),
        core_clock_mhz: mk_f(read.clock_graphics_mhz, Unit::Mhz),
        mem_clock_mhz: mk_f(read.clock_mem_mhz, Unit::Mhz),
        fan_speed_percent: mk_f(read.fan_percent, Unit::Percent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_line() {
        let line = "NVIDIA GeForce GTX 1060 6GB, 12, 59, 1051, 6144, 28.64, 1569, 4006, 0";
        let r = parse_csv_line(line);
        assert_eq!(r.util, Some(12.0));
        assert_eq!(r.temp_c, Some(59.0));
        assert!(r.mem_used_mib.is_some());
    }
}
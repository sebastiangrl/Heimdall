use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use heimdall_core::{GpuSnapshot, MetricValue, Unit};

use super::nvidia_smi::{query_nvidia_smi, smi_to_metrics};

const SMI_TIMEOUT: Duration = Duration::from_millis(800);
const SMI_INTERVAL: Duration = Duration::from_secs(2);
const NVML_RETRY: Duration = Duration::from_secs(30);

/// El handle NVML se conserva. nvidia-smi y lspci corren en un hilo aparte, nunca en cada tick.
pub struct GpuCollector {
    #[cfg(feature = "nvidia")]
    nvml: Option<nvml_wrapper::Nvml>,
    nvml_retry_at: Option<Instant>,
    cached: GpuSnapshot,
    smi: Option<SmiWorker>,
}

impl Default for GpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuCollector {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "nvidia")]
            nvml: None,
            nvml_retry_at: None,
            cached: fast_unavailable("gpu"),
            smi: None,
        }
    }

    pub fn sample(&mut self) -> GpuSnapshot {
        #[cfg(feature = "nvidia")]
        if self.nvml.is_none() && self.nvml_retry_due() {
            match nvml_wrapper::Nvml::init() {
                Ok(nvml) => {
                    self.smi = None;
                    self.nvml = Some(nvml);
                }
                Err(_) => {
                    self.nvml_retry_at = Some(Instant::now() + NVML_RETRY);
                    if self.smi.is_none() {
                        self.smi = Some(SmiWorker::start());
                    }
                }
            }
        }

        #[cfg(feature = "nvidia")]
        if let Some(nvml) = self.nvml.as_ref() {
            if let Some(snap) = sample_nvml(nvml) {
                self.cached = snap.clone();
                return snap;
            }
            self.nvml = None;
            self.nvml_retry_at = Some(Instant::now() + NVML_RETRY);
            if self.smi.is_none() {
                self.smi = Some(SmiWorker::start());
            }
        }

        #[cfg(not(feature = "nvidia"))]
        if self.smi.is_none() && self.nvml_retry_due() {
            self.nvml_retry_at = Some(Instant::now() + NVML_RETRY);
            self.smi = Some(SmiWorker::start());
        }

        if let Some(worker) = self.smi.as_ref() {
            if let Some(snap) = worker.latest() {
                self.cached = snap;
            }
        }
        self.cached.clone()
    }

    fn nvml_retry_due(&self) -> bool {
        match self.nvml_retry_at {
            None => true,
            Some(at) => Instant::now() >= at,
        }
    }
}

#[cfg(feature = "nvidia")]
fn sample_nvml(nvml: &nvml_wrapper::Nvml) -> Option<GpuSnapshot> {
    use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

    let device = nvml.device_by_index(0).ok()?;
    let name = device.name().ok();
    let util = device.utilization_rates().ok()?;
    let mem = device.memory_info().ok()?;
    let temp = device.temperature(TemperatureSensor::Gpu);
    let power = device.power_usage().ok();
    let graphics_clock = device.clock_info(Clock::Graphics);
    let mem_clock = device.clock_info(Clock::Memory);
    let fan = device.fan_speed(0);

    Some(GpuSnapshot {
        name,
        active_provider: Some("nvml".into()),
        hint_code: None,
        utilization_percent: finite_metric(util.gpu as f64, Unit::Percent, "nvml"),
        vram_used_bytes: MetricValue::supported(mem.used, Unit::Bytes, "nvml"),
        vram_total_bytes: MetricValue::supported(mem.total, Unit::Bytes, "nvml"),
        temperature_c: temp
            .ok()
            .map(|t| finite_metric(t as f64, Unit::Celsius, "nvml"))
            .unwrap_or_else(|| MetricValue::unavailable("nvml:temp")),
        power_watts: power
            .map(|mw| finite_metric(mw as f64 / 1000.0, Unit::Watts, "nvml"))
            .unwrap_or_else(|| MetricValue::unavailable("nvml:power")),
        core_clock_mhz: graphics_clock
            .ok()
            .map(|c| finite_metric(c as f64, Unit::Mhz, "nvml"))
            .unwrap_or_else(|| MetricValue::unavailable("nvml:clock")),
        mem_clock_mhz: mem_clock
            .ok()
            .map(|c| finite_metric(c as f64, Unit::Mhz, "nvml"))
            .unwrap_or_else(|| MetricValue::unavailable("nvml:clock_mem")),
        fan_speed_percent: fan
            .ok()
            .map(|f| finite_metric(f as f64, Unit::Percent, "nvml"))
            .unwrap_or_else(|| MetricValue::unavailable("nvml:fan")),
    })
}

fn finite_metric(value: f64, unit: Unit, source: &str) -> MetricValue<f64> {
    if value.is_finite() {
        MetricValue::supported(value, unit, source)
    } else {
        MetricValue::unavailable(source)
    }
}

struct SmiWorker {
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<GpuSnapshot>>>,
    handle: Option<JoinHandle<()>>,
}

impl SmiWorker {
    fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let latest = Arc::new(Mutex::new(None));
        let stop_bg = stop.clone();
        let latest_bg = latest.clone();
        let handle = thread::spawn(move || smi_loop(stop_bg, latest_bg));
        Self {
            stop,
            latest,
            handle: Some(handle),
        }
    }

    fn latest(&self) -> Option<GpuSnapshot> {
        self.latest.lock().ok().and_then(|g| g.clone())
    }
}

impl Drop for SmiWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            // nvidia-smi puede seguir dentro de su timeout. No retiene el tick de muestreo.
            thread::spawn(move || {
                let _ = handle.join();
            });
        }
    }
}

fn smi_loop(stop: Arc<AtomicBool>, latest: Arc<Mutex<Option<GpuSnapshot>>>) {
    while !stop.load(Ordering::Relaxed) {
        let reading = query_nvidia_smi(SMI_TIMEOUT);
        let snap = if reading.error.is_none() && reading.util.is_some() {
            smi_to_metrics(&reading, "nvidia-smi")
        } else {
            let mut snap = fast_unavailable("nvidia-smi");
            snap.hint_code = Some("gpu.hint.smi_failed".into());
            snap.name = reading.name;
            snap
        };
        if let Ok(mut slot) = latest.lock() {
            *slot = Some(snap);
        }
        let mut waited = Duration::ZERO;
        while waited < SMI_INTERVAL {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_millis(100));
            waited += Duration::from_millis(100);
        }
    }
}

fn fast_unavailable(source: &str) -> GpuSnapshot {
    GpuSnapshot {
        name: None,
        active_provider: None,
        hint_code: Some("gpu.hint.unknown".into()),
        utilization_percent: MetricValue::unavailable(source),
        vram_used_bytes: MetricValue::unavailable(source),
        vram_total_bytes: MetricValue::unavailable(source),
        temperature_c: MetricValue::unavailable(source),
        power_watts: MetricValue::unavailable(source),
        core_clock_mhz: MetricValue::unavailable(source),
        mem_clock_mhz: MetricValue::unavailable(source),
        fan_speed_percent: MetricValue::unavailable(source),
    }
}

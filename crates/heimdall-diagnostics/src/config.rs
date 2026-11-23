use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticConfig {
    /// Fraction of RAM still available before memory pressure (e.g. 0.12 = 12%).
    pub memory_available_ratio_warn: f64,
    /// Consecutive samples required for memory pressure.
    pub memory_window_samples: usize,
    /// PSI memory "some" avg10 threshold (Linux PSI).
    pub psi_memory_warn: f64,
    /// Minimum swap growth across window to count as swap activity (bytes).
    pub swap_growth_bytes_min: u64,
    /// CPU global de la máquina, escala 0–100. No es el porcentaje de un núcleo.
    pub cpu_saturation_percent: f64,
    pub cpu_window_samples: usize,
    /// Load average vs logical cores ratio.
    pub load_per_core_warn: f64,
    /// CPU de proceso en porcentaje de un núcleo (100 = un núcleo ocupado).
    /// No usa la escala de máquina. No cambiar este umbral al introducir `cpu_machine_percent`.
    pub process_cpu_highlight: f32,
    /// Disk used/total ratio warning.
    pub storage_used_ratio_warn: f64,
    /// PSI IO "some" avg10 threshold.
    pub psi_io_warn: f64,
    pub io_window_samples: usize,
    /// Cooldown before the same rule can fire again (seconds).
    pub rule_cooldown_secs: i64,
    /// Max snapshots kept for windows (~2 min at 1 Hz).
    pub history_capacity: usize,
    pub gpu_vram_ratio_warn: f64,
    pub gpu_vram_window_samples: usize,
    pub persistent_samples: u32,
    pub thermal_window_samples: usize,
    pub cpu_temp_warn_c: f64,
    pub cpu_temp_crit_c: f64,
    pub gpu_temp_warn_c: f64,
    pub gpu_temp_crit_c: f64,
    /// How long to keep resolved diagnoses visible in the UI (seconds).
    pub resolved_display_secs: i64,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            memory_available_ratio_warn: 0.12,
            memory_window_samples: 12,
            psi_memory_warn: 8.0,
            swap_growth_bytes_min: 32 * 1024 * 1024,
            cpu_saturation_percent: 88.0,
            cpu_window_samples: 12,
            load_per_core_warn: 0.95,
            process_cpu_highlight: 18.0,
            storage_used_ratio_warn: 0.92,
            psi_io_warn: 22.0,
            io_window_samples: 15,
            rule_cooldown_secs: 180,
            history_capacity: 150,
            gpu_vram_ratio_warn: 0.92,
            gpu_vram_window_samples: 10,
            persistent_samples: 8,
            thermal_window_samples: 10,
            cpu_temp_warn_c: 82.0,
            cpu_temp_crit_c: 95.0,
            gpu_temp_warn_c: 83.0,
            gpu_temp_crit_c: 92.0,
            resolved_display_secs: 120,
        }
    }
}
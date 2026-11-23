use std::env;

/// Regla experimental de ráfaga intermitente (`intermittent_cpu_burst`).
#[derive(Debug, Clone)]
pub struct InvestigationConfig {
    /// Ventana temporal hacia atrás (segundos de reloj, no asume 1 Hz exacto).
    pub burst_window_secs: i64,
    /// Muestras con CPU de un núcleo >= `burst_core_cpu_percent` dentro de la ventana.
    pub burst_min_hits: u32,
    /// Umbral en escala de un núcleo (70 ≈ 70 % de un núcleo lógico).
    pub burst_core_cpu_percent: f32,
    /// Segundos sin hits suficientes para cerrar una investigación abierta.
    pub burst_quiet_secs: i64,
    /// Cooldown tras cerrar antes de reabrir la misma identidad (segundos).
    pub identity_cooldown_secs: i64,
    /// Referencia previa: segundos antes del inicio de actividad para baseline global.
    pub baseline_lookback_secs: i64,
    /// Incremento global mínimo (máquina, puntos %) para confianza alta de atribución.
    pub attribution_min_global_delta: f64,
    /// Fracción del incremento global que debe explicar el proceso para confianza alta.
    pub attribution_dominance_ratio: f64,
    /// Cobertura mínima de muestras disponibles (0–1) para confianza alta.
    pub attribution_min_coverage: f64,
    pub psi_cpu_warn: f64,
    pub psi_mem_warn: f64,
    pub psi_io_warn: f64,
    pub global_cpu_saturation: f64,
    pub max_investigations: u32,
    pub max_investigation_age_days: u32,
    pub post_capture_frames: usize,
}

impl Default for InvestigationConfig {
    fn default() -> Self {
        Self {
            burst_window_secs: 20,
            burst_min_hits: 4,
            burst_core_cpu_percent: 70.0,
            burst_quiet_secs: 5,
            identity_cooldown_secs: 180,
            baseline_lookback_secs: 15,
            attribution_min_global_delta: 5.0,
            attribution_dominance_ratio: 0.6,
            attribution_min_coverage: 0.45,
            psi_cpu_warn: 15.0,
            psi_mem_warn: 8.0,
            psi_io_warn: 22.0,
            global_cpu_saturation: 88.0,
            max_investigations: 200,
            max_investigation_age_days: 30,
            post_capture_frames: 30,
        }
    }
}

impl InvestigationConfig {
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(v) = env::var("HEIMDALL_BURST_WINDOW_SECS") {
            if let Ok(n) = v.parse::<i64>() {
                c.burst_window_secs = n.clamp(5, 120);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_BURST_MIN_HITS") {
            if let Ok(n) = v.parse::<u32>() {
                c.burst_min_hits = n.clamp(2, 20);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_BURST_CORE_CPU") {
            if let Ok(n) = v.parse::<f32>() {
                c.burst_core_cpu_percent = n.clamp(10.0, 400.0);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_INVESTIGATION_MAX") {
            if let Ok(n) = v.parse::<u32>() {
                c.max_investigations = n.clamp(20, 5000);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_INVESTIGATION_COOLDOWN_SECS") {
            if let Ok(n) = v.parse::<i64>() {
                c.identity_cooldown_secs = n.clamp(30, 3600);
            }
        }
        c
    }
}
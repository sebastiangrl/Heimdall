use std::env;

#[derive(Debug, Clone)]
pub struct BlackBoxConfig {
    /// Muestras en el ring circular (~1 Hz → segundos de historia).
    pub ring_capacity: usize,
    /// Muestras a guardar tras un incidente.
    pub post_capture_frames: usize,
    pub checkpoint_secs: u64,
    /// Incidentes conservados. Los más antiguos se borran al insertar.
    pub max_incidents: u32,
}

impl Default for BlackBoxConfig {
    fn default() -> Self {
        Self {
            ring_capacity: 180,
            post_capture_frames: 30,
            checkpoint_secs: 15,
            max_incidents: 100,
        }
    }
}

impl BlackBoxConfig {
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(v) = env::var("HEIMDALL_BLACKBOX_RING") {
            if let Ok(n) = v.parse::<usize>() {
                c.ring_capacity = n.clamp(60, 600);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_BLACKBOX_POST_FRAMES") {
            if let Ok(n) = v.parse::<usize>() {
                c.post_capture_frames = n.clamp(5, 120);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_BLACKBOX_CHECKPOINT_SECS") {
            if let Ok(n) = v.parse::<u64>() {
                c.checkpoint_secs = n.clamp(5, 120);
            }
        }
        if let Ok(v) = env::var("HEIMDALL_BLACKBOX_MAX_INCIDENTS") {
            if let Ok(n) = v.parse::<u32>() {
                c.max_incidents = n.clamp(20, 2_000);
            }
        }
        c
    }
}
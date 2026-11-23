use heimdall_profiles::ActivityProfile;

use crate::config::DiagnosticConfig;

/// Ajusta umbrales según el perfil activo (sin tocar el hardware).
pub fn config_for_profile(profile: ActivityProfile, base: &DiagnosticConfig) -> DiagnosticConfig {
    let mut c = base.clone();
    match profile {
        ActivityProfile::Gaming => {
            c.cpu_saturation_percent = 96.0;
            c.gpu_temp_warn_c = c.gpu_temp_warn_c + 4.0;
            c.persistent_samples = c.persistent_samples + 2;
        }
        ActivityProfile::Development | ActivityProfile::ContentCreation => {
            c.cpu_saturation_percent = 92.0;
            c.memory_available_ratio_warn = 0.10;
        }
        ActivityProfile::Idle => {
            c.cpu_saturation_percent = 75.0;
            c.memory_available_ratio_warn = 0.15;
            c.persistent_samples = c.persistent_samples.saturating_sub(2).max(4);
        }
        ActivityProfile::General => {}
    }
    c
}
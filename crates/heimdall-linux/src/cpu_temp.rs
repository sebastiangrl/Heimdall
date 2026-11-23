use std::fs;
use std::path::PathBuf;

use heimdall_core::{MetricValue, SensorStatus, Unit};

/// Reads the first reasonable CPU package temperature from hwmon.
pub fn read_cpu_temperature_c() -> MetricValue<f64> {
    let hwmon_root = PathBuf::from("/sys/class/hwmon");
    let Ok(entries) = fs::read_dir(&hwmon_root) else {
        return MetricValue::unavailable("hwmon");
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        let Ok(name) = fs::read_to_string(&name_path) else {
            continue;
        };
        let name = name.trim();
        if name == "coretemp" || name.contains("k10temp") || name.contains("zenpower") {
            for i in 1..=8 {
                let temp_input = path.join(format!("temp{i}_input"));
                let temp_label = path.join(format!("temp{i}_label"));
                if let Ok(raw) = fs::read_to_string(&temp_input) {
                    let label = fs::read_to_string(&temp_label)
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();
                    if label.contains("package") || i == 1 {
                        if let Ok(milli_c) = raw.trim().parse::<i64>() {
                            let c = milli_c as f64 / 1000.0;
                            return MetricValue::supported(
                                c,
                                Unit::Celsius,
                                format!("hwmon:{name}"),
                            );
                        }
                    }
                }
            }
        }
    }

    MetricValue {
        status: SensorStatus::Unavailable,
        value: None,
        unit: None,
        observed_at: chrono::Utc::now(),
        source: Some("hwmon".into()),
    }
}
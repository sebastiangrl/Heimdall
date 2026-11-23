use std::fs;

use heimdall_core::{MetricValue, Unit};

fn read_psi_some_avg10(kind: &str) -> MetricValue<f64> {
    let path = format!("/proc/pressure/{kind}");
    let Ok(content) = fs::read_to_string(&path) else {
        return MetricValue::unavailable(format!("psi:{kind}"));
    };

    for line in content.lines() {
        if line.starts_with("some") {
            for token in line.split_whitespace() {
                if let Some(rest) = token.strip_prefix("avg10=") {
                    if let Ok(v) = rest.parse::<f64>() {
                        return MetricValue::supported(
                            v,
                            Unit::Percent,
                            format!("psi:{kind}:some"),
                        );
                    }
                }
            }
        }
    }

    MetricValue::unavailable(format!("psi:{kind}"))
}

pub fn read_psi_snapshot() -> heimdall_core::PsiSnapshot {
    heimdall_core::PsiSnapshot {
        cpu_some_avg10: read_psi_some_avg10("cpu"),
        memory_some_avg10: read_psi_some_avg10("memory"),
        io_some_avg10: read_psi_some_avg10("io"),
    }
}
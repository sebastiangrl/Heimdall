use heimdall_core::{MetricValue, SensorStatus, TelemetrySnapshot};

pub fn metric_f64<T: Copy + Into<f64>>(m: &MetricValue<T>) -> Option<f64> {
    if m.status != SensorStatus::Supported {
        return None;
    }
    m.value.map(|v| v.into())
}

pub fn metric_u64(m: &MetricValue<u64>) -> Option<u64> {
    if m.status != SensorStatus::Supported {
        return None;
    }
    m.value
}

pub fn available_memory_ratio(snap: &TelemetrySnapshot) -> Option<f64> {
    let avail = metric_u64(&snap.memory.available_bytes)?;
    let total = metric_u64(&snap.memory.total_bytes)?;
    if total == 0 {
        return None;
    }
    Some(avail as f64 / total as f64)
}

pub fn window_all<F>(history: &[TelemetrySnapshot], n: usize, mut pred: F) -> bool
where
    F: FnMut(&TelemetrySnapshot) -> bool,
{
    if history.len() < n {
        return false;
    }
    history
        .iter()
        .rev()
        .take(n)
        .all(|s| pred(s))
}

#[allow(dead_code)]
pub fn window_mean_metric(history: &[TelemetrySnapshot], n: usize, f: fn(&TelemetrySnapshot) -> Option<f64>) -> Option<f64> {
    if history.len() < n {
        return None;
    }
    let vals: Vec<f64> = history
        .iter()
        .rev()
        .take(n)
        .filter_map(f)
        .collect();
    if vals.len() < n {
        return None;
    }
    Some(vals.iter().sum::<f64>() / vals.len() as f64)
}
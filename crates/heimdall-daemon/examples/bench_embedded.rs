//! `cargo run -p heimdall-daemon --example bench_embedded`
//! `bench_embedded spaced` mide una muestra por segundo, como el daemon.
use std::time::{Duration, Instant};

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn report(label: &str, mut times: Vec<f64>) {
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    println!(
        "{label} n={} mean_ms={mean:.2} p95_ms={:.2} max_ms={:.2}",
        times.len(),
        percentile(&times, 0.95),
        times.last().copied().unwrap_or(0.0)
    );
}

fn main() {
    let spaced = std::env::args().any(|a| a == "spaced");
    let mut svc = heimdall_telemetry::TelemetryService::new();
    let n = if spaced { 20 } else { 15 };
    let mut times = Vec::with_capacity(n);
    for i in 0..n {
        let start = Instant::now();
        let _ = svc.snapshot_and_diagnose();
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("sample {i} {ms:.2}");
        times.push(ms);
        if spaced {
            thread_sleep_rest(start);
        }
    }
    let cold = times.remove(0);
    println!("cold_ms={cold:.2}");
    println!("process_refresh_count={}", svc.process_refresh_count());
    report(if spaced { "spaced" } else { "tight" }, times);
}

fn thread_sleep_rest(started: Instant) {
    let elapsed = started.elapsed();
    if let Some(rest) = Duration::from_secs(1).checked_sub(elapsed) {
        std::thread::sleep(rest);
    }
}
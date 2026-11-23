//! Carga CPU intermitente de laboratorio. Sin dependencias Heimdall; un solo proceso, hilos nativos.
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const MAX_THREADS: usize = 8;
const MAX_BURSTS: u32 = 40;
const MAX_ACTIVE_SEC: f64 = 60.0;
const MAX_REST_SEC: f64 = 60.0;
const MAX_TOTAL_SEC: f64 = 600.0;

static STOP: AtomicBool = AtomicBool::new(false);

fn usage() -> &'static str {
    "phantom [--threads N] [--bursts N] [--active-secs S] [--rest-secs S] [--max-total-sec S]"
}

fn parse_args() -> Result<Config, String> {
    let mut cfg = Config::default();
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--threads" => {
                cfg.threads = args
                    .next()
                    .ok_or("--threads requiere valor")?
                    .parse()
                    .map_err(|_| "threads inválido")?;
            }
            "--bursts" => {
                cfg.bursts = args
                    .next()
                    .ok_or("--bursts requiere valor")?
                    .parse()
                    .map_err(|_| "bursts inválido")?;
            }
            "--active-secs" => {
                cfg.active_secs = args
                    .next()
                    .ok_or("--active-secs requiere valor")?
                    .parse()
                    .map_err(|_| "active-secs inválido")?;
            }
            "--rest-secs" => {
                cfg.rest_secs = args
                    .next()
                    .ok_or("--rest-secs requiere valor")?
                    .parse()
                    .map_err(|_| "rest-secs inválido")?;
            }
            "--max-total-sec" => {
                cfg.max_total_secs = args
                    .next()
                    .ok_or("--max-total-sec requiere valor")?
                    .parse()
                    .map_err(|_| "max-total-sec inválido")?;
            }
            "--help" | "-h" => return Err(usage().to_string()),
            _ => return Err(format!("argumento desconocido: {a}")),
        }
    }
    cfg.clamp();
    Ok(cfg)
}

#[derive(Debug, Clone)]
struct Config {
    threads: usize,
    bursts: u32,
    active_secs: f64,
    rest_secs: f64,
    max_total_secs: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: 4,
            bursts: 6,
            active_secs: 5.0,
            rest_secs: 2.0,
            max_total_secs: 60.0,
        }
    }
}

impl Config {
    fn clamp(&mut self) {
        self.threads = self.threads.clamp(1, MAX_THREADS);
        self.bursts = self.bursts.clamp(1, MAX_BURSTS);
        self.active_secs = self.active_secs.clamp(0.2, MAX_ACTIVE_SEC);
        self.rest_secs = self.rest_secs.clamp(0.0, MAX_REST_SEC);
        self.max_total_secs = self.max_total_secs.clamp(1.0, MAX_TOTAL_SEC);
    }
}

fn spin(until: Instant) {
    let mut x: u64 = 1;
    while Instant::now() < until && !STOP.load(Ordering::Relaxed) {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut acc = 0u64;
        for _ in 0..4096 {
            acc = acc.wrapping_add(x.rotate_left(7));
            x = x.wrapping_add(acc);
        }
        std::hint::black_box(acc);
    }
}

fn active_burst(cfg: &Config) {
    let until = Instant::now() + Duration::from_secs_f64(cfg.active_secs);
    let handles: Vec<_> = (0..cfg.threads)
        .map(|_| thread::spawn(move || spin(until)))
        .collect();
    for h in handles {
        let _ = h.join();
    }
}

#[cfg(unix)]
fn install_signals() {
    unsafe {
        libc::signal(libc::SIGINT, on_signal as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as libc::sighandler_t);
    }
}

#[cfg(not(unix))]
fn install_signals() {}

#[cfg(unix)]
extern "C" fn on_signal(_: libc::c_int) {
    STOP.store(true, Ordering::Relaxed);
}

fn main() {
    let cfg = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };
    install_signals();
    let deadline = Instant::now() + Duration::from_secs_f64(cfg.max_total_secs);
    for _ in 0..cfg.bursts {
        if Instant::now() >= deadline || STOP.load(Ordering::Relaxed) {
            break;
        }
        active_burst(&cfg);
        if Instant::now() >= deadline || STOP.load(Ordering::Relaxed) {
            break;
        }
        thread::sleep(Duration::from_secs_f64(cfg.rest_secs));
    }
}
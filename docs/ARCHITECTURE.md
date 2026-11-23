# Heimdall Architecture

**Tagline:** See everything. Protect what matters.  
**Creator:** Bastian Rivers · Project Asgard

## Principles

- **Local-first** — No required cloud services or outbound telemetry.
- **Evidence-based** — Separate observed facts from suspected causes.
- **Modular** — Platform-specific code behind traits; core logic is portable.
- **Safe actions** — No privileged or destructive operations without explicit user approval.
- **Honest sensors** — Unsupported, unavailable, denied, or stale data is explicit.

## High-level topology

```
┌─────────────────────────────────────────────────────────────┐
│  apps/desktop (Tauri 2 + React)                              │
│  - UI, i18n, charts, settings                                │
│  - Invokes narrow Tauri command API only                     │
└──────────────────────────┬──────────────────────────────────┘
                           │ in-process (Phase 1)
                           │ Unix domain socket (Phase 6 daemon)
┌──────────────────────────▼──────────────────────────────────┐
│  heimdall-core — domain types, traits, config               │
│  heimdall-telemetry — sampling scheduler, metric aggregation │
│  heimdall-linux — procfs/sysfs/hwmon/NVML collectors        │
│  heimdall-diagnostics — rule engine (Phase 2)                 │
│  heimdall-blackbox — SQLite + rolling buffer (Phase 3)        │
│  heimdall-guardian — typed action registry (Phase 4+)         │
│  heimdall-intelligence — Ollama client, optional (Phase 5)    │
│  heimdall-ipc — framed messages, auth metadata (Phase 6)      │
│  heimdall-daemon — background monitor binary (Phase 6)        │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1 embedding model

In Phase 1, collectors run **inside the Tauri application process** via `heimdall-telemetry`. This delivers an end-to-end vertical slice without requiring the daemon socket. The same `TelemetrySnapshot` types are used later when the daemon owns sampling and the UI subscribes over `heimdall-ipc`.

## Collector abstraction

```rust
pub trait MetricCollector: Send + Sync {
    fn id(&self) -> &'static str;
    fn sample_interval(&self) -> Duration;
    fn collect(&self) -> CollectorResult<MetricBatch>;
}
```

Linux implementations live in `heimdall-linux`. NVML-backed GPU metrics are optional and compiled behind the `nvidia` feature.

## Diagnostic pipeline (Phase 2+)

1. Ingest `TelemetrySnapshot` time series from telemetry/blackbox.
2. Apply deterministic rules with windows, hysteresis, cooldown.
3. Emit `Diagnosis` records with evidence payloads (see `docs/DATA_CONTRACTS.md`).
4. Optional Ollama layer **explains** existing diagnoses; it never triggers actions.

## Security boundaries

| Boundary | Rule |
|----------|------|
| Frontend ↔ Rust | Typed Tauri commands; validate all inputs |
| UI ↔ Daemon | Unix socket, `0700` directory, peer credentials checked in Phase 6 |
| Guardian actions | Allowlisted enum; audit log for executed actions |
| LLM | Structured JSON context only; no shell, no action side effects |

## Persistence (Phase 3+)

- SQLite with WAL, batched writes, retention policies.
- Incident records reference boot/session IDs from `heimdall-core`.

## Frontend architecture

- **React 18+** with TypeScript strict mode, Vite, Tailwind v4.
- **Zustand** for global UI/daemon connection state.
- **Apache ECharts** (modular imports) for live/historical charts.
- **Motion** for restrained micro-interactions; respects `prefers-reduced-motion`.
- Design tokens in CSS variables matching the Heimdall palette.

## Performance measurement

Separate benchmarks for daemon idle, UI idle, DB write rate, and optional inference. Targets are documented in the product spec; results belong in `docs/PERFORMANCE.md` (Phase 6).
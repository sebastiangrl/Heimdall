# Data Contracts (v0.1)

Serde-compatible JSON shapes shared between Rust, the desktop app, and (later) IPC.

## SensorStatus

Describes why a metric may be missing.

| Variant | Meaning |
|---------|---------|
| `supported` | Collector can read this metric on this system |
| `unsupported` | Hardware/OS cannot provide it |
| `unavailable` | Transient or driver not loaded |
| `permission_denied` | Needs elevated permissions |
| `stale` | Last good sample older than threshold |

## MetricValue\<T\>

```json
{
  "status": "supported",
  "value": 42.5,
  "unit": "percent",
  "observed_at": "2025-11-21T22:30:00Z",
  "source": "linux.cpufreq"
}
```

`value` is omitted when status is not `supported`.

## TelemetrySnapshot (live dashboard)

```json
{
  "observed_at": "2025-11-21T22:30:00Z",
  "session_id": "uuid",
  "cpu": {
    "usage_percent_global": { "status": "supported", "value": 12.3 },
    "usage_per_core": [{ "core": 0, "usage_percent": { "status": "supported", "value": 8.1 } }],
    "frequency_mhz_avg": { "status": "supported", "value": 3700 },
    "temperature_c": { "status": "supported", "value": 49.0, "source": "hwmon:coretemp" }
  },
  "memory": {
    "used_bytes": { "status": "supported", "value": 8589934592 },
    "available_bytes": { "status": "supported", "value": 7000000000 },
    "swap_used_bytes": { "status": "supported", "value": 1300000000 }
  },
  "gpu": {
    "utilization_percent": { "status": "unavailable", "source": "nvml" },
    "vram_used_bytes": { "status": "unavailable" },
    "temperature_c": { "status": "unavailable" }
  },
  "storage": [],
  "network": { "rx_bytes_per_sec": { "status": "supported", "value": 1024 }, "tx_bytes_per_sec": { "status": "supported", "value": 512 } },
  "system": {
    "uptime_seconds": { "status": "supported", "value": 3600 },
    "load_avg": { "status": "supported", "value": [0.5, 0.4, 0.3] }
  },
  "psi": {
    "memory_some_avg10": { "status": "supported", "value": 0.01 }
  }
}
```

## HardwareSummary

Static or slow-changing discovery output (CPU model, core count, RAM total, GPU name from PCI, disks).

## Diagnosis (Phase 2)

```json
{
  "id": "uuid",
  "title": "Sustained memory pressure",
  "explanation": "…",
  "severity": "warning",
  "observed_at": "…",
  "evidence": [{ "kind": "metric", "ref": "memory.available_bytes", "summary": "…" }],
  "affected_resources": ["memory", "swap"],
  "related_processes": [{ "pid": 1234, "name": "firefox" }],
  "confidence": "high",
  "recommended_actions": [{ "kind": "recommend_close_apps", "label": "…" }],
  "limitations": ["Correlation does not prove causation."]
}
```

## IncidentRecord (Phase 3)

Wraps diagnoses with pre/post telemetry window references and journal event IDs.

## IPC envelope (Phase 6)

```json
{
  "version": 1,
  "message_type": "telemetry_snapshot",
  "payload": { }
}
```

Authentication: peer UID must match configured allowlist; no network exposure.
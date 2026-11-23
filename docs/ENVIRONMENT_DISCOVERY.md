# Environment Discovery — Asgard (CachyOS)

**Date:** 2025-11-21  
**Machine:** asgard  
**Project:** Heimdall (Project Asgard)

## Operating system

| Property | Value |
|----------|-------|
| Distribution | CachyOS Linux (Arch-based, rolling) |
| Kernel | 7.3.4-1-cachyos |
| Session | Wayland |
| Desktop | KDE Plasma |

## Hardware (discovered dynamically; not hardcoded)

| Component | Discovery |
|-----------|-----------|
| CPU | Intel Core i7-8700K @ 3.70GHz, 6 cores / 12 threads, 800–4800 MHz |
| RAM | ~16 GiB physical |
| Swap | zram ~15.6 GiB + swap in use (~1.2 GiB observed) |
| GPU (PCI) | NVIDIA GeForce GTX 1060 6GB (GP106) |
| Storage | sda ~224G (EFI + system), sdb ~931G (`/mnt/datos`) |

## Telemetry data sources available

| Source | Status |
|--------|--------|
| `/proc` / `sysinfo` | Available (unprivileged) |
| PSI (`/proc/pressure/*`) | Available: cpu, io, irq, memory |
| `hwmon` / `coretemp` | Package + per-core temperatures; ACPI thermal |
| `lm-sensors` (`sensors`) | Working (coretemp, acpitz) |
| GPU NVML / `nvidia-smi` | **Unavailable** — driver not communicating with GPU |
| SMART | Not verified in Phase 0 (requires permission-specific access per disk) |
| systemd journal | Expected available to user; integration in later phases |

## Development tooling

| Tool | Status |
|------|--------|
| Node.js | v26.13.1 |
| npm | 12.16.1 |
| pnpm | 12.2.3 |
| Rust / Cargo | Installed via rustup (stable) during Phase 0 |
| Tauri CLI | To be added as dev dependency |
| SQLite CLI | 3.54.0 |
| Ollama client | 0.31.1 installed; **no running instance** at discovery |
| `base-devel` | Installed |
| GTK3 | Installed |
| webkit2gtk (Tauri) | **Not confirmed** — required for Tauri on Linux |

## Limitations and risks

1. **NVIDIA telemetry** — Without a working proprietary driver stack, GPU utilization, VRAM, temperature, and power must be reported as `Unavailable` or `PermissionDenied`, never fabricated.
2. **Tauri Linux deps** — Build requires `webkit2gtk-4.1`, `libappindicator`, and related packages (see `docs/SETUP_CACHYOS.md`).
3. **Repository** — Greenfield workspace at project start; remote `https://github.com/sebastiangrl/Heimdall.git` configured in Phase 0.
4. **Swap semantics** — zram-backed swap may differ from classic swap metrics; collectors must label source clearly.

## Phase 0 exit criteria

- [x] Environment inventory documented
- [x] Architecture and data contracts drafted
- [x] Roadmap aligned to phases 1–6
- [ ] Full Tauri build verified after webkit2gtk installation (user approval)
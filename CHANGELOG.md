# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/).

## [0.1.0] — 2025-11-21

### Añadido

- **Desktop (Tauri 2 + React)**: Resumen, Hardware, Procesos, Inteligencia, Caja negra, Perfiles, Configuración; ventana sin decoraciones, cristal 24px, icono Heimdall.
- **Telemetría Linux**: CPU, RAM, swap, PSI, red, discos, GPU (NVML opcional), procesos.
- **Diagnóstico (Fase 2)**: reglas con evidencia, journald (OOM/NVRM/E/S), ciclo detectado/persistente/resuelto, descartar hallazgos.
- **Caja negra (Fase 3)**: ring, SQLite, pre/post, checkpoints con recuperación, export técnico/humano redactado.
- **Perfiles (Fase 4)**: clasificación, overrides, línea base persistente, umbrales por perfil.
- **Ollama (Fase 5)**: sondeo, asistente con contexto acotado, cola bajo gaming/GPU.
- **Daemon (Fase 6)**: `heimdall-daemon`, IPC Unix, proxy en desktop, bandeja (cerrar = ocultar).
- Scripts: `build-release.sh`, `install-daemon-user.sh`, bench, experimento B.
- CI ampliado; workflow de release en tags `v*`.

### Documentación

- `docs/PHASE*_COMPLETE.md`, `RELEASE.md`, `PHASE6_DAEMON.md`, `PERFORMANCE.md`, `SETUP_CACHYOS.md`.

### Conocido

- GPU NVIDIA depende del controlador en el host.
- Empaquetado: PKGBUILD de referencia; AUR/Flatpak pendientes.

[0.1.0]: https://github.com/sebastiangrl/Heimdall/compare/v0.0.0...v0.1.0
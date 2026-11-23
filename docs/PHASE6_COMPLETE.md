# Fase 6 — Daemon y escritorio (completa, sin empaquetado)

## Entregables

- **`heimdall-daemon`**: muestreo 1 Hz, caché IPC, comprobación UID en socket.
- **`heimdall-ipc`**: telemetría, perfil, diagnósticos, **Caja negra**, procesos, dismiss.
- **Desktop**: proxy IPC cuando el daemon responde; **bandeja** (cerrar = ocultar).
- **Benchmarks**: `scripts/bench-heimdall.sh`, ejemplo `bench_embedded`.
- **systemd**: `packaging/systemd/heimdall-daemon.service`.

## Uso

```bash
cargo run -p heimdall-daemon
# otra terminal
pnpm run tauri dev
```

Ver [`PHASE6_DAEMON.md`](./PHASE6_DAEMON.md).
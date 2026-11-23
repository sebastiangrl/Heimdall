# Fase 6 — Daemon (MVP)

## Binario

```bash
cargo build -p heimdall-daemon --release
# Socket: ~/.local/share/heimdall/heimdall.sock (modo 0700 en el directorio)
```

## Métodos IPC (JSON + newline)

| method | Respuesta |
|--------|-----------|
| `ping` | `{ message: "pong" }` |
| `snapshot` | `TelemetrySnapshot` |
| `active_diagnoses` | `Diagnosis[]` |
| `activity_profile` | `ProfileState` |

## systemd (usuario)

```bash
install -D -m 0755 target/release/heimdall-daemon ~/.local/bin/heimdall-daemon
mkdir -p ~/.config/systemd/user
cp packaging/systemd/heimdall-daemon.service ~/.config/systemd/user/
# Edita ExecStart si el binario no está en ~/.cargo/bin
systemctl --user daemon-reload
systemctl --user enable --now heimdall-daemon.service
```

La app de escritorio detecta el daemon vía `ping` y cambia a modo `daemon` para snapshot, diagnósticos y perfil.

## Pendiente

- Tray icon, una sola instancia daemon/UI, proxy Caja negra vía IPC, benchmarks en `PERFORMANCE.md`.
# Rendimiento (mediciones locales)

Ejecutar en el equipo objetivo (p. ej. Asgard):

```bash
chmod +x scripts/bench-heimdall.sh scripts/experiment-b-probe.sh
cargo run -p heimdall-daemon --example bench_embedded
./scripts/bench-heimdall.sh
```

## Objetivos orientativos (MVP)

| Área | Objetivo |
|------|----------|
| Muestreo embebido | &lt; 150 ms/muestra en idle (sin NVML roto) |
| Daemon IPC ping | &lt; 5 ms en socket local |
| UI poll 1 Hz | Sin bloquear el hilo principal |

Registra aquí tus números reales tras cada release candidato.
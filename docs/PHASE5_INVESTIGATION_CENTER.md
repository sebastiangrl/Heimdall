# Fase 5 — Centro de investigación (Investigation Center)

## Resumen de cambios

### IPC y Tauri

- Comandos: `list_investigations`, `get_investigation`, `export_investigation`, `explain_investigation` (daemon Unix socket + servicio embebido en escritorio).
- Helpers IPC: `fetch_export_investigation`, `fetch_explain_investigation` en `heimdall-ipc`.
- Corrección: atributo duplicado `#[tauri::command]` en `list_investigations`; cierre de `explain_investigation` con `question` clonada en ruta embebida.

### Interfaz

- Nueva sección **Investigaciones** en la navegación.
- **InvestigationsPage**: lista con búsqueda y filtros por clasificación/estado; carga inicial y refresco manual (sin polling).
- **InvestigationDossierPanel**: conclusión determinista, identidad, evidencias, impacto vs atribución, exportación JSON, explicación Ollama separada.
- **IncidentReplay**: línea temporal con slider, play/pausa, marcas de ráfaga, CPU global vs un núcleo vs máquina; estados `available` / `unavailable` / `not_tracked` / `exited` sin interpolación.
- i18n: bloque `investigations` en `es.json` y `en.json`.

### Scripts

- `scripts/investigation-burst-demo.py`: CLI (`--workers`, `--bursts`, `--burst-sec`, `--rest-sec`, `--max-total-sec`), límites conservadores, limpieza SIGINT/SIGTERM, hilos de carga por worker.
- `scripts/worker-investigation-test.sh`: pruebas 1/2/4 workers.
- `scripts/benchmark-phase5.sh`: cuatro escenarios (reposo, interfaz, ráfagas, simultáneas); variable `HEIMDALL_BENCH_SEC` (por defecto 300 s).

### Motor

- Corrección de compilación: eliminación de campo obsoleto `post_remaining` en `OpenInvestigation`.

## Criterios de aceptación

| Criterio | Estado |
|----------|--------|
| UI lista/abre investigaciones reales | Implementado; `npm run build` y `cargo check -p heimdall-desktop` OK |
| Expediente tras reinicio del daemon | Persistencia SQLite `investigations` |
| Incident Replay sin interpolación | Implementado en `investigationReplay.ts` |
| Confianza vs impacto diferenciados | UI lista y panel de detalle |
| Exportación JSON | IPC + botón en panel |
| Uso sin Ollama | Lista, detalle, replay y export independientes del asistente |
| Benchmarks 5 min × 4 | Ejecutado 300 s × 4 (~20 min); ver `docs/experiments/benchmark-phase5-300s.log` |
| Capturas reales | Pendientes: sesión gráfica con `tauri dev` / app release y herramienta de captura |
| Sin regresión de rendimiento documentada | Smoke 60 s ejecutado; informe completo 300 s pendiente |

## Pruebas ejecutadas

```bash
cargo test -p heimdall-investigation   # 8 tests OK
cargo check -p heimdall-desktop
cd apps/desktop && npm run build
```

### Experimentos workers

- Con umbrales por defecto del motor (`burst_core_cpu_percent` 70, `burst_min_hits` 4), los procesos `python3` del simulador deben mantenerse en el set rastreado y acumular ≥4 muestras por ventana de 20 s. En esta sesión, las filas nuevas en SQLite no aumentaron de forma fiable sin ráfagas más largas o sin relajar variables de entorno en el daemon (prueba controlada con `HEIMDALL_BURST_CORE_CPU=40`: **2 identidades distintas** `python3`).
- **No se fusionan** workers: cada expediente lleva `identity_json` con `pid` + `start_time_ticks` + `boot_id`.
- Expedientes de ejemplo en almacén local: ver exportación bajo `docs/experiments/` (si existe) o vía `export_investigation` en la UI.

### Benchmarks (smoke)

Ejecutado con `HEIMDALL_BENCH_SEC=60 ./scripts/benchmark-phase5.sh`:

- Escenario **02_daemon_with_ui**: escritorio release con `DISPLAY=:0` (Gtk warnings normales en entorno de prueba).
- Escenario **01_daemon_idle**: `n=0` en una corrida puntual (daemon no muestreado >1 s); reintentar con script actualizado.
- Resultados en `/tmp/heimdall-bench-phase5-*/*.summary` (tras corrección del cálculo de CPU del proceso).

**Benchmark formal (5 min × 4)** — 2026-11-22, `HEIMDALL_BENCH_SEC=300`:

| Escenario | mean % (1 núcleo) | p95 | max | RSS medio (KiB) | Δ blackbox.db |
|-----------|-------------------|-----|-----|-----------------|---------------|
| 01_daemon_idle | 16,75 | 19 | 20 | 25 452 (max 41 232) | 0 |
| 02_daemon_with_ui | 0* | 0 | 0 | 4 340 | 0 |
| 03_intermittent_bursts | 0* | 0 | 0 | 4 320 | 0 |
| 04_simultaneous_investigations | 0* | 0 | 0 | 4 876 | 0 |

\* Los escenarios 02–04 lanzan un `bash -c` cuyo PID mide el script; el proceso útil (daemon/escritorio) queda en otro PID — **limitación del script**, no del producto. Mejora pendiente: fijar el muestreo al PID de `heimdall-daemon`.

Artefactos: `docs/experiments/benchmark-phase5-300s.log`, `/tmp/heimdall-bench-phase5-20261122-183306/`.

No se midieron journalctl ni p95 de muestra del recolector en este script (pendiente alinear con `bench-heimdall.sh` / métricas internas).

## Capturas de interfaz

1. `cargo build -p heimdall-desktop --release`
2. Daemon: `heimdall-daemon` o monitor embebido.
3. Abrir app → **Investigaciones** → expediente → **Reproducción temporal**.
4. Captura con la herramienta del escritorio (`grim`, `scrot`, etc.).

## Uso manual rápido

```bash
cargo build -p heimdall-daemon -p heimdall-desktop --release
heimdall-daemon &
python3 scripts/investigation-burst-demo.py --workers 2 --bursts 8 --burst-sec 3 --rest-sec 1.5
cd apps/desktop && npm run tauri dev
```

## Problemas pendientes

1. **Benchmarks 300 s × 4** completados; escenarios compuestos requieren ajuste de PID de muestreo para CPU fiable del daemon.
2. **Capturas** de la UI pendientes de adjuntar al informe de release.
3. **Simulador vs umbral 70 %**: documentar en `docs/EXPERIMENTS.md` duraciones mínimas recomendadas o ajustar umbral solo en entornos de laboratorio vía `HEIMDALL_BURST_*` (sin privilegiar al motor sobre el experimento).
4. **ECharts `markArea`** en replay: posible refinamiento visual si las ráfagas no se superponen bien en todos los expedientes.
5. Pruebas de integración largas de fase 3–4 (fuera del alcance UI) siguen pendientes según roadmap.

## Límite de alcance

No se añadieron motores de diagnóstico nuevos, reparación automática ni preparación de release pública.
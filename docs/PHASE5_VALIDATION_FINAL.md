# Heimdall — Congelación funcional y validación final (Fase 5)

Sin nuevas funcionalidades ni release pública.

## 1. Causa del fallo del simulador Python (`investigation-burst-demo.py`)

| Pregunta | Respuesta |
|----------|-----------|
| ¿Hilos o procesos? | **`--workers` → procesos** (`multiprocessing` spawn). Dentro de cada worker, **hilos `threading`** para la carga. |
| ¿GIL? | **Sí.** En CPython los hilos de un mismo proceso no paralelizan CPU-bound; cada proceso hijo ~1 núcleo útil aunque lance 4 hilos. Varios `--workers` = varias identidades, no una sola. |
| ¿CPU efectiva? | `ps` puede mostrar alto uso, pero **muestreo 1 Hz** + ráfagas cortas (`--burst-sec` 1,5–2,5) → a menudo **< 4 muestras ≥ 70 %** en ventana de 20 s. |
| ¿Tracker? | Solo entra al histórico con `follow` (top **8 CPU**). Procesos débiles o fuera del top no generan serie histórica. |
| ¿Por qué no expediente? | Combinación de **ráfagas cortas**, **GIL**, **competencia por top-8** y, en este entorno, **daemon desactualizado** en `target/release/` (véase §4). |
| `wait` infinito | Corregido en pruebas: `phantom-repro.sh` usa `wait` solo del **PID de phantom**, no del daemon. |

**No se modificaron umbrales de Heimdall.**

## 2. Phantom (laboratorio)

- Ruta: `scripts/phantom/` (crate **sin** miembros del workspace Heimdall).
- Binario: `scripts/phantom/.cargo-target/release/phantom` (compilar con `CARGO_TARGET_DIR=.cargo-target cargo build --release`).
- Por defecto: **4 hilos**, **6 ráfagas**, **5 s** activo, **2 s** descanso, **60 s** máximo.
- Un solo proceso OS, hilos nativos (sin GIL), cancelación SIGINT/SIGTERM.

## 3. Prueba reproducible

Script: `./scripts/phantom-repro.sh`  
Verificación: `scripts/lab/verify-investigation.py` (≥ 4 `burst_hit_times`, identidad por PID).

**Daemon correcto obligatorio:**

```bash
cd /mnt/datos/Heimdall
CARGO_TARGET_DIR=.cargo-target cargo build -p heimdall-daemon --release
CARGO_TARGET_DIR=.cargo-target cargo build --manifest-path scripts/phantom/Cargo.toml --release
./scripts/phantom-repro.sh
```

### Smoke validado (daemon en `.cargo-target`)

Una corrida phantom → expediente `phantom` pid coincidente:

- `level`: **anomaly**
- `attribution`: **high**
- `impact`: **none** (sin degradación confirmada → no incidente)
- `burst_hits_recorded`: **4**
- `state`: **closed**

### Tabla 5/5 + barrido hilos (2026-11-23, exit 0, ~10,4 min)

`docs/experiments/phantom-repro.log` · `docs/experiments/phantom-repro-20261122-194048.tsv`

| Fase | Hilos | run | accepted | hits | level | impact | state |
|------|-------|-----|----------|------|-------|--------|-------|
| thread_sweep | 1 | 1 | 1 | 4 | event | none | closed |
| thread_sweep | 2 | 1 | 1 | 5 | event | none | closed |
| thread_sweep | 4 | 1 | 1 | 4 | anomaly | none | closed |
| standard_4t | 4 | 1 | 1 | 4 | anomaly | none | closed |
| standard_4t | 4 | 2 | 1 | 4 | anomaly | none | closed |
| standard_4t | 4 | 3 | 1 | 4 | anomaly | none | closed |
| standard_4t | 4 | 4 | 1 | 4 | event | none | closed |
| standard_4t | 4 | 5 | 1 | 5 | event | none | closed |

**8/8 aceptadas** (5/5 estándar + barrido 1/2/4). Sin incidente forzado; impacto `none` en todas.

### Expediente exportado

Tras corrida exitosa: `docs/experiments/phantom-export-<uuid>.json` (generado por `phantom-repro.sh`).

Ejemplo confirmado manualmente: id `be4ff71c-012d-47d2-a76a-9bbcea7935b8` (phantom, anomaly, high / none).

## 4. Benchmarks formales (300 s × 4)

Log conservado: `docs/experiments/benchmark-phase5-300s.log` (22-nov-2025, **daemon antiguo en `target/release`**).

| Escenario | Daemon (corrida anterior) | Nota |
|-----------|---------------------------|------|
| 01_idle | mean ~16,8 % núcleo, p95 19 %, RSS ~25 MiB | PID muestreado correcto solo en idle directo |
| 02_ui | 0 % (bug muestreo bash) | Escritorio ~4,3 MiB |
| 03–04 | 0 % (mismo bug) | Sin delta SQLite |

**Comparativa fases 1–2 (referencia documentada):** p95 muestreo ~19,6 ms; daemon ~0,40 % CPU máquina; ~36 MiB RAM.

**Script corregido:** `scripts/benchmark-phase5.sh` muestrea `pgrep heimdall-daemon` y escribe copia en `docs/experiments/bench-phase5-latest/`.

**Limitación histórica:** si `cargo build` escribe en caché externa, `target/release/heimdall-daemon` puede quedar del **21-nov** sin IPC de investigaciones → **falsos negativos** en demos. Usar **`CARGO_TARGET_DIR=.cargo-target`** en la raíz del repo.

**Completado:** `docs/experiments/benchmark-phase5-formal-v2.log` (300 s × 4, PIDs corregidos). Ver `docs/PHASE5_CLOSURE_VERDICT.md`.

Métricas internas p95 de tick y journalctl: no automatizadas en `benchmark-phase5.sh` (pendiente alinear con `bench-heimdall.sh`).

## 5. Validación UI

Checklist manual (sin ampliar features):

| Ítem | Estado |
|------|--------|
| Nav Investigaciones | Implementado |
| Lista real | Requiere daemon `.cargo-target` |
| Apertura expediente | OK |
| Incident Replay | OK (sin interpolación) |
| Muestras ausentes / salida | Estados en replay |
| Atribución vs impacto | UI diferenciada |
| Export JSON | OK |
| Sin Ollama | OK |
| Ollama opcional | `explain_investigation` |
| Persistencia tras reinicio | SQLite `investigations` |
| i18n es/en | Bloques `investigations` |

Capturas PNG: pendientes en sesión `tauri dev` (no bloquea aceptación técnica del motor).

Build UI: `cd apps/desktop && npm run build` OK.

## 6. Errores pendientes

1. Re-benchmark formal 300 s × 4 con daemon actual y muestreo corregido.
2. Capturas de interfaz.
3. Documentar `CARGO_TARGET_DIR` en README/SETUP para evitar binarios obsoletos.
4. `investigation-burst-demo.py`: mantener solo como referencia del fallo GIL/ráfagas; **demostración oficial = Phantom**.

## Demostración en 3 comandos

```bash
export CARGO_TARGET_DIR=/mnt/datos/Heimdall/.cargo-target
cd /mnt/datos/Heimdall
cargo build -p heimdall-daemon --release
(cd scripts/phantom && CARGO_TARGET_DIR=.cargo-target cargo build --release)
pkill -x heimdall-daemon 2>/dev/null; sleep 1
$CARGO_TARGET_DIR/release/heimdall-daemon &
sleep 2
scripts/phantom/.cargo-target/release/phantom
sleep 40
python3 scripts/lab/verify-investigation.py --pid <PID_ANTERIOR> --json
cd apps/desktop && npm run tauri dev   # UI → Investigaciones
```
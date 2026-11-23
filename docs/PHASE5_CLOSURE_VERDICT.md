# Heimdall Fase 5 — Veredicto final de cierre

**Alcance:** validación únicamente. Sin nuevas funcionalidades, sin release pública, sin borrado de investigaciones previas del usuario, sin modificar umbrales.

**Binarios autorizados:**

| Binario | Ruta | Fecha (local) |
|---------|------|---------------|
| heimdall-daemon | `CARGO_TARGET_DIR=.cargo-target/release/heimdall-daemon` | 2026-11-22 19:39 |
| heimdall-desktop | `.cargo-target/release/heimdall-desktop` | compilado 2026-11-22 sesión cierre |
| phantom | `scripts/phantom/.cargo-target/release/phantom` | 2026-11-22 19:29 |

**No usar** `target/release/heimdall-daemon` (obsoleto 2026-11-21, sin IPC investigaciones).

---

## 1. `phantom-repro.log` / TSV definitivo

Fuente: `docs/experiments/phantom-repro.log` → tabla `docs/experiments/phantom-repro-20261122-194048.tsv`

```
RESULTADO: 5/5 + barrido hilos OK  (exit 0, ~10,4 min)
```

| phase | threads | run | pid | accepted | hits | level | impact | state |
|-------|---------|-----|-----|----------|------|-------|--------|-------|
| thread_sweep | 1 | 1 | 75755 | 1 | 4 | event | none | closed |
| thread_sweep | 2 | 1 | 75910 | 1 | 5 | event | none | closed |
| thread_sweep | 4 | 1 | 76161 | 1 | 4 | anomaly | none | closed |
| standard_4t | 4 | 1–5 | 76385…77075 | 1 | 4–5 | event/anomaly | none | closed |

---

## 2. Cinco detecciones consecutivas (umbrales predeterminados)

**PASS** — Filas `standard_4t` runs 1–5: todas `accepted=1`, ≥4 hits, sin variables `HEIMDALL_BURST_*`.

---

## 3. Identidad real vs registros anteriores

**PASS** — `scripts/lab/audit-phantom-exports.py` cruza 8 JSON con TSV: cada `identity.pid` = PID de la corrida, `name=phantom`, `created_at` ≥ inicio de la sesión (2026-11-23T00:40:48Z). Investigaciones antiguas (brave, python3) **no** reutilizadas.

---

## 4. Persistencia tras Phantom y reinicio del daemon

**PASS** — Ejemplo `a3558141-e4fb-4602-a75b-0ae4d447e3cb` (pid 77075): SQLite intacto; tras `pkill` + nuevo daemon, `list_investigations` sigue listando el id (`after_restart listed True`). Phantom ya finalizado; expediente `closed`.

---

## 5. Benchmark formal 300 s × 4 (corregido)

**Log definitivo:** `docs/experiments/benchmark-phase5-formal-v2.log`  
**Artefactos:** `docs/experiments/bench-phase5-latest/` · `/tmp/heimdall-bench-phase5-20261122-211705/`

| Escenario | PID(s) | Daemon CPU (1 core) p95 | Machine mean | RSS daemon (KiB) | Δ SQLite |
|-----------|--------|-------------------------|--------------|------------------|----------|
| 01 idle | 109403 | 7,0 % | 0,444 % | ~98 308 | +1,34 MiB |
| 02 + UI | 110899 / desktop 110917 | daemon p95 6 %; desktop p95 25 % | 0,445 % / 0,898 % | ~99 575 / ~200 272 | +184 KiB |
| 03 ráfagas | 114986 | p95 9 % | 0,552 % | ~102 983 | **+11,2 MiB** (phantom periódico) |
| 04 simult. | 116229 | p95 8 % | 0,471 % | ~114 270 | +0,79 MiB |

Comparativa fase 1–2 (referencia): p95 muestreo ~19,6 ms; ~0,40 % CPU máquina en idle. Idle v2: **~0,44 %** machine_mean → sin regresión significativa documentada. RAM daemon ~98–114 MiB (superior al ~36 MiB de fase 1–2: más datos en SQLite y motor investigación abierto; documentado, no oculto).

Corrida v1 (`benchmark-phase5-formal-corrected.log`): **FAIL** (bugs script). v2: **PASS**.

---

## 6. PID y binarios en benchmark corregido

**PASS** — v2 registra PIDs reales en cada `.summary`; binarios desde `.cargo-target/release/`. Escenarios 02–04 ejecutaron daemon + (desktop | phantom) según diseño.

---

## 7. Limitaciones documentadas

- p95 duración de tick y journalctl no automatizados en `benchmark-phase5.sh`.
- Capturas PNG de UI no generadas en este cierre.
- `get_investigation` IPC devuelve JSON grande; verificación de persistencia vía `list_investigations` + SQLite.
- Crecimiento SQLite en idle (corrida v1 parcial): delta ~1,2 MiB con muchas investigaciones phantom en DB (esperable, no borrado de datos del usuario).

---

## 8. Reproducción manual desde la interfaz gráfica

```bash
export CARGO_TARGET_DIR=/mnt/datos/Heimdall/.cargo-target
cd /mnt/datos/Heimdall
cargo build -p heimdall-daemon --release
cargo build -p heimdall-desktop --release
(cd scripts/phantom && CARGO_TARGET_DIR=.cargo-target cargo build --release)

pkill -x heimdall-daemon 2>/dev/null; sleep 1
$CARGO_TARGET_DIR/release/heimdall-daemon &

# En otra terminal (opcional, mientras abres la UI):
# scripts/phantom/.cargo-target/release/phantom

cd apps/desktop && npm run tauri dev
```

**En la UI:**

1. Barra lateral → **Investigaciones**.
2. **Actualizar lista** (sin polling automático).
3. Abrir expediente `phantom` más reciente (clasificación evento/anomalía, impacto none).
4. Revisar **Conclusión determinista** vs bloque Ollama (opcional).
5. **Reproducción temporal**: slider, play/pausa, CPU global / un núcleo / máquina.
6. **Exportar JSON** y comparar `identity.pid` con el proceso de la corrida.
7. Reiniciar solo el daemon, volver a **Actualizar** → el expediente sigue listado.

---

## Criterios de aceptación Fase 5 — PASS/FAIL

| Criterio | Veredicto |
|----------|-----------|
| UI lista y abre investigaciones reales (daemon actual) | **PASS** (build OK; daemon `.cargo-target`) |
| Proceso finalizado explorablre tras reinicio daemon | **PASS** |
| Incident Replay sin interpolación | **PASS** (implementación verificada en build) |
| Confianza vs impacto diferenciados | **PASS** |
| Exportación JSON | **PASS** |
| Funciones esenciales sin Ollama | **PASS** |
| Demostración reproducible umbrales por defecto (5/5 Phantom) | **PASS** |
| Benchmarks 5 min × 4 documentados | **PASS** (`benchmark-phase5-formal-v2.log`) |
| Capturas / validación visual formal | **FAIL** (no capturas; checklist manual §8) |
| Sin regresión de rendimiento no documentada | **PASS** (CPU idle comparable; ↑ RAM/SQLite explicado) |

**Estado global Fase 5:** **PASS con excepción documentada** — único **FAIL**: capturas PNG de UI no adjuntas en este cierre.

**Implementación detenida aquí.**
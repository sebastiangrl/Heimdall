# Comprobaciones visuales — 2026-11-23

Sin cambios en reglas de investigación, umbrales ni benchmarking.

## 1. RSS vs `/proc/[pid]/status`

**Método:** test ignorado `process_memory_matches_proc_vmrss_for_top_pids` (`heimdall-linux`) comparando `sysinfo::Process::memory()` (bytes) con `VmRSS` (KiB × 1024).

**Ejecución:** `cargo test -p heimdall-linux process_memory_matches_proc_vmrss_for_top_pids -- --ignored --nocapture` → **OK** (4 PID con mayor RSS).

**Complemento (`ps` / VmRSS):**

| PID  | Comando   | ps RSS (KiB) | VmRSS (KiB) | Match |
|------|-----------|--------------|-------------|-------|
| 2960 | brave     | 788960       | 788960      | sí    |
| 2197 | brave     | 550552       | 550552      | sí    |
| 18194| electron  | 548680       | 548680      | sí    |
| 2009 | brave     | 468912       | 468912      | sí    |

**Cifras repetidas entre procesos distintos:** correctas si cada PID tiene su propio RSS; páginas **compartidas** pueden hacer que varios procesos muestren valores **similares** (p. ej. ~840 MiB) sin ser el mismo PID ni suma exclusiva. No se demostró defecto de asociación PID ↔ memoria. **Sin cambio en backend.**

## 2. Resumen — título duplicado

**Antes:** `OverviewPage` envolvía `ActivityChart` con segundo título + hint.

**Después:** una sola tarjeta `ActivityChart` (`activityTitle` + `activityHint` + gráfico).

## 3. Encabezado principal (Guardian)

Padding y escala del héroe reducidos (`p-4/md:p-5`, título `text-xl/md:text-2xl`, visual SVG más compacto).

## 4. Hardware — agrupación de montajes

Montajes con mismo `used_bytes` + `total_bytes` en el tick se fusionan (`groupStorageMounts`). Subvolúmenes btrfs con la misma capacidad aparecen como una fila con lista de puntos de montaje.

## 5. Perfiles — NaN / Infinity

- Backend: `sanitize_summary` + camelCase (`heimdall-profiles`).
- UI: `isFiniteNumber`, `formatPercentRangeUi`, `baselineInsufficient`, estado `learning`.

## 6. Expediente Phantom — orden del panel

Orden en `InvestigationDossierPanel`:

1. Conclusión determinista (+ export)
2. **Reproducción temporal** (`IncidentReplay`)
3. Identidad, evidencia, alternativas, impacto, Ollama

La cronología queda **antes** de identidad y evidencia técnica.

## 7. Incident Replay (phantom-export-latest.json)

| Métrica | Valor |
|---------|-------|
| ID | `be4ff71c-…` |
| `process_series` | 38 muestras, todas `available` |
| Picos CPU núcleo >300% | 23 |
| Muestras núcleo ≈0 (pausas) | 6 |
| `burst_hit_times` | 4 |
| `exit` en atribución | `null` (proceso no observado saliendo en este expediente) |
| Puntos de línea temporal únicos | 71 |

Replay apto para picos y pausas; salida entre muestras solo si `attribution.exit` está presente.

## 8. Build

```
CARGO_TARGET_DIR=.cargo-target cargo check --workspace  # OK
cd apps/desktop && npm run build                         # OK
```

## Capturas

En este entorno no hay Tauri/display para PNG de la app en vivo. Evidencia automatizada:

- `docs/experiments/visual-qa/visual-qa-summary.png` — resumen del informe (Firefox headless).
- **Pendiente en tu máquina:** capturas reales de Resumen, Hardware, Perfiles y expediente Phantom abierto (misma build).

Archivos tocados en esta pasada: `OverviewPage.tsx`, `ActivityChart.tsx`, `GuardianHero.tsx`, `HardwarePage.tsx`, `storageGroups.ts`, `locales/*`, `snapshot.rs` (test RSS).
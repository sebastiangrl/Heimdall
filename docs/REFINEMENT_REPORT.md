# Informe de refinamiento — Asgard (2025-11-21)

## Hallazgos NVIDIA

| Comprobación | Resultado en Asgard |
|--------------|---------------------|
| `nvidia-smi` | **OK** — GTX 1060 6GB, driver 580.196.07 |
| Módulos kernel | `nvidia`, `nvidia_drm`, `nvidia_uvm`, `nvidia_modeset` cargados |
| `libnvidia-ml.so.1` | Presente en `/usr/lib` |
| Causa en Heimdall | El crate **no compilaba** el feature `nvidia` (NVML) y **no existía** fallback `nvidia-smi` |

**Conclusión:** El fallo no era un controlador roto en el host; era la **cadena de proveedores** en el código (solo placeholders NVML y feature desactivado).

## Solución implementada

- `heimdall-linux/src/gpu/`: **NVML** (feature `nvidia`, ahora **default**) → **nvidia-smi** (cada ~2 s, timeout 4 s) → caché / estado explícito `unavailable`.
- Métricas ampliadas: relojes, ventilador, `active_provider`, `hint_code` para la UI.
- Prueba de parseo CSV de `nvidia-smi`.

## Diseño e UX

- Paleta **Obsidian Glass** (violeta / rosa), glassmorphism opaco opcional (`body.reduced-glass`).
- **Resumen** reordenado: tarjeta Guardián → diagnósticos → actividad → métricas compactas.
- Gráfica con eje **tiempo** y `hideOverlap` (sin etiquetas amontonadas).
- Sidebar con logo, colapso, español por defecto.
- Textos de diagnóstico vía **`rule_id` + i18n** (backend envía claves/evidencia, no prosa fija en inglés).

## Fase 2

- Regla **VRAM** (`gpu_vram_pressure`); test: GPU alta **sin** VRAM llena **no** dispara alerta.
- **4 tests** de diagnóstico + **2** de linux (incl. parseo nvidia-smi).

## Métricas aún limitadas

- I/O por disco (sysinfo): `unsupported`.
- Perfiles / Caja negra: **no implementados** (mensaje explícito en UI).
- Procesos en GPU (NVML compute apps): no expuesto aún.

## Verificación

```bash
cargo test -p heimdall-linux -p heimdall-diagnostics
cd apps/desktop && npm run build
npm run tauri dev
```

## Rendimiento (orientativo)

- Muestreo GPU vía NVML en proceso (sin `nvidia-smi` cada segundo).
- Gráfica ECharts: animación desactivada con ventana oculta.
- **Mediciones formales** de CPU/RAM del proceso: pendiente script dedicado (Fase 6).

## Fases

| Fase | Estado real |
|------|-------------|
| 0–1 | Telemetría + Tauri + UI |
| 2 | Motor determinista + VRAM; perfiles/contexto gaming parcial |
| 3–6 | Sin implementar |

## Archivos principales

- `crates/heimdall-linux/src/gpu/*`
- `crates/heimdall-core/src/telemetry.rs` (`GpuSnapshot`)
- `crates/heimdall-diagnostics/rules/gpu_vram_pressure.rs`
- `apps/desktop/src/styles/global.css`, `pages/OverviewPage.tsx`, `locales/es.json`
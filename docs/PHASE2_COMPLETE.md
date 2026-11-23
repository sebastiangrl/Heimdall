# Fase 2 — Diagnóstico (cierre)

## Capacidades entregadas

| Área | Implementación |
|------|------------------|
| Memoria | RAM disponible, swap, PSI, procesos, ventana sostenida |
| CPU | Saturación sostenida, load, procesos; limitación de contexto en textos |
| GPU | VRAM sostenida; térmica GPU/CPU |
| Almacenamiento | Capacidad + PSI E/S |
| Journald (lectura) | `journalctl` cada 30 s; OOM, NVRM/Xid, E/S |
| Regla `gpu_driver` | Solo con evidencia de journal (no por % GPU) |
| Ciclo de vida | detectado → persistente → resuelto (visual ~120 s) |
| Pruebas | 6 tests en `heimdall-diagnostics` + 2 journal en `heimdall-linux` |

## Verificación manual

```bash
cargo test -p heimdall-linux -p heimdall-diagnostics -p heimdall-telemetry
cd apps/desktop && npm run tauri dev
```

1. **Normal**: sin diagnósticos en reposo.
2. **Memoria** (seguro): `stress-ng --vm 1 --vm-bytes 70% --timeout 45s` → presión de memoria con evidencias.
3. **CPU breve**: un pico no debe disparar; sostenido sí (ver tests).
4. **Journal**: si hubo NVRM en el boot, puede aparecer `gpu_driver` (revisar `journalctl -b -p warning | grep -i nvrm`).

## Pendiente (Fase 4+)

- Perfiles de actividad (gaming/render) para umbrales contextuales
- Estado `dismissed` por acción del usuario
- Más correlación temporal en textos estructurados en español en backend

## Siguiente: Fase 3

Caja negra: timeline, exportación, persistencia del ring.
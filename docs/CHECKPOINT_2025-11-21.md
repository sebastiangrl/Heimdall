# Checkpoint de desarrollo — 21 nov 2025

## Verificado en esta sesión

| Comprobación | Resultado |
|--------------|-----------|
| `cargo test -p heimdall-diagnostics -p heimdall-blackbox -p heimdall-core` | ✅ 5 tests |
| `npm run build` (desktop) | ✅ |
| UI Caja negra en navegación | ✅ `BlackBoxPage` + comandos Tauri |
| Glassmorphism (tokens CSS) | ✅ `global.css` |
| Ciclo de vida diagnósticos (backend) | ✅ detectado / persistente |
| Ciclo de vida en tarjeta UI | ✅ etiqueta en `DiagnosisCard` |

## Comprobar manualmente (Asgard)

```bash
cd /mnt/datos/Heimdall
cargo test -p heimdall-diagnostics -p heimdall-blackbox
cd apps/desktop && npm run tauri dev
```

1. **Resumen**: telemetría CPU/RAM/GPU real; diagnósticos si hay presión.
2. **Inteligencia**: lista de diagnósticos con estado (detectado/persistente).
3. **Caja negra**: indicador «Registrando», contadores de muestras e incidentes.
4. **Incidente**: provocar presión de memoria controlada (`stress-ng --vm 1 --vm-bytes 70% --timeout 60s`) y comprobar que, tras ~8 muestras, aparece incidente (no forzar OOM).

## Limitaciones conocidas (honestas)

- **Fase 2**: sin `resolved`/`dismissed` automático al cesar la regla; sin journald (OOM/NVRM); reglas térmicas requieren sensores.
- **Fase 3**: sin línea temporal interactiva ni exportación; ring principalmente en RAM; ventanas pre/post fijas en código.
- **Fases 4–6**: no iniciadas (perfiles, Ollama, daemon/tray).

## Siguiente trabajo (orden del brief)

1. Cerrar Fase 2: `resolved`, cooldown, journald lectura, más tests normal vs problema.
2. Ampliar Fase 3: persistencia periódica del ring, vista detallada + timeline, exportación redactada.
3. Fase 4 en adelante solo tras verificación de 2–3.
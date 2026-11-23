# Experimentos reproducibles (Bastian Rivers)

Pruebas **seguras**: sin forzar kernel panic, sin agotar toda la RAM ni dañar disco.

## Experimento A — ¿Por qué está lento?

1. Abre Heimdall (`tauri dev` o el binario).
2. En una terminal:

```bash
stress-ng --vm 1 --vm-bytes 70% --timeout 45s
```

3. En **Resumen** / **Inteligencia**: debe aparecer presión de memoria con evidencias (swap/PSI).
4. En **Caja negra**: incidente con línea temporal y exportación JSON.

## Experimento B — ¿Qué pasó antes del fallo?

Heimdall **no** marca fallo solo porque un proceso desaparezca. Usa journal + caja negra.

**Opción 1 — Evento OOM ya en el boot (solo lectura):**

```bash
journalctl -b -p warning --no-pager | grep -iE 'oom|out of memory' | tail -5
```

Si hay líneas, al abrir Heimdall puede registrarse incidente `journal_oom` (una vez por mensaje nuevo en la sesión).

**Opción 2 — Diagnóstico persistente (controlado):**

Misma carga que el experimento A hasta que el diagnóstico pase a **persistente** → incidente `memory_pressure` con pre/post window.

**Opción 3 — Proceso de prueba (salida anómala, sin engaño):**

```bash
bash scripts/experiment-b-probe.sh
```

El script documenta que la caja negra mostrará contexto **solo** si hay correlación en journal o diagnóstico; no afirma cierre “misterioso” sin evidencia.

## Experimento C — Perfiles (Fase 4)

Pendiente hasta Fase 4: alternar desarrollo / gaming / reposo.

## Variables de caja negra (opcional)

```bash
export HEIMDALL_BLACKBOX_RING=240          # muestras en RAM (~segundos a 1 Hz)
export HEIMDALL_BLACKBOX_POST_FRAMES=45   # muestras después del incidente
export HEIMDALL_BLACKBOX_CHECKPOINT_SECS=15
```
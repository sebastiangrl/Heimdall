# Fase 5 — Inteligencia local (completa)

## Entregables

- **`heimdall-intelligence`**: cliente Ollama (`probe`, `ask`, `build_context`).
- **Ajustes persistentes** en `~/.local/share/heimdall/intelligence.json` (URL, modelo preferido, cola diferida).
- **Cola**: preguntas encoladas bajo gaming/GPU caliente; vaciado automático en cada muestra cuando baja la carga.
- **UI**: Inteligencia (asistente + reglas), Configuración (Ollama).
- **Comandos Tauri**: `get/set_intelligence_settings`, `get_assistant_queue_status`, `take_completed_assistant`.
- **Tests** sin red para `ask` bloqueado y `build_context`.

## Límites conscientes

- Sin descarga de modelos ni shell para Ollama.
- Cola en memoria (una pregunta; expira a 1 h).
- Con daemon activo, el asistente sigue usando el servicio embebido de la app (no el del daemon).

## Prueba rápida

```bash
cargo test -p heimdall-intelligence
systemctl --user start ollama   # si aplica
cd apps/desktop && pnpm run tauri dev
```
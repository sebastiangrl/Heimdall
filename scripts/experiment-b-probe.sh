#!/usr/bin/env bash
# Experimento B — evidencia en journal (solo lectura)
set -euo pipefail
echo "Heimdall — Experimento B (journal, sin simular cierres)"
echo ""
echo "Últimas señales OOM/NVRM/E/S en este arranque:"
journalctl -b -p warning --no-pager 2>/dev/null | grep -iE 'oom|out of memory|nvrm|xid|i/o error|blk_update' | tail -8 || echo "(sin líneas coincidentes o sin journalctl)"
echo ""
echo "Abre Heimdall: si hay mensajes nuevos en esta sesión, la Caja negra puede registrar journal_*."
echo "Un proceso que solo sale no genera incidente sin correlación."
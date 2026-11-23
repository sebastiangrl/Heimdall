#!/usr/bin/env bash
# Mediciones locales (Fase 6). Sin empaquetado.
set -euo pipefail
cd "$(dirname "$0")/.."
BIN="${1:-./target/debug/heimdall-daemon}"
if [[ ! -x "$BIN" ]] && [[ "$BIN" == *heimdall-daemon* ]]; then
  echo "Compilando heimdall-daemon..."
  cargo build -p heimdall-daemon
  BIN=./target/debug/heimdall-daemon
fi

echo "== Heimdall bench =="
echo "Rust tests (core path):"
/usr/bin/time -f "elapsed %e s" cargo test -p heimdall-diagnostics -p heimdall-blackbox -p heimdall-profiles -q

echo ""
echo "10× snapshot_and_diagnose (embedded, bin de prueba cargo):"
cargo run -q -p heimdall-daemon --example bench_embedded 2>/dev/null || {
  echo "(bench_embedded pendiente — usa tiempo de tests como proxy)"
}
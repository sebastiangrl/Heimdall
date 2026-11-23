#!/usr/bin/env bash
# Instala heimdall-daemon en ~/.local/bin y unit systemd de usuario.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

BIN="$CARGO_TARGET_DIR/release/heimdall-daemon"
if [[ ! -x "$BIN" ]]; then
  echo "Compilando daemon..."
  cargo build --release -p heimdall-daemon
fi

install -D -m 0755 "$BIN" "$HOME/.local/bin/heimdall-daemon"

UNIT_DIR="$HOME/.config/systemd/user"
mkdir -p "$UNIT_DIR"
sed "s|%h|$HOME|g" "$ROOT/packaging/systemd/heimdall-daemon.service" > "$UNIT_DIR/heimdall-daemon.service"

systemctl --user daemon-reload
echo "Listo. Iniciar con:"
echo "  systemctl --user enable --now heimdall-daemon.service"
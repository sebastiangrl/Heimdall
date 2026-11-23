#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
export TMPDIR="${TMPDIR:-$ROOT/.tmp}"
mkdir -p "$TMPDIR"
export npm_config_cache="${npm_config_cache:-$ROOT/.tmp/npm-cache}"
mkdir -p "$npm_config_cache"

cd "$ROOT"
echo "TMPDIR=$TMPDIR"
cargo test -p heimdall-core -p heimdall-linux -p heimdall-diagnostics \
  -p heimdall-blackbox -p heimdall-profiles -p heimdall-intelligence -p heimdall-ipc
if [[ "${SKIP_FRONTEND:-0}" != "1" ]]; then
  cd apps/desktop
  npm run build
else
  echo "SKIP_FRONTEND=1 — omitido npm run build"
fi
echo "OK — listo para ./scripts/build-release.sh"
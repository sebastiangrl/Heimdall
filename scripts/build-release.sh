#!/usr/bin/env bash
# Build de release (desktop + daemon). Ejecutar desde la raíz del repo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
export TMPDIR="${TMPDIR:-$ROOT/.tmp}"
mkdir -p "$TMPDIR"
export npm_config_cache="${npm_config_cache:-$ROOT/.tmp/npm-cache}"
mkdir -p "$npm_config_cache"
OUT="$ROOT/dist/release"
PROFILE="${PROFILE:-release}"

echo "== Heimdall release build =="
echo "ROOT=$ROOT"
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "PROFILE=$PROFILE"

cd "$ROOT"
cargo build --profile "$PROFILE" -p heimdall-daemon -p heimdall-desktop

cd "$ROOT/apps/desktop"
npm run build
npm run tauri build -- --bundles deb,appimage,rpm 2>/dev/null || npm run tauri build

mkdir -p "$OUT"
cp -a "$CARGO_TARGET_DIR/$PROFILE/heimdall-daemon" "$OUT/"
cp -a "$CARGO_TARGET_DIR/$PROFILE/heimdall-desktop" "$OUT/" 2>/dev/null || true

BUNDLE_DIR="$ROOT/apps/desktop/src-tauri/target/$PROFILE/bundle"
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="$CARGO_TARGET_DIR/$PROFILE/bundle"
fi

if [[ -d "$BUNDLE_DIR" ]]; then
  mkdir -p "$OUT/bundle"
  cp -a "$BUNDLE_DIR"/* "$OUT/bundle/" 2>/dev/null || true
  echo "Bundles copiados desde $BUNDLE_DIR"
fi

echo ""
echo "Artefactos en: $OUT"
ls -la "$OUT"
#!/usr/bin/env bash
# Publicar tag v0.1.0 en GitHub (requiere `gh` autenticado). No hace push de commits.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-v0.1.0}"
NOTES="$ROOT/docs/GITHUB_RELEASE_v0.1.0.md"

if ! command -v gh >/dev/null 2>&1; then
  echo "Instala GitHub CLI: pacman -S github-cli"
  exit 1
fi

if [[ ! -f "$NOTES" ]]; then
  echo "Falta $NOTES"
  exit 1
fi

echo "== Verificación previa =="
export TMPDIR="${TMPDIR:-$ROOT/.tmp}"
export npm_config_cache="${npm_config_cache:-$ROOT/.tmp/npm-cache}"
mkdir -p "$TMPDIR" "$npm_config_cache"
"$ROOT/scripts/verify-pre-release.sh"

echo ""
echo "== Tag local $VERSION =="
if git rev-parse "$VERSION" >/dev/null 2>&1; then
  echo "El tag $VERSION ya existe."
else
  git tag -s "$VERSION" -m "Heimdall $VERSION — MVP fases 0–6"
fi

echo "== Push tag (dispara release.yml) =="
git push origin "$VERSION"

echo ""
echo "Si el workflow no adjunta artefactos, crea release manual:"
echo "  gh release create $VERSION --notes-file $NOTES dist/release/bundle/*"
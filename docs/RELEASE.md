# Empaquetado y release

## Build local (Asgard)

Si `/tmp` está lleno, los tests y SQLite fallan. Usa un tmp dentro del repo:

```bash
export TMPDIR=/mnt/datos/Heimdall/.tmp
mkdir -p "$TMPDIR"
```

```bash
cd /mnt/datos/Heimdall
chmod +x scripts/build-release.sh scripts/install-daemon-user.sh
./scripts/build-release.sh
```

Salida en `dist/release/`:

- `heimdall-daemon`, `heimdall-desktop` (binarios release)
- `bundle/` — `.deb`, AppImage o `.rpm` si Tauri los generó

### Daemon en segundo plano

```bash
./scripts/install-daemon-user.sh
systemctl --user enable --now heimdall-daemon.service
```

### Icono KDE / `.desktop`

```bash
cd apps/desktop
npm run install:linux-branding   # usa target/release si existe
```

## Arch / CachyOS (PKGBUILD local)

```bash
cd packaging/arch
makepkg -si
```

El `PKGBUILD` construye desde el árbol padre (`../..`); no hace falta tarball.

## CI

- **Push / PR → `main`**: tests Rust ampliados + build frontend (`.github/workflows/ci.yml`).
- **Tag `v*`**: workflow `release.yml` — AppImage + `.deb` en GitHub Releases (Ubuntu runner).

Crear release:

Texto listo para GitHub: [`GITHUB_RELEASE_v0.1.0.md`](./GITHUB_RELEASE_v0.1.0.md)

Automatizado (tras commit y `gh auth login`):

```bash
./scripts/publish-release.sh v0.1.0
```

Manual:

```bash
git tag -s v0.1.0 -m "Heimdall 0.1.0"
git push origin v0.1.0
gh release create v0.1.0 --notes-file docs/GITHUB_RELEASE_v0.1.0.md
```

## Checklist pre-release

1. `cargo test --workspace` (excepto crates que requieren GPU/NVML).
2. `cd apps/desktop && npm run build && npm run tauri build`.
3. Probar daemon + desktop juntos (socket `~/.local/share/heimdall/heimdall.sock`).
4. `./scripts/verify-pre-release.sh` (tests + frontend).
5. Anotar benchmarks en [`PERFORMANCE.md`](./PERFORMANCE.md).

## Fuera de alcance (0.1.0)

- Publicación AUR oficial (solo PKGBUILD de referencia).
- Firma de códigos Windows/macOS.
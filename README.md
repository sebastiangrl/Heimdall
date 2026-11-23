# Heimdall

**See everything. Protect what matters.**

Heimdall is an open-source desktop application (Project Asgard) that monitors system health, records incidents, explains likely causes with **evidence-first rules**, and recommends **safe** actions — fully local. An optional on-device explanation hook (Ollama) is not required to use the app.

**Creator:** Bastian Rivers

## Status

| Phase | State |
|-------|--------|
| 0–6 | Feature-complete MVP (ver [`docs/ROADMAP.md`](docs/ROADMAP.md)) |
| Release | Scripts + CI; ver [`docs/RELEASE.md`](docs/RELEASE.md) |

## Clone

```bash
git clone https://github.com/sebastiangrl/Heimdall.git
cd Heimdall
```

## Install & build (Linux — Arch / CachyOS)

Full steps: **[docs/SETUP_CACHYOS.md](docs/SETUP_CACHYOS.md)** (also works on most Arch-based systems).

1. **System packages** — GTK 3, WebKitGTK 4.1, toolchain (`base-devel`, `openssl`, `webkit2gtk-4.1`, …).
2. **Rust** — [rustup](https://rustup.rs) stable (`source "$HOME/.cargo/env"`).
3. **Node** — npm in `apps/desktop`:

```bash
cd apps/desktop && npm install
source "$HOME/.cargo/env"
cd ../..
cargo test -p heimdall-linux -p heimdall-diagnostics
cd apps/desktop && npm run tauri dev
```

Issues (WebKit, `rlib`/Cargo cache, GPU): **[docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)**.

### Optional background daemon

```bash
./scripts/install-daemon-user.sh
systemctl --user enable --now heimdall-daemon.service
```

## Release build

```bash
export TMPDIR="${TMPDIR:-$PWD/.tmp}"   # si /tmp del sistema está lleno
./scripts/verify-pre-release.sh
./scripts/build-release.sh
```

Artefactos: `dist/release/`. Historial: [`CHANGELOG.md`](CHANGELOG.md).

## Repository layout

- `apps/desktop` — Tauri 2 + React UI
- `crates/*` — Rust workspace
- `packaging/` — systemd, `.desktop`, Arch PKGBUILD
- `docs/` — Architecture, phases, release
- `scripts/` — build, bench, experiments

## Docs

| Topic | File |
|-------|------|
| Architecture | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| Privacy / data | [docs/PRIVACY.md](docs/PRIVACY.md) |
| Release & packages | [docs/RELEASE.md](docs/RELEASE.md) |
| Roadmap | [docs/ROADMAP.md](docs/ROADMAP.md) |

## License

MIT — see [LICENSE](LICENSE).
# Heimdall Roadmap

## Phase 0 — Discovery and architecture ✅

## Phase 1 — Foundational vertical slice ✅

## Phase 2 — Diagnosis ✅ — [`PHASE2_COMPLETE.md`](./PHASE2_COMPLETE.md)

## Phase 3 — Black Box ✅ — [`PHASE3_COMPLETE.md`](./PHASE3_COMPLETE.md)

## Phase 4 — Adaptive profiles ✅ — [`PHASE4_COMPLETE.md`](./PHASE4_COMPLETE.md)

## Phase 5 — Local intelligence ✅ — [`PHASE5_COMPLETE.md`](./PHASE5_COMPLETE.md)

## Phase 6 — Daemon & desktop ✅ — [`PHASE6_COMPLETE.md`](./PHASE6_COMPLETE.md)

## Release & packaging ✅ (MVP)

- [`RELEASE.md`](./RELEASE.md) — build local, daemon user unit, PKGBUILD de referencia
- `scripts/build-release.sh`, `scripts/install-daemon-user.sh`
- CI ampliado; **GitHub Release** en tags `v*` (AppImage + deb)
- **Pendiente:** AUR oficial, firmas multi-OS, flatpak

## Known blockers (Asgard)

| Item | Owner action |
|------|----------------|
| NVIDIA driver / NVML | Fix GPU driver to enable GPU metrics |
| webkit2gtk-4.1 | `pacman -S webkit2gtk-4.1` (see SETUP_CACHYOS.md) |
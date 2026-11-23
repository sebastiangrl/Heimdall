# CachyOS Setup — Heimdall Development

Review before installing system packages.

## 1. Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
```

Bash/zsh:

```bash
source "$HOME/.cargo/env"
```

Fish (do **not** `source ~/.cargo/env` — that file is bash-only):

```fish
fish_add_path $HOME/.cargo/bin
```

## 2. Tauri 2 system dependencies (Arch/CachyOS)

```bash
sudo pacman -S --needed \
  base-devel \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  webkit2gtk-4.1
```

## 3. Optional telemetry helpers

```bash
sudo pacman -S --needed lm_sensors smartmontools
# NVIDIA (when driver works):
# sudo pacman -S --needed nvidia-utils
```

## 4. Node dependencies

From repository root:

```bash
cd apps/desktop
npm install
```

## 5. Release (opcional)

```bash
cd /path/to/Heimdall
./scripts/build-release.sh
./scripts/install-daemon-user.sh
cd apps/desktop && npm run install:linux-branding
```

Ver [`RELEASE.md`](./RELEASE.md).

## 6. Build and run (desarrollo)

```bash
source "$HOME/.cargo/env"
cd /path/to/Heimdall
cargo test --workspace
cd apps/desktop
npm run tauri dev
```

### Icono en KDE (barra de tareas)

Tras la primera compilación (`target/debug/heimdall-desktop`):

```bash
cd apps/desktop
npm run install:linux-branding
kbuildsycoca6   # Plasma 6; en Plasma 5: kbuildsycoca5
```

## 7. Optional Ollama

Install and start the service separately; Heimdall does not download models automatically.

```bash
systemctl --user enable --now ollama  # if using user service
```

## Troubleshooting

- **`nvidia-smi` fails** — GPU metrics stay `unavailable`; fix driver with CachyOS NVIDIA guides.
- **Tauri webkit errors** — Ensure `webkit2gtk-4.1` not the legacy 4.0 package only.
- **Wayland transparency** — Use Settings → Appearance → reduce glass effects if compositor struggles.
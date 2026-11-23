# Troubleshooting

## Tauri build: `javascriptcoregtk-4.1` / `webkit2gtk-4.1` not found

Rust compiles fine; the desktop shell needs **GTK WebKit 4.1** from the system. On Arch/CachyOS both `javascriptcoregtk-4.1` and `webkit2gtk-4.1` come from the same package:

```bash
sudo pacman -S --needed webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg base-devel openssl
```

Verify:

```bash
pkg-config --modversion webkit2gtk-4.1 javascriptcoregtk-4.1
```

Then:

```bash
cd /mnt/datos/Heimdall/apps/desktop
npm run tauri dev
```

First compile after installing WebKit can take several minutes (glib/gtk/webkit chain).

## GPU metrics show unavailable

If `nvidia-smi` fails, Heimdall intentionally reports GPU utilization/VRAM/temperature as **unavailable**. Restore the NVIDIA driver stack; optional `cargo build -p heimdall-linux --features nvidia` enables NVML when the driver works.

## `heimdall_linux` required to be available in rlib format

Rust needs a full **library** build (`.rlib`) of `heimdall-linux` when linking `heimdall-desktop`. A previous **`cargo check`** (or a mixed `target/` vs `.cargo-target/`) can leave only **`.rmeta`** metadata for the same crate, so the linker fails with:

```text
error: crate `heimdall_linux` required to be available in rlib format, but was not found in this form
```

**Fix (from repo root):**

```bash
cd /mnt/datos/Heimdall
cargo clean -p heimdall-linux -p heimdall-telemetry -p heimdall-desktop
cargo build -p heimdall-desktop
```

Or clean the whole artifact dir once:

```bash
rm -rf .cargo-target/debug/build/heimdall-linux target/debug/build/heimdall-linux
cargo build -p heimdall-desktop
```

Use **one** target directory only (`.cargo/config.toml` pins `.cargo-target`; avoid building sometimes in `target/` and sometimes with another `CARGO_TARGET_DIR`).

Then:

```bash
cd apps/desktop
npm run tauri dev
```

## Vite-only preview shows no metrics

Open the app with Tauri (`npm run tauri dev`). Browser-only `npm run dev` cannot reach Rust collectors.

## Wayland glass effects

Enable **Settings → Appearance → reduced glass** (store wired in Phase 1 UI foundation) or toggle `reduced-glass` on `body` for opaque panels.
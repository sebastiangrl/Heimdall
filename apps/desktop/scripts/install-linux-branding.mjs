#!/usr/bin/env node
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync, spawnSync } from "node:child_process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(root, "../..");
const appId = "com.asgard.heimdall";
const iconSrc = join(root, "src-tauri/icons/128x128.png");

const binaryCandidates = [
  join(repoRoot, "target/release/heimdall-desktop"),
  join(repoRoot, "target/debug/heimdall-desktop"),
  join(root, "src-tauri/target/release/heimdall-desktop"),
  join(root, "src-tauri/target/debug/heimdall-desktop"),
  join(repoRoot, "target/debug/heimdall-desktop.exe"),
];

function findBinary() {
  for (const p of binaryCandidates) {
    if (existsSync(p)) return p;
  }
  const locate = spawnSync("which", ["heimdall-desktop"], { encoding: "utf8" });
  if (locate.status === 0) {
    const p = locate.stdout.trim();
    if (p && existsSync(p)) return p;
  }
  return null;
}

if (!existsSync(iconSrc)) {
  console.error("Falta el icono. Genera: npx tauri icon branding/heimdall-app-icon-1024.png -o src-tauri/icons");
  process.exit(1);
}

const binary = findBinary();
if (!binary) {
  console.error("No encuentro heimdall-desktop. Rutas buscadas:");
  binaryCandidates.forEach((p) => console.error("  -", p));
  console.error("");
  console.error("Con el workspace Rust el binario suele quedar en:");
  console.error("  ../../target/debug/heimdall-desktop");
  console.error("Deja `pnpm run tauri dev` corriendo hasta que compile, y vuelve a ejecutar este script.");
  process.exit(1);
}

const iconHome = join(homedir(), ".local/share/icons/hicolor");
for (const size of [16, 22, 24, 32, 48, 64, 128, 256]) {
  const dir = join(iconHome, `${size}x${size}/apps`);
  mkdirSync(dir, { recursive: true });
  copyFileSync(iconSrc, join(dir, `${appId}.png`));
}

try {
  execFileSync("gtk-update-icon-cache", ["-f", "-t", iconHome], { stdio: "pipe" });
} catch {
  /* opcional */
}

const desktopDir = join(homedir(), ".local/share/applications");
mkdirSync(desktopDir, { recursive: true });

const desktop = `[Desktop Entry]
Version=1.0
Type=Application
Name=Heimdall
GenericName=Asistente del sistema
Comment=Lo ve todo. Protege lo que importa.
Exec=${binary}
TryExec=${binary}
Icon=${iconSrc}
StartupWMClass=Heimdall
StartupNotify=true
Terminal=false
Categories=System;Monitor;
Keywords=monitor;diagnostics;system;
X-GTK-Application-ID=${appId}
`;
writeFileSync(join(desktopDir, `${appId}.desktop`), desktop, "utf8");

try {
  execFileSync("update-desktop-database", [desktopDir], { stdio: "pipe" });
} catch {
  /* opcional */
}

for (const cmd of ["kbuildsycoca6", "kbuildsycoca5"]) {
  const hit = spawnSync("which", [cmd], { encoding: "utf8" });
  if (hit.status === 0 && hit.stdout.trim()) {
    try {
      execFileSync(cmd, [], { stdio: "inherit" });
      break;
    } catch {
      /* siguiente */
    }
  }
}

console.log("Icono registrado. Binario:", binary);
console.log("Si el panel sigue genérico, abre desde el menú «Heimdall», no desde un acceso antiguo.");
# Heimdall v0.1.0 — MVP (Project Asgard)

**Lo ve todo. Protege lo que importa.**

Primera versión usable: monitor local, diagnóstico con evidencia, Caja negra, perfiles adaptativos, asistente Ollama opcional y daemon en segundo plano.

## Instalación rápida (Linux)

1. Dependencias: ver [SETUP_CACHYOS.md](./SETUP_CACHYOS.md) (`webkit2gtk-4.1`, GTK).
2. Desde el tag o tarball:

```bash
./scripts/build-release.sh
./scripts/install-daemon-user.sh
systemctl --user enable --now heimdall-daemon.service
cd apps/desktop && npm run install:linux-branding
```

3. O usa el **`.deb`** / **AppImage** adjunto a este release (si el workflow CI los generó).

## Qué incluye

- UI: Resumen, Hardware, Procesos, Inteligencia, Caja negra, Perfiles, Ajustes (español por defecto).
- Reglas deterministas + journal (OOM/NVRM); descartar hallazgos.
- Incidentes con ventana pre/post y export redactado.
- Ollama local (sin descarga automática de modelos).
- Socket `~/.local/share/heimdall/heimdall.sock` cuando el daemon está activo.

## Requisitos conocidos

- GPU NVIDIA: métricas solo si el driver/NVML responde.
- Con `/tmp` lleno en el sistema: export `TMPDIR=$PWD/.tmp` antes de compilar (ver [RELEASE.md](./RELEASE.md)).

## Ver también

- [CHANGELOG.md](../CHANGELOG.md)
- [ROADMAP.md](./ROADMAP.md)
#!/usr/bin/env python3
"""
Carga benigna intermitente (legado). Sin privilegios.

Modelo de ejecución:
- --workers > 1 usa multiprocessing.spawn → procesos OS independientes (identidades distintas).
- Dentro de cada worker, los "hilos" son threading.Thread → el GIL limita el paralelismo
  real de CPU en CPython; varios hilos no saturan varios núcleos en un solo proceso.

Limitaciones frente a Heimdall (umbrales por defecto, burst_core 70 %, min_hits 4, ventana 20 s):
- Solo entra al tracker histórico si el proceso está en el top FOLLOW_CPU (8) del tick.
- El muestreo es ~1 Hz: ráfagas cortas pueden aportar <4 muestras útiles en la ventana.
- Por defecto burst-sec=1.5 suele ser insuficiente; preferir scripts/phantom para laboratorio.

Preferir scripts/phantom (un proceso, hilos nativos) para demostraciones reproducibles.
"""
from __future__ import annotations

import argparse
import multiprocessing as mp
import os
import signal
import sys
import threading
import time

MAX_WORKERS = 8
MAX_BURSTS = 30
MAX_BURST_SEC = 30.0
MAX_REST_SEC = 60.0
MAX_TOTAL_SEC = 600.0


def _spin(secs: float) -> None:
    buf = bytearray(1 << 16)
    end = time.perf_counter() + secs
    while time.perf_counter() < end:
        for i in range(0, len(buf), 64):
            buf[i] = (buf[i] + 3) % 251
        _ = sum(buf)


def _worker(burst_sec: float, rest_sec: float, bursts: int, max_total: float) -> None:
    deadline = time.monotonic() + max_total
    threads = max(1, min(4, os.cpu_count() or 1))
    for _ in range(bursts):
        if time.monotonic() >= deadline:
            break
        workers = [
            threading.Thread(target=_spin, args=(burst_sec,), daemon=True)
            for _ in range(threads)
        ]
        for t in workers:
            t.start()
        for t in workers:
            t.join()
        if time.monotonic() + rest_sec >= deadline:
            break
        time.sleep(rest_sec)


def main() -> int:
    parser = argparse.ArgumentParser(description="Simulador de ráfagas CPU benignas")
    parser.add_argument("--workers", type=int, default=1, help=f"Hilos de proceso (1-{MAX_WORKERS})")
    parser.add_argument("--bursts", type=int, default=6, help=f"Ráfagas por worker (1-{MAX_BURSTS})")
    parser.add_argument("--burst-sec", type=float, default=1.5, help="Duración activa por ráfaga")
    parser.add_argument("--rest-sec", type=float, default=2.5, help="Descanso entre ráfagas")
    parser.add_argument("--max-total-sec", type=float, default=120.0, help="Límite total por worker")
    args = parser.parse_args()

    workers = max(1, min(MAX_WORKERS, args.workers))
    bursts = max(1, min(MAX_BURSTS, args.bursts))
    burst_sec = min(MAX_BURST_SEC, max(0.2, args.burst_sec))
    rest_sec = min(MAX_REST_SEC, max(0.0, args.rest_sec))
    max_total = min(MAX_TOTAL_SEC, max(1.0, args.max_total_sec))

    procs: list[mp.Process] = []

    def shutdown(*_args: object) -> None:
        for p in procs:
            if p.is_alive():
                p.terminate()
        for p in procs:
            p.join(timeout=2.0)
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    ctx = mp.get_context("spawn")
    for _ in range(workers):
        p = ctx.Process(
            target=_worker,
            args=(burst_sec, rest_sec, bursts, max_total),
            daemon=False,
        )
        p.start()
        procs.append(p)

    for p in procs:
        p.join()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
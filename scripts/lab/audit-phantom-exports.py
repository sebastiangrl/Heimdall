#!/usr/bin/env python3
"""Cruza phantom-repro.tsv con JSON exportados: PID, nombre, created_at posteriores a la corrida."""
from __future__ import annotations

import csv
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TSV = ROOT / "docs/experiments/phantom-repro-20261122-194048.tsv"
EXP = ROOT / "docs/experiments"
RUN_START = datetime(2026, 11, 23, 0, 40, 48, tzinfo=timezone.utc)


def main() -> int:
    if not TSV.is_file():
        print("FAIL: falta TSV", TSV)
        return 2
    rows = []
    with TSV.open() as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("phase") or line.startswith("    "):
                continue
            parts = line.split("\t")
            if len(parts) >= 3:
                rows.append(
                    {
                        "phase": parts[0],
                        "threads": int(parts[1]),
                        "run": int(parts[2]),
                        "pid": int(parts[3]),
                        "accepted": int(parts[4]),
                    }
                )
    exports = sorted(EXP.glob("phantom-export-*.json"))
    exports = [p for p in exports if p.name != "phantom-export-latest.json"]
    ok = True
    by_pid = {r["pid"]: r for r in rows}
    used = set()
    for path in exports:
        data = json.loads(path.read_text())
        pid = data.get("identity", {}).get("pid")
        name = data.get("identity", {}).get("name", "")
        created = data.get("created_at", "")
        if pid not in by_pid:
            print(f"FAIL {path.name}: pid {pid} no está en TSV")
            ok = False
            continue
        if name != "phantom":
            print(f"FAIL {path.name}: name={name}")
            ok = False
        if created and created < RUN_START.isoformat().replace("+00:00", "Z"):
            print(f"FAIL {path.name}: created_at antiguo {created}")
            ok = False
        if pid in used:
            print(f"WARN {path.name}: pid {pid} duplicado en export")
        used.add(pid)
        print(f"OK {path.name} pid={pid} created={created[:19]}")
    std5 = [r for r in rows if r["phase"] == "standard_4t"]
    if len(std5) != 5 or not all(r["accepted"] == 1 for r in std5):
        print("FAIL: standard_4t no es 5/5 accepted")
        ok = False
    else:
        print("OK: 5/5 standard_4t accepted en TSV")
    if len(exports) < 8:
        print(f"WARN: solo {len(exports)} exports (esperado 8)")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
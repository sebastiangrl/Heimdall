#!/usr/bin/env python3
"""Registro post-ejecución: expediente en SQLite e identidad del proceso bajo prueba."""
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path

DEFAULT_DB = Path.home() / ".local/share/heimdall/blackbox.db"
THRESHOLD = 70.0
WINDOW_HINT = 20
MIN_HITS = 4


def find_dossier_for_pid(db: Path, pid: int, since: str | None) -> dict | None:
    conn = sqlite3.connect(db)
    q = "SELECT payload_json, updated_at FROM investigations ORDER BY updated_at DESC LIMIT 50"
    rows = conn.execute(q).fetchall()
    for payload, updated in rows:
        if since:
            pass  # filtrado por created_at tras parsear
        try:
            d = json.loads(payload)
        except json.JSONDecodeError:
            continue
        if since and (d.get("created_at") or "") < since:
            continue
        idn = d.get("identity") or {}
        if idn.get("pid") == pid:
            d["_updated_at"] = updated
            return d
    return None


def summarize_dossier(d: dict) -> dict:
    attr = d.get("attribution", {})
    hits = attr.get("burst_hit_times") or []
    series = d.get("process_series") or []
    core_over = [
        s
        for s in series
        if (s.get("cpu_core_percent") or 0) >= THRESHOLD
        and str(s.get("availability", "available")).lower() in ("available", '"available"')
    ]
    idn = d.get("identity", {})
    exit_ev = attr.get("exit")
    return {
        "id": d.get("id"),
        "rowid": d.get("_rowid"),
        "pid": idn.get("pid"),
        "name": idn.get("name"),
        "start_time_ticks": idn.get("start_time_ticks"),
        "level": d.get("level"),
        "state": d.get("state"),
        "confidence": attr.get("confidence"),
        "impact": (d.get("impact") or {}).get("severity"),
        "burst_hits_recorded": len(hits),
        "samples_core_ge_threshold": len(core_over),
        "process_samples": len(series),
        "coverage": attr.get("coverage"),
        "exit": exit_ev,
        "updated_at": d.get("_updated_at"),
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--pid", type=int, default=None, help="PID esperado del proceso bajo prueba")
    p.add_argument("--since", type=str, default=None, help="ISO updated_at watermark")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()
    if not args.db.is_file():
        print("sin base de datos", file=sys.stderr)
        return 2
    match = None
    if args.pid is not None:
        dossier = find_dossier_for_pid(args.db, args.pid, args.since)
        if dossier:
            match = summarize_dossier(dossier)
    ok = match is not None and (match.get("burst_hits_recorded") or 0) >= MIN_HITS
    if args.json:
        print(
            json.dumps(
                {
                    "threshold_core": THRESHOLD,
                    "min_hits": MIN_HITS,
                    "window_secs_hint": WINDOW_HINT,
                    "match": match,
                    "accepted": ok,
                },
                indent=2,
            )
        )
    else:
        print(f"aceptacion={ok}")
        if match:
            print(json.dumps(match, indent=2))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
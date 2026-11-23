#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PHANTOM="${PHANTOM_BIN:-$ROOT/scripts/phantom/.cargo-target/release/phantom}"
SOCK="${HOME}/.local/share/heimdall/heimdall.sock"
DAEMON="${HEIMDALL_DAEMON_BIN:-$ROOT/target/release/heimdall-daemon}"

pkill -x heimdall-daemon 2>/dev/null || true
sleep 1
"$DAEMON" &
sleep 2
"$PHANTOM" &
P=$!
for _ in $(seq 1 45); do
  python3 - "$SOCK" "$P" <<'PY'
import json, socket, sys
sock, target = sys.argv[1], int(sys.argv[2])
req = {"id": 1, "method": "process_sample", "params": {}}
try:
    s = socket.socket(socket.AF_UNIX)
    s.settimeout(2)
    s.connect(sock)
    s.sendall((json.dumps(req) + "\n").encode())
    line = s.recv(2_000_000).decode().splitlines()[0]
    resp = json.loads(line)
    data = resp.get("data") or {}
    rows = data.get("byCpu") or data.get("by_cpu") or []
    hit = next((r for r in rows if r.get("pid") == target), None)
    if hit:
        print("ipc", hit.get("name"), "pid", hit.get("pid"), "cpu", hit.get("cpuPercent", hit.get("cpu_percent")))
    else:
        print("ipc phantom not in top cpu list (len=%d)" % len(rows))
except Exception as e:
    print("ipc_err", e)
PY
  sleep 1
done
wait "$P" 2>/dev/null || true
pkill -x heimdall-daemon 2>/dev/null || true
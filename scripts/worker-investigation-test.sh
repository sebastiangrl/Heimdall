#!/usr/bin/env bash
# Prueba 1/2/4 workers: cada worker debe poder generar identidades distintas (no fusionar sin evidencia).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_ROOT="${CARGO_TARGET_DIR:-$ROOT/.cargo-target}"
DAEMON="${HEIMDALL_DAEMON_BIN:-$CARGO_ROOT/release/heimdall-daemon}"
DB="${HOME}/.local/share/heimdall/blackbox.db"
DEMO="$ROOT/scripts/investigation-burst-demo.py"

if [[ ! -x "$DAEMON" ]]; then
  echo "Falta daemon: $DAEMON" >&2
  exit 1
fi

pkill -x heimdall-daemon 2>/dev/null || true
sleep 1
"$DAEMON" &
DPID=$!
trap 'kill $DPID 2>/dev/null || true' EXIT
sleep 2

count_identities() {
  python3 - "$DB" <<'PY'
import json, sqlite3, sys
conn = sqlite3.connect(sys.argv[1])
rows = conn.execute(
    "SELECT identity_json FROM investigations ORDER BY updated_at DESC LIMIT 40"
).fetchall()
keys = []
for (ij,) in rows:
    idj = json.loads(ij)
    keys.append((idj.get("pid"), idj.get("start_time_ticks"), idj.get("name")))
uniq = sorted(set(keys))
print(f"  total_rows={len(rows)} distinct_identities={len(uniq)}")
for k in uniq[-8:]:
    print(f"    pid={k[0]} ticks={k[1]} name={k[2]}")
PY
}

for w in 1 2 4; do
  echo "=== workers=$w ==="
  python3 "$DEMO" --workers "$w" --bursts 8 --burst-sec 2.5 --rest-sec 1.2 --max-total-sec 55
  sleep 25
  count_identities
done
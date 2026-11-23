#!/usr/bin/env bash
# Demostración reproducible con umbrales Heimdall por defecto (sin HEIMDALL_BURST_*).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_ROOT="${CARGO_TARGET_DIR:-$ROOT/.cargo-target}"
DAEMON="${HEIMDALL_DAEMON_BIN:-$CARGO_ROOT/release/heimdall-daemon}"
PHANTOM_DIR="$ROOT/scripts/phantom/.cargo-target"
PHANTOM="${PHANTOM_BIN:-$PHANTOM_DIR/release/phantom}"
VERIFY="$ROOT/scripts/lab/verify-investigation.py"
DB="${HOME}/.local/share/heimdall/blackbox.db"
LOG="${ROOT}/docs/experiments/phantom-repro-$(date +%Y%m%d-%H%M%S).tsv"
EXPORT_DIR="${ROOT}/docs/experiments"

mkdir -p "$EXPORT_DIR"
: >"$LOG"
echo -e "phase\tthreads\trun\tpid\taccepted\tburst_hits\tlevel\timpact\tstate" | tee -a "$LOG"

build() {
  (cd "$ROOT" && CARGO_TARGET_DIR="$CARGO_ROOT" cargo build -p heimdall-daemon --release)
  (cd "$ROOT/scripts/phantom" && CARGO_TARGET_DIR=.cargo-target cargo build --release)
}

watermark() {
  python3 -c "import datetime; print(datetime.datetime.now(datetime.timezone.utc).isoformat())"
}

export_dossier() {
  local id=$1
  local out="$EXPORT_DIR/phantom-export-${id}.json"
  sqlite3 "$DB" "SELECT payload_json FROM investigations WHERE id='$id' LIMIT 1;" \
    | python3 -m json.tool >"$out" 2>/dev/null || true
  echo "  exportado: $out"
}

run_one() {
  local phase=$1 threads=$2 run=$3
  local since pid
  since=$(watermark)
  "$PHANTOM" --threads "$threads" --bursts 6 --active-secs 5 --rest-secs 2 --max-total-sec 60 &
  pid=$!
  wait "$pid" || true
  sleep 35
  if python3 "$VERIFY" --pid "$pid" --since "$since" --json >"${EXPORT_DIR}/.verify-${pid}.json" 2>/dev/null; then
    local acc=1
  else
    acc=0
  fi
  local hits level impact state inv_id
  hits=$(python3 -c "import json; d=json.load(open('${EXPORT_DIR}/.verify-${pid}.json')); m=d.get('match') or {}; print(m.get('burst_hits_recorded',0))")
  level=$(python3 -c "import json; d=json.load(open('${EXPORT_DIR}/.verify-${pid}.json')); m=d.get('match') or {}; print(m.get('level',''))")
  impact=$(python3 -c "import json; d=json.load(open('${EXPORT_DIR}/.verify-${pid}.json')); m=d.get('match') or {}; print(m.get('impact',''))")
  state=$(python3 -c "import json; d=json.load(open('${EXPORT_DIR}/.verify-${pid}.json')); m=d.get('match') or {}; print(m.get('state',''))")
  inv_id=$(python3 -c "import json; d=json.load(open('${EXPORT_DIR}/.verify-${pid}.json')); m=d.get('match') or {}; print(m.get('id',''))")
  echo -e "${phase}\t${threads}\t${run}\t${pid}\t${acc}\t${hits}\t${level}\t${impact}\t${state}" | tee -a "$LOG"
  if [[ "$acc" == "1" && -n "$inv_id" ]]; then
    export_dossier "$inv_id"
  fi
  return "$((1 - acc))"
}

build

pkill -x heimdall-daemon 2>/dev/null || true
sleep 1
"$DAEMON" &
DPID=$!
trap 'kill $DPID 2>/dev/null || true' EXIT
sleep 3

fail=0
for t in 1 2 4; do
  echo "=== hilos=$t (1 corrida) ==="
  run_one "thread_sweep" "$t" 1 || fail=$((fail + 1))
done

for run in 1 2 3 4 5; do
  echo "=== configuración estándar, repetición $run/5 ==="
  run_one "standard_4t" 4 "$run" || fail=$((fail + 1))
done

echo ""
echo "Log: $LOG"
if [[ "$fail" -eq 0 ]]; then
  echo "RESULTADO: 5/5 + barrido hilos OK"
  exit 0
fi
echo "RESULTADO: $fail corridas fallidas (revisar verify JSON en docs/experiments)"
exit 1
#!/usr/bin/env bash
# Benchmarks fase 5 — 300 s (5 min) por escenario. Muestrea PIDs reales del daemon / escritorio.
set -euo pipefail
SEC="${HEIMDALL_BENCH_SEC:-300}"
NCPU=$(nproc)
CLK=$(getconf CLK_TCK)
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_ROOT="${CARGO_TARGET_DIR:-$ROOT/.cargo-target}"
DAEMON="${HEIMDALL_DAEMON_BIN:-$CARGO_ROOT/release/heimdall-daemon}"
DESKTOP="${HEIMDALL_DESKTOP_BIN:-$CARGO_ROOT/release/heimdall-desktop}"
PHANTOM="${PHANTOM_BIN:-$ROOT/scripts/phantom/.cargo-target/release/phantom}"
DB="${HOME}/.local/share/heimdall/blackbox.db"
LOGDIR="${TMPDIR:-/tmp}/heimdall-bench-phase5-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$LOGDIR"
export LOGDIR SEC NCPU CLK ROOT DB DAEMON DESKTOP PHANTOM

if [[ ! -x "$DAEMON" ]]; then
  echo "Compilando daemon…" >&2
  (cd "$ROOT" && CARGO_TARGET_DIR="$CARGO_ROOT" cargo build -p heimdall-daemon --release)
fi

db_size() {
  [[ -f "$DB" ]] && stat -c%s "$DB" || echo 0
}

daemon_pid() {
  pgrep -x heimdall-daemon | head -1 || true
}

desktop_pid() {
  pgrep -f 'heimdall-desktop$' | head -1 || pgrep -f heimdall-desktop | head -1 || true
}

sample_pid() {
  local pid=$1
  local tag=$2
  local out="$LOGDIR/${tag}.tsv"
  local end=$((SECONDS + SEC))
  : >"$out"
  while (( SECONDS < end )); do
  if [[ -z "$pid" ]] || [[ ! -d "/proc/$pid" ]]; then
      echo "gone" >>"$out"
      sleep 1
      continue
    fi
    st=$(awk '{print $14+$15}' "/proc/$pid/stat")
    rss=$(awk '/^VmRSS:/{print $2}' "/proc/$pid/status")
    echo "$SECONDS $st $rss" >>"$out"
    sleep 1
  done
}

summarize() {
  local file=$1
  local label=$2
  python3 - "$file" "$NCPU" "$CLK" "$label" <<'PY'
import sys
rows=[]
for line in open(sys.argv[1]):
    if line.startswith('gone'): continue
    p=line.split()
    if len(p)>=3: rows.append(tuple(map(int,p)))
if len(rows)<2:
    print(f"{sys.argv[4]}: n=0")
    sys.exit(0)
cpus=[]; rss=[]
for i in range(1,len(rows)):
    dt=rows[i][0]-rows[i-1][0]
    if dt<=0: continue
    ds=(rows[i][1]-rows[i-1][1])/(dt*float(sys.argv[3]))*100
    cpus.append(ds)
    rss.append(rows[i][2])
cpus.sort()
print(f"{sys.argv[4]}: n={len(cpus)} mean_one_core_pct={sum(cpus)/len(cpus):.2f} p95={cpus[int(0.95*(len(cpus)-1))]:.2f} max={cpus[-1]:.2f} machine_mean={sum(cpus)/len(cpus)/float(sys.argv[2]):.3f} rss_mean={sum(rss)/len(rss):.0f} max_rss={max(rss)}")
PY
}

stop_daemon() {
  pkill -x heimdall-daemon 2>/dev/null || true
  pkill -f heimdall-desktop 2>/dev/null || true
  sleep 1
}

run_idle() {
  local name=01_daemon_idle
  echo "=== $name (${SEC}s) ===" | tee "$LOGDIR/$name.summary"
  local db0 db1
  db0=$(db_size)
  "$DAEMON" &
  local wrap=$!
  sleep 2
  local dp
  dp=$(daemon_pid)
  [[ -z "$dp" ]] && dp=$wrap
  echo "daemon_pid=$dp" | tee -a "$LOGDIR/$name.summary"
  sample_pid "$dp" "01_daemon" &
  local sp=$!
  wait $sp
  kill "$dp" 2>/dev/null || true
  wait "$wrap" 2>/dev/null || true
  db1=$(db_size)
  summarize "$LOGDIR/01_daemon.tsv" "daemon" | tee -a "$LOGDIR/$name.summary"
  echo "db_bytes_start=$db0 db_bytes_end=$db1 delta=$((db1-db0))" | tee -a "$LOGDIR/$name.summary"
}

run_ui() {
  local name=02_daemon_with_ui
  if [[ ! -x "$DESKTOP" || -z "${DISPLAY:-}" ]]; then
    echo "=== $name SKIPPED ===" | tee "$LOGDIR/$name.summary"
    return
  fi
  echo "=== $name (${SEC}s) ===" | tee "$LOGDIR/$name.summary"
  local db0 db1
  db0=$(db_size)
  "$DAEMON" &
  sleep 2
  "$DESKTOP" &
  sleep 2
  local dp tp
  dp=$(daemon_pid)
  tp=$(desktop_pid)
  echo "daemon_pid=$dp desktop_pid=$tp" | tee -a "$LOGDIR/$name.summary"
  sample_pid "$dp" "02_daemon" &
  local s1=$!
  sample_pid "$tp" "02_desktop" &
  local s2=$!
  wait "$s1" "$s2"
  stop_daemon
  db1=$(db_size)
  summarize "$LOGDIR/02_daemon.tsv" "daemon" | tee -a "$LOGDIR/$name.summary"
  summarize "$LOGDIR/02_desktop.tsv" "desktop" | tee -a "$LOGDIR/$name.summary"
  echo "db_bytes_delta=$((db1-db0))" | tee -a "$LOGDIR/$name.summary"
}

run_bursts() {
  local name=03_intermittent_bursts
  echo "=== $name (${SEC}s) ===" | tee "$LOGDIR/$name.summary"
  local db0 db1
  db0=$(db_size)
  "$DAEMON" &
  sleep 2
  local dp
  dp=$(daemon_pid)
  echo "daemon_pid=$dp" | tee -a "$LOGDIR/$name.summary"
  sample_pid "$dp" "03_daemon" &
  local sp=$!
  local end=$((SECONDS + SEC - 10))
  while (( SECONDS < end )); do
    if [[ -x "$PHANTOM" ]]; then
      "$PHANTOM" --threads 4 --bursts 2 --active-secs 5 --rest-secs 2 --max-total-sec 20 || true
    fi
    sleep 10
  done
  wait $sp
  stop_daemon
  db1=$(db_size)
  summarize "$LOGDIR/03_daemon.tsv" "daemon" | tee -a "$LOGDIR/$name.summary"
  echo "db_bytes_delta=$((db1-db0))" | tee -a "$LOGDIR/$name.summary"
}

run_simultaneous() {
  local name=04_simultaneous_investigations
  echo "=== $name (${SEC}s) ===" | tee "$LOGDIR/$name.summary"
  local db0 db1
  db0=$(db_size)
  "$DAEMON" &
  sleep 2
  local dp
  dp=$(daemon_pid)
  echo "daemon_pid=$dp" | tee -a "$LOGDIR/$name.summary"
  if [[ -x "$PHANTOM" ]]; then
    "$PHANTOM" --threads 4 --bursts 6 --active-secs 5 --rest-secs 2 --max-total-sec 55 &
    local pp=$!
  fi
  sample_pid "$dp" "04_daemon" &
  local sp=$!
  wait "$sp"
  [[ -n "${pp:-}" ]] && wait "$pp" 2>/dev/null || true
  sleep 5
  stop_daemon
  db1=$(db_size)
  summarize "$LOGDIR/04_daemon.tsv" "daemon" | tee -a "$LOGDIR/$name.summary"
  echo "db_bytes_delta=$((db1-db0))" | tee -a "$LOGDIR/$name.summary"
}

stop_daemon
run_idle
stop_daemon
sleep 1
run_ui
stop_daemon
sleep 1
run_bursts
stop_daemon
sleep 1
run_simultaneous
stop_daemon

echo "Resultados en $LOGDIR"
cp -r "$LOGDIR" "$ROOT/docs/experiments/bench-phase5-latest" 2>/dev/null || true
ls -la "$LOGDIR"
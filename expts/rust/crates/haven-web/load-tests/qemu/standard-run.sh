#!/usr/bin/env bash
# standard-run.sh — Haven load-test orchestrator
# Usage: ./standard-run.sh --scenario 02-steady-state --pool 16 --rate 1000 \
#                          --target https://10.10.0.2 --label calib-1
# Requires: sudo pin.sh apply run before this. Run this as the normal user.
set -euo pipefail

# ── Constants ────────────────────────────────────────────────────────────────
GUEST_HOST="10.10.0.2"
GUEST="root@$GUEST_HOST"
K6_CORE=2 # CPU for k6
X=1       # Guest vCPU pinned core
Y=0       # QEMU helper / host core
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SESSIONS_JSON="$SCRIPT_DIR/../data/sessions.json"
K6_SCENARIOS="$SCRIPT_DIR/../k6/scenarios"
WARM_UP_MAX=180        # seconds to wait for warm-up stability
WARM_UP_STABLE=30      # seconds stability must hold
WARM_UP_RATE_DIVISOR=5 # warm-up rate = RATE / 5

# ── SSH helpers (multiplexed) ─────────────────────────────────────────────────
SSH_CTL="/tmp/haven-ctl-$$"
SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=5
  -o ControlMaster=auto -o ControlPath="$SSH_CTL"
  -o ControlPersist=15m)
g() { ssh "${SSH_OPTS[@]}" "$GUEST" "$@"; }
gscp() { scp "${SSH_OPTS[@]}" "$@"; }
gt() { ssh "${SSH_OPTS[@]}" "$GUEST" "timeout ${1:?} ${*:2}"; } # gt <secs> <cmd>

# ── Argument parsing ─────────────────────────────────────────────────────────
SCENARIO="" POOL="" RATE="" TARGET="" LABEL=""
while [[ "$#" -gt 0 ]]; do
  case $1 in
  --scenario)
    SCENARIO="$2"
    shift
    ;;
  --pool)
    POOL="$2"
    shift
    ;;
  --rate)
    RATE="$2"
    shift
    ;;
  --target)
    TARGET="$2"
    shift
    ;;
  --label)
    LABEL="$2"
    shift
    ;;
  *)
    echo "Unknown parameter: $1"
    exit 1
    ;;
  esac
  shift
done
: "${SCENARIO:?--scenario required}"
: "${POOL:?--pool required}"
: "${RATE:?--rate required}"
: "${TARGET:?--target required}"
: "${LABEL:?--label required}"

# ── Results dir (refuse to reuse) ────────────────────────────────────────────
TS=$(date +%Y%m%dT%H%M%S)
RESULTS_DIR="$SCRIPT_DIR/results/${LABEL}_pool${POOL}_${TS}"
[ -d "$RESULTS_DIR" ] && {
  echo "FATAL: $RESULTS_DIR already exists"
  exit 1
}
mkdir -p "$RESULTS_DIR"

# ── State flags ───────────────────────────────────────────────────────────────
START_SNAPSHOTS_DONE=0
TELEMETRY_STARTED=0
CLEANUP_DONE=0
K6_RC=0
ORIGINAL_EXIT=0

# ── Cleanup trap ──────────────────────────────────────────────────────────────
cleanup() {
  local exit_code=$?
  [[ $CLEANUP_DONE -eq 1 ]] && return
  CLEANUP_DONE=1
  # Preserve the real exit code (trap receives 0 for normal exit)
  [[ $ORIGINAL_EXIT -ne 0 ]] && exit_code=$ORIGINAL_EXIT

  echo ""
  echo "[CLEANUP] Running cleanup (exit=$exit_code, K6_RC=$K6_RC)..."

  # Stop telemetry first (time-sensitive)
  if [[ $TELEMETRY_STARTED -eq 1 ]]; then
    echo "[CLEANUP] Stopping telemetry..."
    "$SCRIPT_DIR/telemetry.sh" stop "$RESULTS_DIR" || true
  fi

  # End-snapshots only if start-snapshots completed
  if [[ $START_SNAPSHOTS_DONE -eq 1 ]]; then
    echo "[CLEANUP] Capturing end snapshots..."
    _take_snapshots end || true

    echo "[CLEANUP] Second clock-offset bracket..."
    _clock_offset post || true

    echo "[CLEANUP] Snapping /proc/interrupts (end)..."
    cp /proc/interrupts "$RESULTS_DIR/interrupts_end.txt" 2>/dev/null || true
    cp /proc/softirqs "$RESULTS_DIR/softirqs_end.txt" 2>/dev/null || true
  fi

  # Write k6 exit code
  echo "$K6_RC" >"$RESULTS_DIR/k6_exit_code"

  # Determine run status
  local status="ABORTED"
  if [[ $START_SNAPSHOTS_DONE -eq 1 ]]; then
    if [[ $K6_RC -eq 0 ]]; then
      status="PASS" # tentative; analyze-run.py may downgrade
    else
      status="FAIL"
    fi
  fi
  echo "$status" >"$RESULTS_DIR/status"

  # Run analysis last, non-fatal
  if [[ $START_SNAPSHOTS_DONE -eq 1 ]]; then
    if [[ -f "$SCRIPT_DIR/analyze-run.py" ]]; then
      echo "[CLEANUP] Running run validity analysis..."
      python3 "$SCRIPT_DIR/analyze-run.py" "$RESULTS_DIR" || true
    fi
    if [[ -f "$SCRIPT_DIR/analyze-snapshots.py" ]]; then
      echo "[CLEANUP] Running snapshot analysis..."
      python3 "$SCRIPT_DIR/analyze-snapshots.py" "$RESULTS_DIR" \
        >"$RESULTS_DIR/snapshot_analysis.txt" 2>&1 || true
    fi
  fi

  echo "[CLEANUP] Done. Status=$status Results=$RESULTS_DIR"

  # Close SSH multiplexer
  ssh "${SSH_OPTS[@]}" -O exit "$GUEST" 2>/dev/null || true

  exit $exit_code
}
trap 'ORIGINAL_EXIT=$?; cleanup' EXIT
trap 'ORIGINAL_EXIT=130; cleanup' INT
trap 'ORIGINAL_EXIT=143; cleanup' TERM

# ── Helpers ───────────────────────────────────────────────────────────────────
_take_snapshots() {
  local suffix=$1
  local views=(pg_stat_statements pg_stat_database pg_stat_io
    pg_stat_user_tables pg_stat_activity pg_stat_bgwriter)
  for view in "${views[@]}"; do
    gt 15 "runuser -u postgres -- psql -d haven -c '\\COPY (SELECT * FROM $view) TO stdout CSV HEADER'" \
      >"$RESULTS_DIR/${view}_${suffix}.csv" || true
  done
  # pg_stat_checkpointer exists in PG 17+; avoid dollar-quoting (expands to PID in bash)
  local has_it
  has_it=$(g "runuser -u postgres -- psql -d haven -At -c \"SELECT 1 FROM information_schema.views WHERE table_name='pg_stat_checkpointer'\"" || true)
  if [ "$has_it" = "1" ]; then
    gt 15 "runuser -u postgres -- psql -d haven -c '\\COPY (SELECT * FROM pg_stat_checkpointer) TO stdout CSV HEADER'" \
      >"$RESULTS_DIR/pg_stat_checkpointer_${suffix}.csv" || true
  fi
}

_clock_offset() {
  local tag=$1 samples=5 offsets=()
  local out="$RESULTS_DIR/clock_offset_${tag}.txt"
  for ((i = 0; i < samples; i++)); do
    local t0 g t1 offset bracket
    t0=$(date +%s%N)
    g=$(g "date +%s%N")
    t1=$(date +%s%N)
    bracket=$((t1 - t0))
    offset=$((g - (t0 + t1) / 2))
    offsets+=("$offset")
    echo "sample $i: offset=${offset}ns bracket=${bracket}ns" >>"$out"
  done
  # Sort and pick median
  local sorted median
  sorted=$(printf '%s\n' "${offsets[@]}" | sort -n)
  median=$(printf '%s\n' "${offsets[@]}" | sort -n | sed -n "$(((samples + 1) / 2))p")
  echo "median_offset_ns=$median" >>"$out"
  echo "Clock offset ($tag): median=${median}ns"
}

# ════════════════════════════════════════════════════════════════════════════
echo "========================================"
echo " Haven Load Test: $LABEL  pool=$POOL  rate=$RATE"
echo "========================================"

# ── [0] Establish SSH multiplexer ────────────────────────────────────────────
echo "[0] Opening SSH control connection..."
ssh "${SSH_OPTS[@]}" "$GUEST" true

# ── [PREFLIGHT] Environment checks ───────────────────────────────────────────
echo "[PREFLIGHT] Pinning check..."
"$SCRIPT_DIR/pin.sh" check || {
  echo "FATAL: CPU pinning not applied. Run: sudo ./pin.sh apply"
  exit 1
}

echo "[PREFLIGHT] Host swap check..."
[[ -z "$(swapon --show 2>/dev/null)" ]] || {
  echo "FATAL: swap is active on host"
  exit 1
}

echo "[PREFLIGHT] QEMU PID..."
VM_PID_FILE="$SCRIPT_DIR/vm.pid"
if [[ -f "$VM_PID_FILE" ]]; then
  QPID=$(cat "$VM_PID_FILE")
else
  QPID=$(pgrep -f 'qemu-system-x86_64.*haven-bench\.img' | head -n1 || true)
fi
[[ -n "$QPID" ]] && kill -0 "$QPID" 2>/dev/null || {
  echo "FATAL: QEMU process not found (vm.pid=$VM_PID_FILE)"
  exit 1
}
echo "  QEMU PID: $QPID"

echo "[PREFLIGHT] Guest dump file..."
gt 10 "test -f /var/lib/postgresql/dump.fc" ||
  {
    echo "FATAL: /var/lib/postgresql/dump.fc not found in guest"
    exit 1
  }

echo "[PREFLIGHT] Sessions file..."
[[ -f "$SESSIONS_JSON" ]] || {
  echo "FATAL: $SESSIONS_JSON not found"
  exit 1
}

# ── [META] Environment fingerprint ───────────────────────────────────────────
echo "[META] Capturing environment..."
{
  echo "label=$LABEL"
  echo "scenario=$SCENARIO"
  echo "pool=$POOL"
  echo "rate=$RATE"
  echo "target=$TARGET"
  echo "timestamp=$TS"
  echo "cmdline=$0 $*"
  echo "host_uname=$(uname -r)"
  echo "host_governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
  echo "host_turbo_disabled=$(cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || cat /sys/devices/system/cpu/cpufreq/boost 2>/dev/null || echo unknown)"
  echo "ac_online=$(cat /sys/class/power_supply/AC*/online 2>/dev/null | head -1 || echo unknown)"
  echo "host_lscpu=$(lscpu -e 2>/dev/null | tr '\n' '|')"
  echo "git_sha=$(git -C "$SCRIPT_DIR" rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "sessions_sha256=$(sha256sum "$SESSIONS_JSON" | cut -d' ' -f1)"
  echo "guest_sampler_sha256=$(sha256sum "$SCRIPT_DIR/guest-sampler.sh" | cut -d' ' -f1)"
  gt 10 "sha256sum /var/lib/postgresql/dump.fc | cut -d' ' -f1" |
    {
      read -r h
      echo "dump_sha256=$h"
    }
  gt 10 "uname -r" | {
    read -r v
    echo "guest_uname=$v"
  }
  gt 10 "nginx -v 2>&1" | {
    read -r v
    echo "nginx_version=$v"
  }
  gt 10 "runuser -u postgres -- psql -At -c 'SELECT version()'" |
    {
      read -r v
      echo "pg_version=$v"
    }
  gt 10 "runuser -u postgres -- psql -d haven -At -c 'SHOW io_method'" |
    {
      read -r v
      echo "pg_io_method=$v"
    }
  gt 10 "cat /proc/cmdline" | {
    read -r v
    echo "guest_kcmdline=$v"
  }
} >"$RESULTS_DIR/run_meta.txt"
echo "  Saved run_meta.txt"

# ── [1] Stop app and truncate logs ───────────────────────────────────────────
echo "[1] Stopping app and truncating logs..."
gt 10 "systemctl stop haven-web"
g "truncate -s 0 /var/log/nginx/access.log && nginx -s reopen"
gt 10 "journalctl --vacuum-time=1s"

# ── [2] Update parameters ────────────────────────────────────────────────────
echo "[2] Updating parameters (pool=$POOL)..."
gt 10 "sed -i 's/^DB_POOL_SIZE.*/DB_POOL_SIZE=$POOL/' /etc/haven/haven.env"
gt 10 "sed -i 's/^max_connections.*/max_connections = $((POOL + 10))/' /etc/postgresql/18/main/conf.d/haven.conf"

# ── [3] Restore DB ───────────────────────────────────────────────────────────
echo "[3] Restarting Postgres and restoring DB..."
gt 15 "systemctl restart postgresql"
gt 10 "runuser -u postgres -- dropdb --force --if-exists haven"
gt 10 "runuser -u postgres -- createdb -O haven haven"
echo "  pg_restore (may take up to 5 minutes)..."
gt 300 "runuser -u postgres -- pg_restore -d haven /var/lib/postgresql/dump.fc" ||
  {
    echo "FATAL: pg_restore failed or timed out"
    exit 1
  }
gt 10 "runuser -u postgres -- psql -d haven -c 'CREATE EXTENSION IF NOT EXISTS pg_stat_statements;'"
gt 30 "runuser -u postgres -- psql -d haven -c 'VACUUM ANALYZE;'"
gt 10 "runuser -u postgres -- psql -d haven -c 'CHECKPOINT;'"
echo "  DB restored."

# ── [4] Start app and wait for readiness ─────────────────────────────────────
echo "[4] Starting app and polling readiness (30s max)..."
gt 10 "systemctl start haven-web"
ready=0
for i in $(seq 1 30); do
  if curl -sf -k --max-time 3 "$TARGET/auth/sign-in" >/dev/null 2>&1; then
    ready=1
    echo "  Ready after ${i}s"
    break
  fi
  sleep 1
done
[[ $ready -eq 1 ]] || {
  echo "FATAL: App failed to start after 30s"
  exit 1
}

# ── [POOL CHECK] Validate effective pool size in app logs ────────────────────
echo "[POOL] Verifying effective pool size..."
# App logs: 'Effective DB_POOL_SIZE: N' (colon-space, not equals)
LOGGED_VAL=$(gt 10 "journalctl -u haven-web -b --no-pager | grep -o 'Effective DB_POOL_SIZE: [0-9]*' | tail -1 | grep -o '[0-9]*$'" || true)
[ "$LOGGED_VAL" = "$POOL" ] ||
  {
    echo "FATAL: pool mismatch (app logged '$LOGGED_VAL', expected '$POOL'). Check RUST_LOG level."
    exit 1
  }
echo "  Pool OK: Effective DB_POOL_SIZE: $LOGGED_VAL"

# ── [5] Session gate ─────────────────────────────────────────────────────────
echo "[5] Session gate (10 random sessions)..."
mapfile -t TOKENS < <(jq -r '.[].session_token' "$SESSIONS_JSON" | shuf -n 10)
for tok in "${TOKENS[@]}"; do
  code=$(curl -sk -o /dev/null -w '%{http_code}' \
    --cookie "__Host-sid=$tok" --max-time 5 "$TARGET/offers")
  [ "$code" = "200" ] ||
    {
      echo "FATAL: session gate got HTTP $code for token ...${tok: -8}"
      exit 1
    }
done
echo "  All 10 sessions valid."

# ── [6] Warm-up ──────────────────────────────────────────────────────────────
echo "[6] Warm-up (windowed stability: hit_ratio>=99%, disk_reads<=100/s for ${WARM_UP_STABLE}s, max ${WARM_UP_MAX}s)..."

WARM_RATE=$((RATE / WARM_UP_RATE_DIVISOR))
[[ "$WARM_RATE" -ge 1 ]] || WARM_RATE=1

taskset -c "$K6_CORE" k6 run --insecure-skip-tls-verify \
  -e BASE_URL="$TARGET" -e TARGET="$TARGET" -e RATE="$WARM_RATE" -e DURATION="600s" \
  "$K6_SCENARIOS/warm-up.js" >"$RESULTS_DIR/warmup.log" 2>&1 &
WARM_PID=$!

# Establish the initial cumulative PostgreSQL counters.
# All subsequent measurements use deltas from these counters so the
# stability decision reflects the current warm-up window, not activity
# that happened before warm-up began.
PG_COUNTERS=$(gt 10 "runuser -u postgres -- psql -d haven -At -c \
  \"SELECT blks_hit, blks_read FROM pg_stat_database WHERE datname='haven'\"" ||
  echo "0|0")

PREV_HIT=$(echo "$PG_COUNTERS" | cut -d'|' -f1)
PREV_READ=$(echo "$PG_COUNTERS" | cut -d'|' -f2)

stable_secs=0
elapsed=0

while true; do
  sleep 5
  elapsed=$((elapsed + 5))

  # Current PostgreSQL cumulative counters.
  PG_COUNTERS=$(gt 10 "runuser -u postgres -- psql -d haven -At -c \
    \"SELECT blks_hit, blks_read FROM pg_stat_database WHERE datname='haven'\"" ||
    echo "$PREV_HIT|$PREV_READ")

  CUR_HIT=$(echo "$PG_COUNTERS" | cut -d'|' -f1)
  CUR_READ=$(echo "$PG_COUNTERS" | cut -d'|' -f2)

  # Calculate deltas for this 5-second observation window.
  DELTA_HIT=$((CUR_HIT - PREV_HIT))
  DELTA_READ=$((CUR_READ - PREV_READ))

  # Guard against unexpected counter resets.
  if [[ "$DELTA_HIT" -lt 0 || "$DELTA_READ" -lt 0 ]]; then
    echo "  [+${elapsed}s] PostgreSQL counters reset; resetting stability window"
    PREV_HIT=$CUR_HIT
    PREV_READ=$CUR_READ
    stable_secs=0
    continue
  fi

  PREV_HIT=$CUR_HIT
  PREV_READ=$CUR_READ

  TOTAL=$((DELTA_HIT + DELTA_READ))

  if [[ "$TOTAL" -gt 0 ]]; then
    # Integer percentage for the current 5-second window only.
    HIT_PCT=$((DELTA_HIT * 100 / TOTAL))
  else
    # No database block activity is not evidence of cache convergence.
    HIT_PCT=0
  fi

  # Measure vda reads over a 1-second interval.
  DISK_RD=$(gt 10 "awk '\$3==\"vda\"{print \$6}' /proc/diskstats" || echo 0)
  sleep 1
  DISK_RD2=$(gt 10 "awk '\$3==\"vda\"{print \$6}' /proc/diskstats" || echo 0)
  DISK_DELTA=$((DISK_RD2 - DISK_RD))

  printf "  [+%ds] window_hit_ratio=%d%% db_hits=%d db_reads=%d disk_reads_per_sec=%d\n" \
    "$elapsed" "$HIT_PCT" "$DELTA_HIT" "$DELTA_READ" "$DISK_DELTA"

  if [[ "$HIT_PCT" -ge 99 ]] && [[ "$DISK_DELTA" -le 100 ]]; then
    stable_secs=$((stable_secs + 5))
    echo "  Stable for ${stable_secs}s..."
    if [[ "$stable_secs" -ge "$WARM_UP_STABLE" ]]; then
      echo "  Warm-up stable!"
      break
    fi
  else
    if [[ "$stable_secs" -gt 0 ]]; then
      echo "  Stability lost; resetting stability timer."
    fi
    stable_secs=0
  fi

  if [[ "$elapsed" -ge "$WARM_UP_MAX" ]]; then
    kill "$WARM_PID" 2>/dev/null || true
    wait "$WARM_PID" 2>/dev/null || true
    echo "FATAL: Warm-up never reached stability in ${WARM_UP_MAX}s"
    exit 1
  fi
done

echo "  Warm-up done in ${elapsed}s"

# SIGTERM the warm-up k6; non-zero exit from it is normal.
kill "$WARM_PID" 2>/dev/null || true
wait "$WARM_PID" 2>/dev/null || true

# ── [7] Deploy guest sampler ─────────────────────────────────────────────────
echo "[7] Deploying guest-sampler.sh..."
gscp "$SCRIPT_DIR/guest-sampler.sh" "$GUEST:/root/guest-sampler.sh"
gt 5 "chmod +x /root/guest-sampler.sh"
echo "guest_sampler_deployed=$(sha256sum "$SCRIPT_DIR/guest-sampler.sh" | cut -d' ' -f1)" \
  >>"$RESULTS_DIR/run_meta.txt"

# ── [8] Start snapshots ───────────────────────────────────────────────────────
echo "[8] Capturing start snapshots..."
_take_snapshots start
START_SNAPSHOTS_DONE=1

# ── [9] Snapshot /proc/interrupts (pre) ──────────────────────────────────────
echo "[9] Snapping /proc/interrupts (start)..."
cp /proc/interrupts "$RESULTS_DIR/interrupts_start.txt"
cp /proc/softirqs "$RESULTS_DIR/softirqs_start.txt"

# ── [10] Clock offset (pre) ──────────────────────────────────────────────────
echo "[10] Bracketing clock offset (pre-run)..."
_clock_offset pre

# ── [11] Start telemetry ──────────────────────────────────────────────────────
echo "[11] Starting telemetry..."
"$SCRIPT_DIR/telemetry.sh" start "$RESULTS_DIR" "$QPID" "$X" "$Y" "$K6_CORE"
TELEMETRY_STARTED=1
date +%s.%N >"$RESULTS_DIR/k6_start_time"

# ── [12] Run load test ────────────────────────────────────────────────────────
echo "[12] Running load test (taskset -c $K6_CORE k6)..."
set +e
taskset -c "$K6_CORE" k6 run --insecure-skip-tls-verify \
  --summary-export "$RESULTS_DIR/k6_summary.json" \
  -e BASE_URL="$TARGET" -e TARGET="$TARGET" -e RATE="$RATE" \
  "$K6_SCENARIOS/$SCENARIO.js" \
  2>&1 | tee "$RESULTS_DIR/k6_output.log"
K6_RC=${PIPESTATUS[0]}
set -e
date +%s.%N >"$RESULTS_DIR/k6_end_time"
echo "K6 exit code: $K6_RC"

# cleanup trap fires here and handles everything else

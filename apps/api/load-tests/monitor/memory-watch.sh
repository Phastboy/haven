#!/usr/bin/env bash
# ─── Haven Load Test — Process Memory Monitor ────────────────────────────────
# Tracks the Bun API process memory (RSS + VmRSS) every N seconds and writes
# a timestamped CSV for analysis. Run this BEFORE starting any k6 scenario.
#
# Usage:
#   ./apps/api/load-tests/monitor/memory-watch.sh [INTERVAL_SEC] [PORT]
#
# Defaults: interval=5s, port=3000
#
# Output : apps/api/load-tests/results/memory-<timestamp>.csv
#          apps/api/load-tests/results/memory-<timestamp>.log  (human-readable)
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

INTERVAL="${1:-5}"
PORT="${2:-3000}"
RESULTS_DIR="apps/api/load-tests/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
CSV="${RESULTS_DIR}/memory-${TIMESTAMP}.csv"
LOG="${RESULTS_DIR}/memory-${TIMESTAMP}.log"

mkdir -p "$RESULTS_DIR"

# Find the PID of whatever is listening on PORT
find_pid() {
  # Try lsof first (common on Linux/macOS)
  local pid
  pid=$(lsof -ti :"$PORT" 2>/dev/null | head -1) && echo "$pid" && return 0
  # Fallback: ss + awk
  pid=$(ss -lpnt "sport = :${PORT}" 2>/dev/null \
    | grep -oP 'pid=\K[0-9]+' | head -1) && echo "$pid" && return 0
  return 1
}

PID=$(find_pid) || {
  echo "❌  No process found listening on port ${PORT}. Is the API running?"
  exit 1
}

echo "🔍  Monitoring PID ${PID} (port ${PORT}) every ${INTERVAL}s"
echo "📄  CSV : ${CSV}"
echo "📄  LOG : ${LOG}"
echo "    Press Ctrl+C to stop."
echo ""

# CSV header
echo "timestamp_epoch,timestamp_iso,pid,rss_kb,vsz_kb,cpu_pct,threads" > "$CSV"

# Log header
{
  echo "Haven API Memory Monitor"
  echo "PID: ${PID}  Port: ${PORT}  Interval: ${INTERVAL}s"
  echo "Started: $(date -Iseconds)"
  echo "─────────────────────────────────────────────────────────────────"
  printf "%-20s  %10s  %10s  %8s  %8s\n" "Timestamp" "RSS (KB)" "VSZ (KB)" "CPU%" "Threads"
  echo "─────────────────────────────────────────────────────────────────"
} | tee "$LOG"

cleanup() {
  echo ""
  echo "──────────────────────────────────────"
  echo "Monitor stopped: $(date -Iseconds)"
  echo "Results: ${CSV}"
  echo ""
  # Print min/max/avg RSS
  if command -v awk &>/dev/null; then
    awk -F',' 'NR>1 {
      rss=$4; sum+=rss; n++;
      if(n==1||rss<mn) mn=rss;
      if(rss>mx) mx=rss;
    }
    END {
      printf "RSS  min=%d KB  max=%d KB  avg=%d KB\n", mn, mx, (n>0?sum/n:0)
    }' "$CSV"
  fi
}
trap cleanup EXIT

while true; do
  # Re-check PID still exists
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "⚠️  PID ${PID} no longer running. Stopping monitor."
    break
  fi

  NOW_EPOCH=$(date +%s)
  NOW_ISO=$(date -Iseconds)

  # Read from /proc for accuracy
  if [[ -f "/proc/${PID}/status" ]]; then
    RSS_KB=$(awk '/^VmRSS:/{print $2}' "/proc/${PID}/status" 2>/dev/null || echo 0)
    VSZ_KB=$(awk '/^VmSize:/{print $2}' "/proc/${PID}/status" 2>/dev/null || echo 0)
    THREADS=$(awk '/^Threads:/{print $2}' "/proc/${PID}/status" 2>/dev/null || echo 0)
  else
    RSS_KB=$(ps -o rss= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0)
    VSZ_KB=$(ps -o vsz= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0)
    THREADS=0
  fi

  # CPU from ps (sampled, not cumulative)
  CPU_PCT=$(ps -o %cpu= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0)

  # Append to CSV
  echo "${NOW_EPOCH},${NOW_ISO},${PID},${RSS_KB},${VSZ_KB},${CPU_PCT},${THREADS}" >> "$CSV"

  # Human log line
  printf "%-20s  %10s  %10s  %8s  %8s\n" \
    "$NOW_ISO" "${RSS_KB} KB" "${VSZ_KB} KB" "${CPU_PCT}%" "${THREADS}" | tee -a "$LOG"

  sleep "$INTERVAL"
done

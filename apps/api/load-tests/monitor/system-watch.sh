#!/usr/bin/env bash
# ─── System Saturation Monitor ────────────────────────────────────────────────
# Captures CPU, RSS, DB connections, and open FDs every 5 seconds.
# Correlates system resources to load test traffic to identify bottlenecks.
#
# Output: load-tests/results/system-saturation-<timestamp>.csv
# ──────────────────────────────────────────────────────────────────────────────

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$DIR/../results"
mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUT_FILE="$RESULTS_DIR/system-saturation-$TIMESTAMP.csv"

# 1. Find the target process
TARGET_PID=$1
if [ -z "$TARGET_PID" ]; then
  # Try to auto-detect the Bun API process
  # Filtering out grep itself and any test runners
  TARGET_PID=$(pgrep -f "bun.*apps/api/src/index.ts" | head -n 1)
  
  if [ -z "$TARGET_PID" ]; then
    echo "❌ Could not auto-detect API process."
    echo "Usage: $0 [pid]"
    exit 1
  fi
fi

echo "▶ Monitoring PID: $TARGET_PID"
echo "▶ Output: $OUT_FILE"
echo "timestamp,rss_mb,cpu_pct,db_active_conn,open_fds" > "$OUT_FILE"

# For CPU calculation
CLK_TCK=$(getconf CLK_TCK)
PREV_SYS_TIME=0
PREV_PROC_TIME=0

# DB config
DB_URL=${DATABASE_URL:-"postgresql://user:password@localhost:5433/mydb"}

while sleep 5; do
  # Check if process is still alive
  if ! kill -0 "$TARGET_PID" 2>/dev/null; then
    echo "Process $TARGET_PID terminated."
    break
  fi
  
  NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  # --- 1. Memory (RSS) ---
  # VmRSS from /proc/[pid]/status is in kB
  RSS_KB=$(grep -s "VmRSS" "/proc/$TARGET_PID/status" | awk '{print $2}')
  RSS_MB=$((RSS_KB / 1024))

  # --- 2. CPU % ---
  # /proc/[pid]/stat fields 14 (utime) and 15 (stime) in jiffies
  STAT=($(cat "/proc/$TARGET_PID/stat" 2>/dev/null))
  PROC_TIME=$((${STAT[13]} + ${STAT[14]}))
  
  # Total system time from /proc/stat
  SYS_CPU=($(head -n 1 /proc/stat))
  SYS_TIME=$((${SYS_CPU[1]} + ${SYS_CPU[2]} + ${SYS_CPU[3]} + ${SYS_CPU[4]} + ${SYS_CPU[5]} + ${SYS_CPU[6]} + ${SYS_CPU[7]}))
  
  if [ $PREV_SYS_TIME -gt 0 ]; then
    SYS_DELTA=$((SYS_TIME - PREV_SYS_TIME))
    PROC_DELTA=$((PROC_TIME - PREV_PROC_TIME))
    
    if [ $SYS_DELTA -gt 0 ]; then
      # Multiply by 1000 for 1 decimal place integer division
      CPU_PCT=$((1000 * PROC_DELTA / SYS_DELTA))
      # Format as float e.g. 12.3
      CPU_STR=$(printf "%d.%d" $((CPU_PCT / 10)) $((CPU_PCT % 10)))
    else
      CPU_STR="0.0"
    fi
  else
    CPU_STR="0.0"
  fi
  
  PREV_SYS_TIME=$SYS_TIME
  PREV_PROC_TIME=$PROC_TIME

  # --- 3. Open File Descriptors ---
  # Total FDs (sockets, files, etc)
  FDS=$(ls -1 "/proc/$TARGET_PID/fd" 2>/dev/null | wc -l)

  # --- 4. DB Connections ---
  # Query pg_stat_activity for active connections to 'mydb'
  DB_CONN=$(psql "$DB_URL" -t -A -c "SELECT count(*) FROM pg_stat_activity WHERE datname = 'mydb' AND state = 'active';" 2>/dev/null || echo "0")

  # --- Output ---
  echo "$NOW,$RSS_MB,$CPU_STR,$DB_CONN,$FDS" >> "$OUT_FILE"
  # Optional: echo to stdout for visibility
  # echo "[$NOW] RSS: ${RSS_MB}MB | CPU: ${CPU_STR}% | DB: $DB_CONN | FDs: $FDS"
done

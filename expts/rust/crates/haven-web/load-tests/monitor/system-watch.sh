#!/usr/bin/env bash
# ─── System Saturation Monitor ───────────────────────────────────────────────
# Logs CPU, Memory, and Network connections to a CSV.
# Run this in the background while k6 executes.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUT_FILE="${RESULTS_DIR}/system-saturation-${TIMESTAMP}.csv"

mkdir -p "$RESULTS_DIR"

echo "timestamp,cpu_usr,cpu_sys,cpu_idl,cpu_iow,mem_free_mb,mem_buff_mb,mem_cache_mb,tcp_estab,tcp_time_wait" > "$OUT_FILE"

while true; do
  ts=$(date +%s)
  
  # CPU & Mem from vmstat (skipping headers)
  vm_line=$(vmstat -SM 1 2 | tail -1 | tr -s ' ')
  
  mem_free=$(echo "$vm_line" | cut -d' ' -f5)
  mem_buff=$(echo "$vm_line" | cut -d' ' -f6)
  mem_cache=$(echo "$vm_line" | cut -d' ' -f7)
  
  cpu_usr=$(echo "$vm_line" | cut -d' ' -f14)
  cpu_sys=$(echo "$vm_line" | cut -d' ' -f15)
  cpu_idl=$(echo "$vm_line" | cut -d' ' -f16)
  cpu_iow=$(echo "$vm_line" | cut -d' ' -f17)

  # Network state (Linux specific, fallback to 0 if not available)
  if command -v ss &>/dev/null; then
    tcp_estab=$(ss -s | grep TCP: | grep -o 'estab [0-9]*' | awk '{print $2}' || echo 0)
    tcp_time_wait=$(ss -s | grep TCP: | grep -o 'timewait [0-9]*' | awk '{print $2}' || echo 0)
  else
    tcp_estab=0
    tcp_time_wait=0
  fi

  echo "${ts},${cpu_usr},${cpu_sys},${cpu_idl},${cpu_iow},${mem_free},${mem_buff},${mem_cache},${tcp_estab},${tcp_time_wait}" >> "$OUT_FILE"
  
  sleep 5
done

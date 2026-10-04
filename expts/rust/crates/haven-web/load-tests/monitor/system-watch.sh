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
  vm_line=$(vmstat -SM 1 2 | tail -1)
  read -r -a vm_fields <<< "$vm_line"
  
  mem_free="${vm_fields[3]:-0}"
  mem_buff="${vm_fields[4]:-0}"
  mem_cache="${vm_fields[5]:-0}"
  
  cpu_usr="${vm_fields[12]:-0}"
  cpu_sys="${vm_fields[13]:-0}"
  cpu_idl="${vm_fields[14]:-0}"
  cpu_iow="${vm_fields[15]:-0}"

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

#!/usr/bin/env bash
# ─── Haven Load Test — Full Suite Orchestrator ───────────────────────────────
# Runs every scenario in order with memory monitoring in the background.
# Results land in apps/api/load-tests/results/.
#
# Usage (from repo root):
#   ./apps/api/load-tests/run-all.sh
#
# Options (env vars):
#   SKIP_SMOKE=1          skip scenario 01
#   SKIP_NORMAL=1         skip scenario 02
#   SKIP_HIGH_LOAD=1      skip scenario 03
#   SKIP_SPIKE=1          skip scenario 04
#   SKIP_SOAK=1           skip scenario 05
#   SKIP_CAPACITY=1       skip scenario 06
#   SKIP_OHA=1            skip oha baseline
#   MAX_VUS=5000          override capacity ceiling (default 5000)
#   BASE_URL=http://...   override API base URL
#
# Example — just run smoke + capacity:
#   SKIP_NORMAL=1 SKIP_HIGH_LOAD=1 SKIP_SPIKE=1 SKIP_SOAK=1 SKIP_OHA=1 ./apps/api/load-tests/run-all.sh
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BASE_URL="${BASE_URL:-http://localhost:3000/api}"

mkdir -p "$RESULTS_DIR"

banner() {
  local cols
  cols=$(tput cols 2>/dev/null || echo 60)
  local line
  line=$(printf '─%.0s' $(seq 1 "$cols"))
  echo ""
  echo "$line"
  printf "  🚀  %s\n" "$1"
  echo "$line"
}

check_deps() {
  local missing=0
  for cmd in k6 oha jq lsof; do
    if ! command -v "$cmd" &>/dev/null; then
      echo "❌  Missing dependency: $cmd"
      missing=1
    fi
  done
  [[ $missing -eq 1 ]] && exit 1
  echo "✅  All dependencies found (k6, oha, jq, lsof)"
}

check_api() {
  echo -n "🔎  Checking API at ${BASE_URL}/health ... "
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/health") || true
  if [[ "$status" == "200" ]]; then
    echo "✅  online (HTTP 200)"
  else
    echo "❌  HTTP ${status} — is the API running?"
    exit 1
  fi
}

run_k6() {
  local scenario="$1"
  local label="$2"
  local out="${RESULTS_DIR}/k6-${label}-${TIMESTAMP}.json"

  banner "k6 — ${label}"
  k6 run \
    --env BASE_URL="$BASE_URL" \
    --summary-export="$out" \
    "${SCRIPT_DIR}/k6/scenarios/${scenario}" \
    2>&1 | tee "${RESULTS_DIR}/k6-${label}-${TIMESTAMP}.log"

  echo ""
  echo "  📊 Summary export → ${out}"
}

# ─── Pre-flight ───────────────────────────────────────────────────────────────
check_deps
check_api

# ─── Start system saturation monitor in background ───────────────────────────
banner "System Monitor — starting"
bash "${SCRIPT_DIR}/monitor/system-watch.sh" &
MONITOR_PID=$!
echo "  System monitor PID: ${MONITOR_PID}"
sleep 2  # let it log an initial sample

# ─── Scenarios ────────────────────────────────────────────────────────────────
[[ "${SKIP_SMOKE:-0}" != "1" ]]       && run_k6 "01-smoke.js"       "01-smoke"
[[ "${SKIP_NORMAL:-0}" != "1" ]]      && run_k6 "02-normal.js"      "02-normal"
[[ "${SKIP_HIGH_LOAD:-0}" != "1" ]]   && run_k6 "03-high-load.js"   "03-high-load"
[[ "${SKIP_SPIKE:-0}" != "1" ]]       && run_k6 "04-spike.js"       "04-spike"
[[ "${SKIP_SOAK:-0}" != "1" ]]        && run_k6 "05-soak.js"        "05-soak"
[[ "${SKIP_CAPACITY:-0}" != "1" ]]    && \
  MAX_VUS="${MAX_VUS:-5000}" k6 run \
    --env BASE_URL="$BASE_URL" \
    --env MAX_VUS="${MAX_VUS:-5000}" \
    --summary-export="${RESULTS_DIR}/k6-06-capacity-${TIMESTAMP}.json" \
    "${SCRIPT_DIR}/k6/scenarios/06-capacity.js" \
    2>&1 | tee "${RESULTS_DIR}/k6-06-capacity-${TIMESTAMP}.log"

# ─── oha baseline ─────────────────────────────────────────────────────────────
[[ "${SKIP_OHA:-0}" != "1" ]] && {
  banner "oha — Baseline & Break-Point"
  bash "${SCRIPT_DIR}/oha/run-baseline.sh" "$BASE_URL"
}

# ─── Stop system monitor ──────────────────────────────────────────────────────
banner "Stopping system monitor"
kill "$MONITOR_PID" 2>/dev/null || true
wait "$MONITOR_PID" 2>/dev/null || true

# ─── Final summary ────────────────────────────────────────────────────────────
banner "ALL DONE — Results in ${RESULTS_DIR}/"
echo ""
echo "  k6 JSON summaries:"
ls -1 "${RESULTS_DIR}"/k6-*-"${TIMESTAMP}".json 2>/dev/null | sed 's/^/    /'

echo ""
echo "  oha JSON results:"
ls -1 "${RESULTS_DIR}"/oha-*-"${TIMESTAMP}".json 2>/dev/null | sed 's/^/    /'

echo ""
echo "  System Saturation CSV:"
ls -1 "${RESULTS_DIR}"/system-saturation-*.csv 2>/dev/null | tail -1 | sed 's/^/    /'

echo ""
echo "  Quick RPS snapshot (k6 — http_reqs):"
for f in "${RESULTS_DIR}"/k6-*-"${TIMESTAMP}".json; do
  name=$(basename "$f" .json | sed "s/-${TIMESTAMP}//")
  jq -r '"    \(input_filename): http_reqs=\(.metrics.http_reqs.values.rate | round // "n/a") rps, p95=\(.metrics.http_req_duration.values["p(95)"] | round // "n/a")ms"' \
    "$f" 2>/dev/null | sed "s|${RESULTS_DIR}/||" || true
done

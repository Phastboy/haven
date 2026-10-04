#!/usr/bin/env bash
# ─── Haven Rust Load Test — Full Suite Orchestrator ───────────────────────────────
# Runs every scenario in order with system monitoring in the background.
# Results land in results/.
#
# Usage (from load-tests root):
#   ./run-all.sh
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"

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
  if ! command -v k6 &>/dev/null; then
    echo "❌  Missing dependency: k6"
    exit 1
  fi
  echo "✅  Core dependency found (k6)"
}

check_api() {
  if ! command -v curl &>/dev/null; then
    echo "⚠️  Skipping API check (curl missing)"
    return
  fi
  echo -n "🔎  Checking API at ${BASE_URL}/ ... "
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/") || true
  if [[ "$status" == "200" || "$status" == "303" ]]; then
    echo "✅  online"
  else
    echo "❌  Failed to connect — is the API running?"
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
MONITOR_PID=""
if command -v vmstat &>/dev/null; then
  banner "System Monitor — starting"
  bash "${SCRIPT_DIR}/monitor/system-watch.sh" &
  MONITOR_PID=$!
  trap 'echo ""; banner "Stopping system monitor"; kill "$MONITOR_PID" 2>/dev/null || true; wait "$MONITOR_PID" 2>/dev/null || true' EXIT
  echo "  System monitor PID: ${MONITOR_PID}"
  sleep 2  # let it log an initial sample
else
  echo "⚠️  Skipping System Monitor (vmstat missing)"
fi

# ─── Scenarios ────────────────────────────────────────────────────────────────
[[ "${SKIP_SMOKE:-0}" != "1" ]]       && run_k6 "01-smoke.js"       "01-smoke"
[[ "${SKIP_STEADY_STATE:-0}" != "1" ]] && run_k6 "02-steady-state.js" "02-steady-state"
[[ "${SKIP_CAPACITY:-0}" != "1" ]]    && run_k6 "03-capacity.js"    "03-capacity"
[[ "${SKIP_AUTHORIZATION:-0}" != "1" ]] && run_k6 "04-authorization.js" "04-authorization"

# ─── Stop system monitor (handled by EXIT trap) ─────────────────────────────

# ─── Final summary ────────────────────────────────────────────────────────────
banner "ALL DONE — Results in ${RESULTS_DIR}/"
echo ""
echo "  k6 JSON summaries:"
ls -1 "${RESULTS_DIR}"/k6-*-"${TIMESTAMP}".json 2>/dev/null | sed 's/^/    /'

echo ""
echo "  System Saturation CSV:"
ls -1 "${RESULTS_DIR}"/system-saturation-*.csv 2>/dev/null | tail -1 | sed 's/^/    /'

echo ""
echo "  Quick RPS snapshot (k6 — http_reqs):"
if command -v jq &>/dev/null; then
  for f in "${RESULTS_DIR}"/k6-*-"${TIMESTAMP}".json; do
    name=$(basename "$f" .json | sed "s/-${TIMESTAMP}//")
    jq -r '"    \(input_filename): http_reqs=\(.metrics.http_reqs.values.rate | round // "n/a") rps, p95=\(.metrics.http_req_duration.values["p(95)"] | round // "n/a")ms"' \
      "$f" 2>/dev/null | sed "s|${RESULTS_DIR}/||" || true
  done
else
  echo "    (jq not installed, skipping snapshot parsing)"
fi

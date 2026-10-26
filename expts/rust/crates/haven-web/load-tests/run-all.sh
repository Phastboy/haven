#!/usr/bin/env bash
# ─── Haven Rust Load Test — Go Suite Orchestrator ─────────────────────────────────
# Builds the Go runner, then runs smoke, steady-state, authorization, v0.5 and v0.6.
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
SESSIONS_FILE="${SESSIONS_FILE:-${SCRIPT_DIR}/data/sessions.json}"
RUNNER="${RESULTS_DIR}/haven-load"
RATE="${RATE:-50}"
WORKERS="${WORKERS:-16}"
STEADY_DURATION="${STEADY_DURATION:-5m}"
FEATURE_DURATION="${FEATURE_DURATION:-60s}"

cd "$SCRIPT_DIR"

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
  if ! command -v go &>/dev/null; then
    echo "❌  Missing dependency: Go"
    exit 1
  fi
  echo "✅  Core dependency found (Go)"
}

check_api() {
  if ! command -v curl &>/dev/null; then
    echo "⚠️  Skipping API check (curl missing)"
    return
  fi
  echo -n "🔎  Checking API at ${BASE_URL}/health ... "
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/health") || true
  if [[ "$status" == "200" ]]; then
    echo "✅  online"
  else
    echo "❌  Failed to connect — is the API running?"
    exit 1
  fi
}

run_scenario() {
  local scenario="$1" rate="$2" duration="$3" label="$4"
  local summary="${RESULTS_DIR}/go-${label}-${TIMESTAMP}.json"
  local log="${RESULTS_DIR}/go-${label}-${TIMESTAMP}.log"

  banner "Go load runner — ${label}"
  "$RUNNER" \
    --base-url "$BASE_URL" \
    --scenario "$scenario" \
    --sessions "$SESSIONS_FILE" \
    --rate "$rate" \
    --duration "$duration" \
    --workers "$WORKERS" \
    --summary "$summary" \
    2>&1 | tee "$log"

  echo ""
  echo "  📊 Summary export → ${summary}"
}

# ─── Pre-flight ───────────────────────────────────────────────────────────────
check_deps
check_api
if [[ ! -f "$SESSIONS_FILE" ]]; then
  echo "❌  Missing sessions fixture: $SESSIONS_FILE"
  echo "    Generate one with: cargo run -p load-test-support -- --users 10000"
  exit 1
fi
banner "Build Go load runner"
(cd "${SCRIPT_DIR}/go" && go build -o "$RUNNER" ./cmd/haven-load)

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
[[ "${SKIP_SMOKE:-0}" != "1" ]] && run_scenario "smoke" 1 1s "01-smoke"
[[ "${SKIP_STEADY_STATE:-0}" != "1" ]] && run_scenario "steady" "$RATE" "$STEADY_DURATION" "02-steady"
[[ "${SKIP_AUTHORIZATION:-0}" != "1" ]] && run_scenario "authorization" 10 "$FEATURE_DURATION" "03-authorization"
[[ "${SKIP_V05:-0}" != "1" ]] && run_scenario "v05" 10 "$FEATURE_DURATION" "04-v05-search-filter"
[[ "${SKIP_V06:-0}" != "1" ]] && run_scenario "v06" 1 "$FEATURE_DURATION" "05-v06-save-follow"

# ─── Stop system monitor (handled by EXIT trap) ─────────────────────────────

# ─── Final summary ────────────────────────────────────────────────────────────
banner "ALL DONE — Results in ${RESULTS_DIR}/"
echo ""
echo "  Go JSON summaries:"
ls -1 "${RESULTS_DIR}"/go-*-"${TIMESTAMP}".json 2>/dev/null | sed 's/^/    /'

echo ""
echo "  System Saturation CSV:"
ls -1 "${RESULTS_DIR}"/system-saturation-*.csv 2>/dev/null | tail -1 | sed 's/^/    /'

echo ""
echo "  Per-route p50/p95/p99, request counts and correctness checks are in each JSON summary."

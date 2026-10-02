#!/usr/bin/env bash
# ─── Haven Load Test — oha Baseline & Break-Point ────────────────────────────
# oha: fast visual HTTP benchmarker (lower overhead than k6 for raw RPS).
# Use this to find the pure throughput ceiling before running k6 scenarios.
#
# Usage:
#   ./apps/api/load-tests/oha/run-baseline.sh [BASE_URL]
#
# Defaults to http://localhost:3000/api
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BASE_URL="${1:-http://localhost:3000/api}"
TOKEN="${LOAD_TEST_TOKEN:?Environment variable LOAD_TEST_TOKEN must be set}"
RESULTS_DIR="apps/api/load-tests/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

mkdir -p "$RESULTS_DIR"

banner() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  $1"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

oha_run() {
  local label="$1"
  local concurrency="$2"
  local duration="$3"
  local url="$4"
  shift 4
  local extra_args=("$@")

  local outfile="${RESULTS_DIR}/oha-${label}-c${concurrency}-${TIMESTAMP}.json"

  echo ""
  echo "▶ [${label}] c=${concurrency}, z=${duration}, url=${url}"

  # oha writes JSON to -o file; stdout is empty in --no-tui + json mode
  oha \
    -c "$concurrency" \
    -z "$duration" \
    --no-tui \
    --output-format json \
    -o "$outfile" \
    "${extra_args[@]}" \
    "$url"

  # Print a compact one-liner from the saved file
  # Latency percentile keys: p50, p95, p99, "p99.9" — all in seconds, *1000 → ms
  jq '{
      rps:        (.summary.requestsPerSec | round),
      total:      .summary.total,
      success:    .summary.successRate,
      p50_ms:     (.latencyPercentiles.p50  * 1000 | round),
      p95_ms:     (.latencyPercentiles.p95  * 1000 | round),
      p99_ms:     (.latencyPercentiles.p99  * 1000 | round),
      slowest_ms: (.summary.slowest         * 1000 | round),
      errors:     .errorDistribution
    }' "$outfile"

  echo "  → saved: $outfile"
}

# ─── 1. Warm-up: health endpoint, no auth ─────────────────────────────────────
banner "WARM-UP — Health (c=50, 30s)"
oha_run "warmup-health" 50 "30s" "${BASE_URL}/health"

# ─── 2. Public reads at increasing concurrency ────────────────────────────────
banner "PUBLIC READS — Users list"
for c in 100 500 1000 2000; do
  oha_run "users-c${c}" "$c" "30s" "${BASE_URL}/users"
  sleep 5  # breathe between levels
done

# ─── 3. Authenticated reads ───────────────────────────────────────────────────
banner "AUTH READS — /auth/me"
for c in 100 500 1000; do
  oha_run "auth-me-c${c}" "$c" "30s" "${BASE_URL}/auth/me" \
    --header "Authorization: Bearer ${TOKEN}"
  sleep 5
done

# ─── 4. Raw throughput ceiling — health only, maximum concurrency ─────────────
banner "BREAK-POINT — Health endpoint (escalating concurrency)"
for c in 1000 2000 5000 10000 20000 50000; do
  echo ""
  echo "🔥 Pushing c=${c} ..."
  oha_run "breakpoint-health-c${c}" "$c" "60s" "${BASE_URL}/health" || {
    echo "⚠️  oha exited non-zero at c=${c} — likely the ceiling. Stopping."
    break
  }
  # If error rate > 20% in previous run, stop escalating
  last_file=$(ls -t "${RESULTS_DIR}"/oha-breakpoint-health-*.json 2>/dev/null | head -1)
  if [[ -n "$last_file" ]]; then
    success=$(jq '.summary.successRate // 1' "$last_file")
    if (( $(echo "$success < 0.80" | bc -l) )); then
      echo "⛔  Success rate ${success} < 80% — break-point found. Stopping escalation."
      break
    fi
  fi
  sleep 10
done

# ─── 5. Summary ───────────────────────────────────────────────────────────────
banner "SUMMARY"
echo "All oha results saved to: ${RESULTS_DIR}/"
echo ""
echo "RPS overview:"
for f in "${RESULTS_DIR}"/oha-*-"${TIMESTAMP}".json; do
  label=$(basename "$f" .json | sed "s/-${TIMESTAMP}//")
  jq -r "\"  ${label}: \(.summary.requestsPerSec | round) rps  p95=\(.latencyPercentiles.p95 * 1000 | round)ms  p99=\(.latencyPercentiles.p99 * 1000 | round)ms  ok=\(.summary.successRate)\"" "$f" 2>/dev/null || true
done

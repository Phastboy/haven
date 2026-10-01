// ─── Haven Load Test — Shared Helpers ───────────────────────────────────────
import { check } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// PRIMARY signal — fires only on hard failures: 5xx responses or network errors.
// This is the metric that means "the server is broken".
export const errorRate = new Rate('haven_error_rate');

// SECONDARY signal — fires when a check assertion doesn't match.
// Useful for debugging but doesn't indicate server health on its own.
export const checkFailRate = new Rate('haven_check_failure');

// TTFB proxy for DB query time
export const dbLatency = new Trend('haven_db_latency_ms', true);

/**
 * Standard response checker.
 * - haven_error_rate  : incremented only on 5xx or network errors (status 0)
 * - haven_check_failure: incremented on any check mismatch
 * - dbLatency         : always recorded
 *
 * @param {object} res            - k6 http response
 * @param {string} label          - human label for the check group
 * @param {number} [expectedStatus=200]
 */
export function checkResponse(res, label, expectedStatus = 200) {
  const checksOk = check(res, {
    [`${label}: status ${expectedStatus}`]: (r) => r.status === expectedStatus,
    [`${label}: not empty body`]:           (r) => r.body && r.body.length > 0,
    [`${label}: latency < 2s`]:             (r) => r.timings.duration < 2000,
  });

  // Hard failure: 5xx or no response (network error returns status 0)
  const isHardFailure = res.status === 0 || res.status >= 500;
  errorRate.add(isHardFailure);

  // Soft failure: any check mismatch
  checkFailRate.add(!checksOk);

  dbLatency.add(res.timings.waiting);

  return checksOk;
}

/**
 * Log a concise one-liner per request for debugging at low VU counts.
 */
export function logRequest(res, label) {
  if (__ENV.VERBOSE === '1') {
    console.log(`[${label}] ${res.status} ${res.timings.duration.toFixed(1)}ms`);
  }
}


// ─── Scenario 04: Soak / Endurance Test ──────────────────────────────────────
// Purpose : Run at moderate load for an extended period to detect memory leaks
//           in the event bus, session cache, Postgres connection pool, or any
//           growing heap in the Bun/Elysia runtime.
//
//           Run memory-watch.sh BEFORE starting this scenario. The watcher logs
//           RSS/VmRSS of the Bun process every 10s to results/memory-<ts>.csv.
//
// Load    : 30 VUs × 10 minutes (steady, no ramp drama).
// Watch for:
//   - Monotonically growing memory in memory-watch.sh output
//   - Latency drift — p95 creeping up over time
//   - Any connection reset / ECONNRESET errors mid-test
// Pass    : Error rate < 1%, p95 < 300ms throughout, zero memory leak signal.
// Run     : k6 run apps/api/load-tests/k6/scenarios/04-soak.js
// ─────────────────────────────────────────────────────────────────────────────
import http from 'k6/http';
import { sleep, group } from 'k6';
import { Trend } from 'k6/metrics';
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from '../lib/config.js';
import { checkResponse } from '../lib/helpers.js';

// Time-bucketed latency trend — lets us see if latency drifts across the run
const latencyBucket = new Trend('soak_latency_ms', true);

export const options = {
  vus: 30,
  duration: '10m',
  thresholds: {
    http_req_failed:   ['rate<0.01'],   // strict: <1% errors over 10 min
    http_req_duration: [
      'p(95)<300',
      'p(99)<800',
    ],
    haven_error_rate:  ['rate<0.01'],
    soak_latency_ms:   ['p(95)<300'],   // same threshold, separate trend for analysis
  },
};

export default function () {
  const pick = Math.random();

  if (pick < 0.35) {
    group('Health + public reads', () => {
      const r1 = http.get(`${BASE_URL}/health`);
      checkResponse(r1, 'health', 200);
      latencyBucket.add(r1.timings.duration);

      const r2 = http.get(`${BASE_URL}/users`);
      checkResponse(r2, 'users', 200);
      latencyBucket.add(r2.timings.duration);
    });

  } else if (pick < 0.65) {
    group('Offer reads', () => {
      const res = http.get(`${BASE_URL}/offers/user/${TEST_USER_ID}`);
      checkResponse(res, 'offers', 200);
      latencyBucket.add(res.timings.duration);
    });

  } else if (pick < 0.85) {
    group('Auth session read', () => {
      const res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS });
      checkResponse(res, 'auth-me', 200);
      latencyBucket.add(res.timings.duration);
    });

  } else {
    group('Orders read', () => {
      const res = http.get(`${BASE_URL}/orders/me`, { headers: AUTH_HEADERS });
      checkResponse(res, 'orders-me', 200);
      latencyBucket.add(res.timings.duration);
    });
  }

  sleep(Math.random() * 0.5 + 0.2); // 200–700ms — realistic pacing
}

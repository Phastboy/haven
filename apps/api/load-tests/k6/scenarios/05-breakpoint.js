// ─── Scenario 05: Break-Point Test ───────────────────────────────────────────
// Purpose : Find the actual ceiling — the VU/RPS count where the API starts
//           returning errors, latency explodes, or the process OOMs.
//
//           Strategy: step-ramp in large increments. Each step is 90s so the
//           system has time to stabilise. We do NOT abort on threshold failure
//           so we can observe all stages and find the exact inflection point.
//
//           k6 will report per-stage latency in the summary. Look for the
//           stage where p99 crosses 2s or error rate crosses 5%.
//
// Max VUs : 10,000 (k6 memory ~ 5KB/VU = ~50MB overhead on top of API).
//           If your machine has <8GB free RAM, reduce MAX_VUS env var.
//           For 100k target use oha/run-breakpoint.sh (lower k6 overhead).
//
// Run     : k6 run apps/api/load-tests/k6/scenarios/05-breakpoint.js
//           MAX_VUS=20000 k6 run apps/api/load-tests/k6/scenarios/05-breakpoint.js
// ─────────────────────────────────────────────────────────────────────────────
import http from 'k6/http';
import { sleep } from 'k6';
import { Counter } from 'k6/metrics';
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from '../lib/config.js';
import { checkResponse } from '../lib/helpers.js';

const maxVUs = parseInt(__ENV.MAX_VUS || '10000', 10);

const errorCounter = new Counter('haven_errors_total');

export const options = {
  // Step-ramp: each stage holds long enough to see steady-state behaviour
  stages: [
    { duration: '30s',  target: 100    },  // Step 1 — warm up
    { duration: '90s',  target: 100    },  // hold
    { duration: '20s',  target: 500    },  // Step 2
    { duration: '90s',  target: 500    },  // hold
    { duration: '20s',  target: 1000   },  // Step 3
    { duration: '90s',  target: 1000   },  // hold
    { duration: '20s',  target: 2000   },  // Step 4
    { duration: '90s',  target: 2000   },  // hold
    { duration: '20s',  target: 5000   },  // Step 5 — getting spicy
    { duration: '90s',  target: 5000   },  // hold
    { duration: '20s',  target: maxVUs },  // Step 6 — find the wall
    { duration: '90s',  target: maxVUs },  // hold at ceiling
    { duration: '30s',  target: 0      },  // ramp down
  ],
  // Do NOT abort on failure — we want data from all stages
  thresholds: {
    http_req_failed:   [{ threshold: 'rate<0.05', abortOnFail: false }],
    http_req_duration: [{ threshold: 'p(99)<5000', abortOnFail: false }],
  },
};

export default function () {
  // At break-point we focus purely on the cheapest representative requests
  // so latency reflects server capacity, not payload complexity.
  const pick = Math.random();

  let res;
  let label;

  if (pick < 0.50) {
    // Health — pure Bun/Elysia overhead, no DB
    res = http.get(`${BASE_URL}/health`);
    label = 'health';

  } else if (pick < 0.80) {
    // DB read — cheapest SQL (single PK lookup)
    res = http.get(`${BASE_URL}/users`);
    label = 'users-list';

  } else {
    // Auth token validation — involves JWT decode + session DB lookup
    res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS });
    label = 'auth-me';
  }

  const ok = checkResponse(res, label, res.status < 500 ? res.status : 200);
  if (!ok || res.status >= 400) {
    errorCounter.add(1);
  }

  // Minimal think time — we want maximum pressure
  sleep(0.01);
}

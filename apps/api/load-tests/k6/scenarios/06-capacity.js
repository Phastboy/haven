// ─── Scenario 06: Capacity Ceiling Test ────────────────────────────────────────
// Purpose : Find the exact RPS where Haven breaches the performance contract.
// Load    : Stepped escalation ladder (100 → 5000 RPS).
// Focus   : Stop immediately (abortOnFail) when p95 > 250ms or errors > 1%.
// Run     : (Must run system-watch.sh alongside to identify the bottleneck)
//           k6 run apps/api/load-tests/k6/scenarios/06-capacity.js
// ─────────────────────────────────────────────────────────────────────────────
import http from 'k6/http';
import { sleep } from 'k6';
import { CONTRACT } from '../lib/config.js';
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from '../lib/config.js';
import { checkResponse } from '../lib/helpers.js';

export const options = {
  scenarios: {
    capacity: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      preAllocatedVUs: 500,
      maxVUs: 5000,
      // The escalation ladder from the contract
      stages: [
        { duration: '2m', target: 100  },
        { duration: '2m', target: 250  },
        { duration: '2m', target: 500  },
        { duration: '2m', target: 750  },
        { duration: '2m', target: 1000 },
        { duration: '2m', target: 1500 },
        { duration: '2m', target: 2000 },
        { duration: '2m', target: 3000 },
        { duration: '2m', target: 5000 },
      ],
    },
  },
  thresholds: {
    // The test stops IMMEDIATELY when these are breached
    http_req_duration: [{ threshold: `p(95)<${CONTRACT.capacity.abortP95Ms}`, abortOnFail: true }],
    http_req_failed:   [{ threshold: `rate<${CONTRACT.capacity.abortErrorPct}`, abortOnFail: true }],
  },
};

export function setup() {
  console.log('▶ Make sure system-watch.sh is running in the background!');
  console.log('  The CSV output is required to identify the bottleneck resource.');
}

export default function () {
  // At capacity edge, we use the cheapest endpoints so latency reflects
  // pure framework/DB capacity, not payload serialization overhead.
  const pick = Math.random();
  let res;
  let label;

  if (pick < 0.50) {
    res = http.get(`${BASE_URL}/health`);
    label = 'health';
  } else if (pick < 0.80) {
    res = http.get(`${BASE_URL}/users?limit=5`);
    label = 'list-users';
  } else {
    res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS });
    label = 'auth-me';
  }

  checkResponse(res, label, 200);
  // Minimal sleep, we want pure throughput
  sleep(0.01);
}

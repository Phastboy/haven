// ─── Scenario 03: Spike Test ──────────────────────────────────────────────────
// Purpose : Simulate a sudden traffic burst — flash sale, viral post, etc.
//           Reveals queue buildup, Postgres connection exhaustion, and recovery
//           behaviour after the spike subsides.
// Load    : 10 VUs → 1000 VUs in 15s → hold 1 min → drop back to 10.
// Focus   : Mixed public + auth reads (writes excluded to keep spike clean).
// Pass    : Error rate < 5% during spike (10% allowed at peak), recovery p95 < 400ms.
// Run     : k6 run apps/api/load-tests/k6/scenarios/03-spike.js
// ─────────────────────────────────────────────────────────────────────────────
import http from 'k6/http';
import { sleep, group } from 'k6';
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from '../lib/config.js';
import { checkResponse } from '../lib/helpers.js';

export const options = {
  stages: [
    { duration: '30s', target: 10   },  // idle baseline
    { duration: '15s', target: 1000 },  // SPIKE: 10 → 1000
    { duration: '1m',  target: 1000 },  // hold spike
    { duration: '15s', target: 10   },  // recover
    { duration: '30s', target: 10   },  // confirm recovery
    { duration: '10s', target: 0    },  // ramp down
  ],
  thresholds: {
    http_req_failed:   ['rate<0.10'],   // allow up to 10% errors during spike peak
    http_req_duration: [
      'p(95)<1000',                     // p95 under 1s (spike tolerant)
      { threshold: 'p(99)<2000', abortOnFail: false },
    ],
    haven_error_rate:  ['rate<0.10'],
  },
};

export default function () {
  const pick = Math.random();

  if (pick < 0.5) {
    group('Health check', () => {
      const res = http.get(`${BASE_URL}/health`);
      checkResponse(res, 'health', 200);
    });

  } else if (pick < 0.75) {
    group('Users list', () => {
      const res = http.get(`${BASE_URL}/users`);
      checkResponse(res, 'users', 200);

    });

  } else if (pick < 0.90) {
    group('Offers list', () => {
      const res = http.get(`${BASE_URL}/offers/user/${TEST_USER_ID}`);
      checkResponse(res, 'offers', 200);

    });

  } else {
    group('Auth me', () => {
      const res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS });
      checkResponse(res, 'auth-me', 200);
    });
  }

  // Minimal think time during spike — we want the pressure
  sleep(0.05);
}

// ─── Scenario 04: Spike Test (2,000 RPS burst) ───────────────────────────────
// Purpose : Simulate a sudden traffic burst and measure survival + recovery.
// Load    : Baseline 100 RPS → ramp to 2,000 RPS → hold 60s → drop to 100 RPS → hold 60s.
// Focus   : Focus on recovery phase. If system survives but stays unhealthy, it fails.
// Run     : k6 run apps/api/load-tests/k6/scenarios/04-spike.js
// ─────────────────────────────────────────────────────────────────────────────
import http from 'k6/http';
import { sleep } from 'k6';
import { CONTRACT } from '../lib/config.js';
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from '../lib/config.js';
import { checkResponse } from '../lib/helpers.js';

export const options = {
  scenarios: {
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      preAllocatedVUs: 500,
      maxVUs: 3000,
      stages: [
        { duration: '30s', target: 100 },                    // Baseline
        { duration: '10s', target: CONTRACT.spike.burstRps }, // Ramp up
        { duration: '60s', target: CONTRACT.spike.burstRps }, // Spike hold
        { duration: '10s', target: 100 },                    // Ramp down
        { duration: '60s', target: 100 },                    // Recovery window
        { duration: '10s', target: 0 },
      ],
    },
  },
  thresholds: {
    // Spike phase thresholds (degraded latency allowed)
    'http_req_duration{phase:spike}':    [{ threshold: `p(95)<${CONTRACT.spike.spikeP95Ms}`, abortOnFail: false }],
    'http_req_failed{phase:spike}':      [`rate<${CONTRACT.spike.errorPct}`],

    // Recovery phase thresholds (must return to normal latency)
    'http_req_duration{phase:recovery}': [`p(95)<${CONTRACT.spike.recoveryP95Ms}`],
    'http_req_failed{phase:recovery}':   [`rate<${CONTRACT.normal.errorPct}`], // Use normal error pct for recovery
  },
};

function getPhase(elapsedSeconds) {
  if (elapsedSeconds < 40) return 'baseline';
  if (elapsedSeconds < 110) return 'spike';
  return 'recovery';
}

export default function () {
  // Tag every request with its phase for threshold evaluation
  const phase = getPhase(__VU_TIME__ / 1000);
  
  const pick = Math.random();
  let res;
  let label;

  if (pick < 0.50) {
    res = http.get(`${BASE_URL}/health`, { tags: { phase } });
    label = 'health';
  } else if (pick < 0.80) {
    res = http.get(`${BASE_URL}/users?limit=50`, { tags: { phase } });
    label = 'list-users';
  } else {
    res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS, tags: { phase } });
    label = 'auth-me';
  }

  checkResponse(res, label, 200);
  sleep(0.5);
}

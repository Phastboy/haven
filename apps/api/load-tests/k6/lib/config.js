// ─── Haven Load Test — Shared Config ────────────────────────────────────────
// Override any value via env vars:
//   BASE_URL=http://localhost:3000/api k6 run scenarios/01-smoke.js
//   BEARER_TOKEN=<token>              k6 run scenarios/02-steady-state.js

export const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000/api';

// Long-lived session token (30-day TTL). Rotate via BEARER_TOKEN env var.
export const BEARER_TOKEN =
  __ENV.BEARER_TOKEN ||
  'f6263838172babc84ab986cdd6ed496dee859ff2cfa79104794c9889cfba1540d49873817a94303123df5d937c3accb0da8c3082f1d7d119d2a16ab77ee3d183';

export const AUTH_HEADERS = {
  Authorization: `Bearer ${BEARER_TOKEN}`,
  'Content-Type': 'application/json',
};

export const JSON_HEADERS = {
  'Content-Type': 'application/json',
};

// Known test identity (profile ID of the authenticated user)
export const TEST_USER_ID  = __ENV.TEST_USER_ID  || '9c630b3d-9641-4a0d-b76f-4e5bdda5ed85';
// A real offer ID for single-offer GET tests
export const TEST_OFFER_ID = __ENV.TEST_OFFER_ID || '9af24220-9472-4b5d-8889-6a8e914b167f';

// Results dir (relative to where k6 is invoked from)
export const RESULTS_DIR = 'apps/api/load-tests/results';

// ─── PERFORMANCE CONTRACT TARGETS ─────────────────────────────────────────────
// Source of truth: apps/api/load-tests/PERFORMANCE_CONTRACT.md
export const CONTRACT = {
  normal:   { rps: 500,  p95: 50,   p99: 100,  errorPct: 0.001 },
  highLoad: { rps: 1000, p95: 100,  p99: 250,  errorPct: 0.001 },
  spike:    { burstRps: 2000, spikeP95Ms: 1000, recoveryP95Ms: 100, errorPct: 0.01 },
  soak:     { rps: 500,  p95: 100,  p99: 250,  errorPct: 0.001, durationMin: 30 },
  capacity: { abortP95Ms: 250, abortErrorPct: 0.01 },
};

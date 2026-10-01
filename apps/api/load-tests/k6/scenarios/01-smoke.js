// ─── Scenario 01: Smoke Test ─────────────────────────────────────────────────
// Purpose : Confirm the API is alive and correctly wired before real load.
// Load    : 5 VUs × 30s — intentionally light.
// Targets : Health, public reads (users, offers).
// Pass    : 100% success, p95 < 300ms.
// Run     : k6 run apps/api/load-tests/k6/scenarios/01-smoke.js
// ─────────────────────────────────────────────────────────────────────────────
import http from "k6/http";
import { sleep, group } from "k6";
import { BASE_URL, AUTH_HEADERS, TEST_USER_ID } from "../lib/config.js";
import { checkResponse } from "../lib/helpers.js";

export const options = {
  vus: 5,
  duration: "30s",
  thresholds: {
    http_req_failed: ["rate<0.01"], // <1% errors
    http_req_duration: ["p(95)<300"], // p95 under 300ms
    haven_error_rate: ["rate<0.01"],
  },
};

export default function () {
  group("Health", () => {
    const res = http.get(`${BASE_URL}/health`);
    checkResponse(res, "health", 200);
  });

  group("Public — Users", () => {
    const res = http.get(`${BASE_URL}/users`);
    checkResponse(res, "list-users", 200);
  });

  group("Public — Offers by user", () => {
    const res = http.get(`${BASE_URL}/offers/user/${TEST_USER_ID}`);
    checkResponse(res, "offers-by-user", 200);
  });

  group("Authenticated — /auth/me", () => {
    const res = http.get(`${BASE_URL}/auth/me`, { headers: AUTH_HEADERS });
    checkResponse(res, "auth-me", 200);
  });

  sleep(0.5);
}

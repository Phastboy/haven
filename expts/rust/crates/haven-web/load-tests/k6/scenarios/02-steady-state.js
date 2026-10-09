import http from "k6/http";
import { check } from "k6";
import exec from "k6/execution";
import { getSession, extractIdempotencyKey, getHeaders } from "../lib/lib.js";

// Workload mix:
//   50% -> GET /offers                         = 1 request
//   35% -> GET /offers/:id                     = 1 request
//   12% -> GET /                              = 1 request
//    3% -> write journey                       = 9 requests
//
// Expected average:
//   (0.50 * 1) + (0.35 * 1) + (0.12 * 1) + (0.03 * 9)
//   = 1.24 HTTP requests / iteration
//
// Therefore:
//   ~806 iterations/s ≈ 1000 HTTP requests/s
//
// RATE is supplied by standard-run.sh:
//
//   --rate 50   -> 50 iterations/s
//   --rate 200  -> 200 iterations/s
//   --rate 400  -> 400 iterations/s
//   --rate 600  -> 600 iterations/s
//   --rate 806  -> ~1000 HTTP requests/s
//   --rate 826  -> ~1024 HTTP requests/s
//
// This makes the scenario suitable for capacity testing.

const RATE = Number(__ENV.RATE || 50);

if (!Number.isFinite(RATE) || RATE <= 0) {
  throw new Error(`RATE must be a positive number, got: ${__ENV.RATE}`);
}

const RAMP_DURATION_MS = 30 * 1000;

export const options = {
  scenarios: {
    steady_state: {
      executor: "ramping-arrival-rate",
      startRate: 0,
      timeUnit: "1s",

      // Keep enough VUs available for the arrival rate without
      // unnecessarily constraining the load generator.
      preAllocatedVUs: 1000,
      maxVUs: 5000,

      stages: [
        // Ramp to the requested capacity point.
        { duration: "30s", target: RATE },

        // Hold the requested rate long enough to measure stability.
        { duration: "5m", target: RATE },
      ],
    },
  },

  thresholds: {
    // Capacity/SLO contract:
    // - No failed HTTP requests
    // - No dropped iterations
    // - 100% correctness
    // - p95 <= 200ms
    // - p99 <= 500ms
    http_req_failed: ["rate==0.0"],
    dropped_iterations: ["count==0"],
    checks: ["rate==1.0"],

    // Do NOT put a fixed http_reqs rate threshold here.
    //
    // At RATE=100, for example, the expected workload is only
    // ~124 HTTP requests/s. A fixed `rate>=950` threshold would
    // incorrectly fail that capacity point.

    // Detailed latency gates by route tag for the 5-minute hold stage.
    "http_req_duration{stage:hold,name:GET /offers}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:GET /offers/id}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:GET /}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:GET /offers/new}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:POST /offers/new}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:GET /offers/id/edit}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:POST /offers/id/edit}": ["p(95)<=200", "p(99)<=500"],
    "http_req_duration{stage:hold,name:POST /offers/id/delete}": ["p(95)<=200", "p(99)<=500"],
  },
};

const BASE_URL = __ENV.BASE_URL || "http://127.0.0.1:3000";

export default function () {
  exec.vu.tags["stage"] =
    exec.instance.currentTestRunDuration >= RAMP_DURATION_MS ? "hold" : "ramp";

  const session = getSession(__VU, __ITER);
  const headers = getHeaders(session.session_token);

  const rand = Math.random();

  if (rand < 0.5) {
    // 50%: GET /offers (list)
    const res = http.get(`${BASE_URL}/offers`, {
      headers,
      redirects: 0,
      tags: { name: "GET /offers" },
    });

    check(res, {
      "list is 200": (r) => r.status === 200,
    });
  } else if (rand < 0.85) {
    // 35%: GET /offers/{id} (detail)
    // Pick a random offer from the seeded data for this user.
    if (session.offer_ids && session.offer_ids.length > 0) {
      const offerId = session.offer_ids[Math.floor(Math.random() * session.offer_ids.length)];

      const res = http.get(`${BASE_URL}/offers/manage/${offerId}`, {
        headers,
        redirects: 0,
        tags: { name: "GET /offers/id" },
      });

      check(res, {
        "detail is 200": (r) => r.status === 200,
      });
    }
  } else if (rand < 0.97) {
    // 12%: GET / (public landing)
    // Ensure this is unauthenticated so it doesn't redirect.
    const res = http.get(`${BASE_URL}/`, {
      tags: { name: "GET /" },
    });

    check(res, {
      "public is 200": (r) => r.status === 200,
    });
  } else {
    // 3%: write journey (9 requests)
    //
    // 1. GET  /offers/manage/new
    // 2. POST /offers/manage/new
    // 3. GET  /offers/manage/{id}
    // 4. GET  /offers/manage/{id}/edit
    // 5. POST /offers/manage/{id}/edit
    // 6. GET  /offers/manage/{id}
    // 7. POST /offers/manage/{id}/delete
    // 8. GET  /offers/manage/{id}       -> 404
    // 9. GET  /offers

    let res = http.get(`${BASE_URL}/offers/manage/new`, {
      headers,
      tags: { name: "GET /offers/new" },
    });

    check(res, {
      "new form is 200": (r) => r.status === 200,
    });

    const idempKey = extractIdempotencyKey(res.body);

    check(idempKey, {
      "extracted idempotency key": (k) => !!k,
    });

    if (!idempKey) return;

    const createPayload = {
      title: `Write Journey ${__VU}-${__ITER}`,
      description: "Load test dynamic offer",
      price: "500",
      currency: "NGN",
      idempotency_key: idempKey,
    };

    res = http.post(`${BASE_URL}/offers/manage/new`, createPayload, {
      headers: Object.assign({}, headers, {
        "Content-Type": "application/x-www-form-urlencoded",
      }),
      redirects: 0,
      tags: { name: "POST /offers/new" },
    });

    check(res, {
      "create redirects": (r) => r.status === 303,
    });

    const location = res.headers["Location"];

    check(location, {
      "location header present": (l) => !!l,
    });

    if (!location) return;

    // View detail (assert title and price).
    res = http.get(`${BASE_URL}${location}`, {
      headers,
      tags: { name: "GET /offers/id" },
    });

    check(res, {
      "detail shows created offer": (r) => r.body.includes(createPayload.title),
    });

    // Edit form.
    res = http.get(`${BASE_URL}${location}/edit`, {
      headers,
      tags: { name: "GET /offers/id/edit" },
    });

    check(res, {
      "edit form is 200": (r) => r.status === 200,
    });

    // POST edit.
    const editPayload = {
      title: createPayload.title + " (Edited)",
      description: createPayload.description,
      price: "0",
      currency: "NGN",
    };

    res = http.post(`${BASE_URL}${location}/edit`, editPayload, {
      headers: Object.assign({}, headers, {
        "Content-Type": "application/x-www-form-urlencoded",
      }),
      redirects: 0,
      tags: { name: "POST /offers/id/edit" },
    });

    check(res, {
      "edit redirects": (r) => r.status === 303,
    });

    // View detail again.
    res = http.get(`${BASE_URL}${location}`, {
      headers,
      tags: { name: "GET /offers/id" },
    });

    check(res, {
      "detail shows edited title": (r) => r.body.includes(editPayload.title),
      "detail shows Free": (r) => r.body.includes("Free"),
    });

    // POST delete.
    res = http.post(
      `${BASE_URL}${location}/delete`,
      {},
      {
        headers: Object.assign({}, headers, {
          "Content-Type": "application/x-www-form-urlencoded",
        }),
        redirects: 0,
        tags: { name: "POST /offers/id/delete" },
      },
    );

    check(res, {
      "delete redirects": (r) => r.status === 303,
    });

    // Verify 404.
    res = http.get(`${BASE_URL}${location}`, {
      headers,
      tags: { name: "GET /offers/id" },
      responseCallback: http.expectedStatuses(404),
    });

    check(res, {
      "404 after delete": (r) => r.status === 404,
    });

    // GET list at the end.
    res = http.get(`${BASE_URL}/offers`, {
      headers,
      tags: { name: "GET /offers" },
    });

    check(res, {
      "list after delete is 200": (r) => r.status === 200,
    });
  }
}

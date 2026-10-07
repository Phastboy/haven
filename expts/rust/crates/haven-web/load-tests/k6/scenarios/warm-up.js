import http from "k6/http";
import { check } from "k6";
import { users, getSession, getHeaders } from "../lib/lib.js";

// The warm-up script runs in the background while the orchestrator monitors the guest's
// cache hit ratio and disk I/O. It needs to run until the orchestrator kills it.
export const options = {
  scenarios: {
    warm_up: {
      executor: "constant-arrival-rate",
      rate: __ENV.RATE ? parseInt(__ENV.RATE) : 20,
      timeUnit: "1s",
      duration: __ENV.DURATION || "600s",
      preAllocatedVUs: 10,
      maxVUs: 100,
    },
  },
};

const BASE_URL = __ENV.BASE_URL || __ENV.TARGET || "https://10.10.0.2";

export default function () {
  const user = getSession(__VU, __ITER);
  const headers = getHeaders(user.session_token);

  // Read-only endpoints to warm up caches without mutating the DB
  const res = http.batch([
    { method: "GET", url: `${BASE_URL}/auth/sign-in`, params: { headers } },
    { method: "GET", url: `${BASE_URL}/offers`, params: { headers } },
  ]);

  // We don't care about thresholds for warm-up, but we do care that they aren't failing hard
  check(res[0], { "me is 200": (r) => r.status === 200 });
  check(res[1], { "offers is 200": (r) => r.status === 200 });
}

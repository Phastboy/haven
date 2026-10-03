# Haven v0.1.x — Verification Plan

This document records the mandatory verification steps required before any version of Haven is considered "done". This prevents regressions and guarantees our security and concurrency guardrails.

## 1. Topcoat Framework Assertions
These are the foundational assertions we rely on from the Topcoat framework. We must verify they hold true in practice:
- **CSRF / OriginPolicy**: Topcoat rejects state-changing cross-origin browser requests with `403 Forbidden` by default. (Cited from `topcoat/router` documentation).
- **Session Cookie Security**: The session cookie is set with `__Host-` prefix, `Secure`, `HttpOnly`, `SameSite=Lax`, and `Path=/`. (Cited from `topcoat/session` documentation).
- **XSS Escaping**: Topcoat's `view!` macro automatically escapes all dynamic content unless explicitly opted out via `_unescaped`. (Cited from `topcoat/view` documentation).

## 2. Correctness & Concurrency Tests
The following automated/manual tests must pass:
1. **Ownership**: User B receives a `404 Not Found` when attempting to access, edit, or delete User A's `/offers/{id}`.
2. **XSS Payload**: Create an offer titled `<script>alert(1)</script>` and assert the rendered HTML contains the escaped form (e.g. `&lt;script&gt;`).
3. **Idempotency (Concurrency)**: 20 concurrent creates with the same idempotency key must result in exactly *one* offer in the database.
4. **Rate Limiter (Token Bucket)**: A burst of distinct creates from a single session produces `429 Too Many Requests` when the burst limit (e.g., 5) is exceeded.
5. **Atomic Verification**: Concurrent verifies of the same magic link produce exactly *one* success. Subsequent attempts fail or redirect to an invalid state.
6. **No Enumeration**: A rate-limited sign-in request must be indistinguishable from a normal sign-in request (identical redirect to `/auth/sent`).
7. **CSRF**: A state-changing `POST` request with a foreign `Origin` header must return `403 Forbidden`.

## 3. Load Testing (Performance Contract)
Using the open-model (arrival-rate) `k6` executor defined in `PERFORMANCE_CONTRACT.md`:
- **Target**: 1000 RPS for 5 minutes.
- **Gate**: `p99 < 500ms`.
- **Metrics to observe**: DB pool wait time, memory (RSS) drift over the 5 minutes, and latency drift.
- **Stress Step**: Push beyond 1000 RPS to discover the actual breaking point (capacity ceiling) and record it.

# Haven API — Performance Contract

> **This document is the source of truth for Haven's performance requirements.**
>
> Targets are established based on what Haven _should_ be capable of — not what it currently achieves.
> A failing test means **investigate**, not adjust the target.
> The only permitted direction for any target is stricter.

---

## Philosophy

Load testing answers two different questions. It is important not to conflate them:

1. **"What can Haven do right now?"** — this is measurement.
2. **"What must Haven be capable of?"** — this is the contract.

The contract is defined first. Measurement tells us how far we are from it.

We do not define targets by looking at current performance and setting numbers we can already hit.
We define targets by asking what a serious production API in this domain should be capable of,
and then we let the engineering close the gap.

---

## The Ten Dimensions

Every load test run is evaluated against ten dimensions:

| # | Dimension | What it asks |
|---|-----------|-------------|
| 1 | **Latency p50** | How fast is the typical request? |
| 2 | **Latency p95** | How fast are almost all requests? |
| 3 | **Latency p99** | How bad is the slow tail? |
| 4 | **Error rate** | How often does the system fail outright? |
| 5 | **Correctness** | Did successful-looking operations actually work? |
| 6 | **Throughput** | How much work can Haven sustain? |
| 7 | **CPU utilisation** | Are we exhausting compute? |
| 8 | **Memory utilisation** | Are we exhausting memory? Is RSS bounded? |
| 9 | **Resource saturation** | Which resource becomes the bottleneck first? |
| 10 | **Maximum sustainable load** | Where does the contract break? |

Latency averages are **not used as gates**. Averages hide bad tail behaviour.
p95 and p99 are the gates.

Correctness is not implied by HTTP 200. A `POST /offers` that returns 201 in 5ms
but fails to persist the row is not a correct operation. Correctness checks
verify the round-trip: create → retrieve → confirm match.

---

## The Contract

### 1. Normal — 500 RPS sustained

This is the most important tier. The API must maintain these characteristics
under continuous production-representative load.

| Metric | Target |
|--------|--------|
| Sustained RPS | ≥ 500 |
| p50 | < 10 ms |
| p95 | < 50 ms |
| p99 | < 100 ms |
| Error rate | < 0.1% |
| Correctness (write round-trip) | 100% |

### 2. High Load — 1,000 RPS sustained

The API must be able to sustain double normal load within acceptable latency degradation.

| Metric | Target |
|--------|--------|
| Sustained RPS | ≥ 1,000 |
| p95 | < 100 ms |
| p99 | < 250 ms |
| Error rate | < 0.1% |

### 3. Spike — 100 → 2,000 → 100 RPS

A sudden burst followed by a full recovery. Two separate criteria:

**During spike (60 second hold at 2,000 RPS):**

| Metric | Target |
|--------|--------|
| p95 | < 1,000 ms |
| Error rate | < 1% |
| Crash | Not permitted |
| Persistent error state | Not permitted |

**Recovery (60 second window after spike subsides):**

| Metric | Target |
|--------|--------|
| p95 | < 100 ms |
| Error rate | < 0.1% |

A server that survives a spike but remains unhealthy afterward has not handled the spike.
The recovery criterion is a hard gate.

### 4. Soak — 500 RPS × 30 minutes

Sustained load over time, designed to surface memory leaks, connection pool exhaustion,
event loop drift, and any resource that accumulates without bound.

| Metric | Target |
|--------|--------|
| Sustained RPS | ≥ 500 |
| p95 | < 100 ms |
| p99 | < 250 ms |
| Error rate | < 0.1% |
| RSS growth | ≈ 0 (no unbounded growth) |
| Latency drift | None — p95 must hold for the full 30 minutes |

Memory target is not a fixed ceiling — it is a growth rate.
A system that uses 300 MB and holds it is fine.
A system that grows from 200 MB to 700 MB over 30 minutes is not.

### 5. Capacity — find the ceiling

Push Haven until the contract breaks. No arbitrary upper limit.

**Escalation ladder (RPS):**
```
100 → 250 → 500 → 750 → 1,000 → 1,500 → 2,000 → 3,000 → 5,000 → ...
```
Each step held for 2 minutes to allow stabilisation before escalating.

**Stop condition (first breach terminates the test):**

| Trigger | Threshold |
|---------|-----------|
| p95 latency | > 250 ms |
| Error rate | > 1% |
| Resource exhaustion | Any |

The RPS at which the stop condition is triggered is Haven's **current capacity ceiling**.
This number is recorded in the scorecard after every capacity run.

**Observe alongside:**
- CPU utilisation
- RSS
- Active DB connections
- Open file descriptors

The resource that saturates first at the ceiling is the **bottleneck**. That is the
target for the next round of optimisation.

---

## The Scorecard

After every test run, update the scorecard below with actual measurements.
Commit the updated scorecard. This creates a git-tracked history of Haven's
performance over time, tied to the code that produced it.

Format: `date · git sha · pass/fail per tier`

---

### Historical JS Baseline

**Date:** 2026-10-03
**Branch:** `perf/load-test-optimizations`
**Commit:** `d4b537a`
**Executor model:** `ramping-arrival-rate` (Open Model)

> ⚠️ The following measurements were taken against the old Node.js implementation with the old `ramping-vus` executor
> at 50 VUs over 4 minutes. They do not directly represent RPS-contract compliance for the Rust implementation.
> They are the historical baseline from which the contract suite was built.
> The upcoming Rust tests MUST use the open model (`ramping-arrival-rate` or `constant-arrival-rate`) to prevent coordinated omission.

| Metric | Measured |
|--------|----------|
| Achieved RPS | ~123 req/s @ 50 VUs |
| p50 | 3.9 ms |
| p95 | 14.8 ms |
| p99 | 30.4 ms |
| Error rate | 0% |
| Correctness | 100% |
| DB TTFB p95 | 14.6 ms |
| data_received | 101 MB / 4 min |

**Previous baseline (before performance fixes, same branch):**

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| p95 | 188 ms | 14.8 ms | 12.7× |
| p99 | 347 ms | 30 ms | 11.6× |
| data_received / 4 min | 1.9 GB | 101 MB | 18.8× |

**Fixes applied (historical):**
- DB indexes added: `offer(user_id)` (+ composite + `session(expires_at)`)
- Connection pool: 10 → 20 (configurable via `DB_POOL_MAX`)
- Pagination enforced at DB level: `LIMIT`/`OFFSET` on all list endpoints, max 100 rows

---

### Contract Compliance (arrival-rate suite — not yet run)

| Tier | Scenario | Status | Notes |
|------|----------|--------|-------|
| Normal (500 RPS) | `02-normal.js` | ⏳ Not yet run | |
| High Load (1,000 RPS) | `03-high-load.js` | ⏳ Not yet run | |
| Spike (2,000 RPS burst) | `04-spike.js` | ⏳ Not yet run | |
| Soak (500 RPS × 30 min) | `05-soak.js` | ⏳ Not yet run | |
| Capacity (find ceiling) | `06-capacity.js` | ⏳ Not yet run | |

---

## Running the Suite

### Prerequisites

```bash
# k6
which k6        # should resolve — installed at /usr/local/bin/k6

# oha (for raw RPS ceiling tests)
which oha       # should resolve — installed at ~/.cargo/bin/oha

# Postgres client (for system-watch.sh DB metrics)
which psql
```

### Individual scenarios

```bash
# Smoke — always run first to confirm the API is up
k6 run crates/haven-web/load-tests/k6/scenarios/01-smoke.js

# Normal contract (500 RPS, 5 min)
k6 run crates/haven-web/load-tests/k6/scenarios/02-normal.js

# High-load contract (1,000 RPS, 5 min)
k6 run crates/haven-web/load-tests/k6/scenarios/03-high-load.js

# Spike + recovery
k6 run crates/haven-web/load-tests/k6/scenarios/04-spike.js

# Soak (500 RPS × 30 min) — start system monitor first
./crates/haven-web/load-tests/monitor/system-watch.sh &
k6 run crates/haven-web/load-tests/k6/scenarios/05-soak.js

# Capacity — find the ceiling
./crates/haven-web/load-tests/monitor/system-watch.sh &
k6 run crates/haven-web/load-tests/k6/scenarios/06-capacity.js
```

### Full suite

```bash
./crates/haven-web/load-tests/run-all.sh
```

Results are written to `crates/haven-web/load-tests/results/` (git-ignored).

### Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `BASE_URL` | `http://localhost:8080` | Application base URL |
| `TOKEN` | (see `config.js`) | Auth token for authenticated scenarios |
| `TEST_USER_ID` | (see `config.js`) | User ID for offer listing tests |
| `TEST_OFFER_ID` | (see `config.js`) | Offer ID for single-offer GET tests |
| `DB_POOL_MAX` | `20` | Postgres connection pool size |

---

## Rules

1. **Targets move in one direction only — stricter.**
   If a target is missed, the response is investigation and optimisation, not target relaxation.

2. **The scorecard is updated after every meaningful run.**
   A run without a scorecard update did not happen as far as the contract is concerned.

3. **The contract does not change because the implementation changed.**
   The contract reflects what Haven must do. Implementation is what gets Haven there.

4. **Correctness is not optional.**
   A test that passes on latency and error rate but skips correctness checks is incomplete.

5. **Resource metrics are first-class.**
   A test that passes on latency but shows unbounded RSS growth is a failing test.

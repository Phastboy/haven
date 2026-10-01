# Haven API — Load Testing Reference

## File Layout

```
apps/api/load-tests/
├── k6/
│   ├── lib/
│   │   ├── config.js          ← BASE_URL, token, user IDs
│   │   └── helpers.js         ← shared check + custom metrics
│   └── scenarios/
│       ├── 01-smoke.js        ← 5 VUs × 30s — sanity check
│       ├── 02-steady-state.js ← 50 VUs ramp — normal ops baseline
│       ├── 03-spike.js        ← 10→1000 VU burst — resilience
│       ├── 04-soak.js         ← 30 VUs × 10min — memory leak hunt
│       └── 05-breakpoint.js   ← step-ramp to 10k+ VU — find the wall
├── oha/
│   └── run-baseline.sh        ← raw RPS ceiling (escalates to 50k concurrency)
├── monitor/
│   └── memory-watch.sh        ← /proc-based RSS/CPU poller → CSV
├── results/                   ← all output lands here (gitignored)
└── run-all.sh                 ← full suite with memory monitor
```

---

## Quick Commands

### Run individual scenarios
```bash
# From repo root
k6 run apps/api/load-tests/k6/scenarios/01-smoke.js
k6 run apps/api/load-tests/k6/scenarios/02-steady-state.js
k6 run apps/api/load-tests/k6/scenarios/03-spike.js
k6 run apps/api/load-tests/k6/scenarios/04-soak.js
k6 run apps/api/load-tests/k6/scenarios/05-breakpoint.js

# Override token or base URL
BEARER_TOKEN=<new-token> k6 run apps/api/load-tests/k6/scenarios/02-steady-state.js
BASE_URL=http://staging:3000/api k6 run ...

# Raise the break-point ceiling (default 10k)
MAX_VUS=50000 k6 run apps/api/load-tests/k6/scenarios/05-breakpoint.js
```

### Start memory monitor (run BEFORE any scenario)
```bash
# Terminal 1 — leaves running while you test in Terminal 2
./apps/api/load-tests/monitor/memory-watch.sh 5 3000
# Ctrl+C to stop; prints min/max/avg RSS on exit
```

### Run oha baseline only
```bash
./apps/api/load-tests/oha/run-baseline.sh
```

### Run everything
```bash
./apps/api/load-tests/run-all.sh

# Skip individual stages
SKIP_SOAK=1 SKIP_OHA=1 ./apps/api/load-tests/run-all.sh

# Full suite with 20k break-point ceiling
MAX_VUS=20000 ./apps/api/load-tests/run-all.sh
```

---

## What Each Test Measures

| Scenario | VUs | Duration | Key Signal |
|---|---|---|---|
| 01 Smoke | 5 | 30s | Everything is wired. p95 < 300ms |
| 02 Steady-state | 0→50→0 | ~4m | Normal operating envelope. p95 target: 200ms |
| 03 Spike | 10→1000→10 | ~3m | Queue buildup, conn pool exhaustion, recovery |
| 04 Soak | 30 | 10m | Memory leak, latency drift over time |
| 05 Break-point | 100→10k | ~15m | Hard ceiling — where errors & latency blow up |
| oha baseline | c=50…50k | 30s ea | Raw RPS capacity of the Bun HTTP layer |

---

## Reading Results

### k6 JSON summary (`results/k6-*.json`)
Key fields:
```json
{
  "metrics": {
    "http_reqs":          { "values": { "rate": 1234.5 } },   ← RPS
    "http_req_duration":  { "values": { "p(95)": 180.3 } },   ← p95 ms
    "http_req_failed":    { "values": { "rate": 0.002 } },    ← error rate
    "haven_error_rate":   { "values": { "rate": 0.002 } },
    "haven_db_latency_ms":{ "values": { "p(95)": 45.1 } }     ← DB TTFB
  }
}
```

### oha JSON (`results/oha-*.json`)
Key fields:
```json
{
  "summary": {
    "requestsPerSec": 8432.1,
    "successRate": 0.9987,
    "slowest": 2.345
  },
  "latencyPercentiles": { "50": 0.012, "95": 0.089, "99": 0.234 }
}
```

### Memory CSV (`results/memory-*.csv`)
```
timestamp_epoch,timestamp_iso,pid,rss_kb,vsz_kb,cpu_pct,threads
1727779200,2026-10-01T12:00:00+01:00,12345,148200,342000,12.3,8
```
Plot `rss_kb` over time — a clean line is healthy; monotonic growth = leak.

---

## Break-Point Signals to Watch

| Signal | Meaning |
|---|---|
| p99 > 2000ms | API struggling with backpressure |
| error rate > 5% | Connection refusals or 5xx |
| RSS growing monotonically | Memory leak |
| `ECONNRESET` / `connection refused` | Socket queue full (OS backlog) |
| Postgres `pg_stat_activity` count near max | DB connection pool exhausted |

### Live Postgres connection count
```bash
PGPASSWORD=password psql -h localhost -p 5433 -U user -d mydb \
  -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"
```

---

## Rotating the Token

The bearer token has a 30-day TTL. When it expires:

```bash
# 1. Request a new magic link
curl -X POST localhost:3000/api/auth/magic-link/request \
  -H 'Content-Type: application/json' \
  -d '{"email":"you@example.com"}'

# 2. Grab token from server console output, then verify
curl -X POST localhost:3000/api/auth/magic-link/verify \
  -H 'Content-Type: application/json' \
  -d '{"token":"<magic-token>"}'

# 3. Use the session token
BEARER_TOKEN=<new-session-token> k6 run ...
# Or update BEARER_TOKEN in apps/api/load-tests/k6/lib/config.js
```

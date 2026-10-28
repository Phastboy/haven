# Load Test Scorecard

| Date | Environment | Scenario | Target RPS | Actual RPS | p95 (ms) | p99 (ms) | Notes |
|---|---|---|---|---|---|---|---|
| 2026-10-05 | QEMU (1vCPU, 2GB RAM, pinned) | `02-steady-state` | 630 iters/s (~750 req/s) | ~750 req/s | < 200ms | < 500ms | Baseline established; 0% error rate; physical capacity ceiling for 1vCPU/2GB |
| 2026-10-12 | Host (4 CPU, native) | `02-steady-state` | 630 iters/s (~750 req/s) | 744 req/s | 3.85ms | 5.65ms | v0.3.x maker identity & profile verification; 0% error rate; 100% checks passed; p95/p99 well within 200ms/500ms contract targets |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v05` | 10 journeys/s (~40 req/s) | 40 req/s (600/600 journeys) | 28.0ms | 31.4ms | v0.5.x search/filter/pagination gate; commit `506c651`; 0% errors; 2,400/2,400 checks passed; 0 dropped. Aggregate reconstructed from per-route summaries; worst route 29.5/34.2ms |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v06` | 1 journey/s @ 16 workers (~172 req/s) | 171.3 req/s (60/60 journeys) | 17.8ms | 23.4ms | v0.6.x save/follow + collection pagination gate; commit `506c651`; 0% errors; 10,320/10,320 checks passed; 0 dropped. Isolated write outliers of 329–365ms (max single request 365.2ms); aggregate reconstructed from per-route summaries |


## v0.7 verification — in progress

Use the Go open-arrival runner. For each public/saved/inquired class, pair a
100 HTTP RPS correctness pass checking every result with a 1,000 HTTP RPS contract
pass checking status/errors on every response and semantic correctness on a
reproducible stratified 10% sample. Require all checked results to be correct;
report sample counts rather than claiming every contract-rate result was checked.
The mixed workload is 70% public / 15% saved / 15% inquired reads, an assumption
favoring discovery until production evidence exists; split each personal class
equally between sparse and dense histories. Separate class gates still apply.

Record both passes with code/fixture SHA, duration, requested/achieved HTTP RPS,
latency, errors, semantic checked/failed counts, sample seed/rate, dropped arrivals,
runner/server CPU and RSS, saturation, and hardware sharing (including whether
the generator shares the 2-core/4-thread host). Isolate and rerun if generator CPU
prevents offered load. No v0.7 performance result or gate is claimed here.

### 2026-10-28 initial Public correctness run — load failure

Release `02b9535893f1926ac577a0306d44d66c6aefee7f`, built by passing
[CI run 40711873227](https://github.com/Phastboy/haven/actions/runs/40711873227).
Shared host: i5-3437U, 2 physical cores / 4 threads, 5.8 GiB RAM; application,
PostgreSQL and Go generator share the host with the desktop. Isolated database;
10,000 people / 100,000 offers / 359 cases. Fixture SHA256
`935596856d6abc647f4566c3cc3bd6cce0edf8331761151cb1a205e2f396273b`.

At 100 offered HTTP RPS for 60 seconds: 36.86 achieved RPS (including drain),
p95 11,145.54 ms, p99 12,223.88 ms; 2,737 responses checked, all correct,
zero request failures, 3,263 dropped arrivals. Semantic sample 100%, seed 42.
**Load gate failed; subsequent stages stopped.** Raw summaries and host/process
telemetry are in `/tmp/haven-v07-load` on the measurement host.

Investigation isolated the prepared generic plan for combined category/title/price:
43,788 matching offers were joined and sorted before LIMIT, 150,621 shared-buffer
hits, 677.25 ms execution. Planning the same query with actual values used ordered
keyset streaming and took 0.75 ms. Dynamic public/personal feed statements now
close after execution so PostgreSQL plans for current facet selectivity; rerun
required on the resulting release binary. No v0.7 gate is claimed.

# Load Test Scorecard

| Date | Environment | Scenario | Target RPS | Actual RPS | p95 (ms) | p99 (ms) | Notes |
|---|---|---|---|---|---|---|---|
| 2026-10-05 | QEMU (1vCPU, 2GB RAM, pinned) | `02-steady-state` | 630 iters/s (~750 req/s) | ~750 req/s | < 200ms | < 500ms | Baseline established; 0% error rate; physical capacity ceiling for 1vCPU/2GB |
| 2026-10-12 | Host (4 CPU, native) | `02-steady-state` | 630 iters/s (~750 req/s) | 744 req/s | 3.85ms | 5.65ms | v0.3.x maker identity & profile verification; 0% error rate; 100% checks passed; p95/p99 well within 200ms/500ms contract targets |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v05` | 10 journeys/s (~40 req/s) | 40 req/s (600/600 journeys) | 28.0ms | 31.4ms | v0.5.x search/filter/pagination gate; commit `506c651`; 0% errors; 2,400/2,400 checks passed; 0 dropped. Aggregate reconstructed from per-route summaries; worst route 29.5/34.2ms |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v06` | 1 journey/s @ 16 workers (~172 req/s) | 171.3 req/s (60/60 journeys) | 17.8ms | 23.4ms | v0.6.x save/follow + collection pagination gate; commit `506c651`; 0% errors; 10,320/10,320 checks passed; 0 dropped. Isolated write outliers of 329–365ms (max single request 365.2ms); aggregate reconstructed from per-route summaries |


## v0.7 planned verification — not run

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

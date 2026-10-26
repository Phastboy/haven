# Load Test Scorecard

| Date | Environment | Scenario | Target RPS | Actual RPS | p95 (ms) | p99 (ms) | Notes |
|---|---|---|---|---|---|---|---|
| 2026-10-05 | QEMU (1vCPU, 2GB RAM, pinned) | `02-steady-state` | 630 iters/s (~750 req/s) | ~750 req/s | < 200ms | < 500ms | Baseline established; 0% error rate; physical capacity ceiling for 1vCPU/2GB |
| 2026-10-12 | Host (4 CPU, native) | `02-steady-state` | 630 iters/s (~750 req/s) | 744 req/s | 3.85ms | 5.65ms | v0.3.x maker identity & profile verification; 0% error rate; 100% checks passed; p95/p99 well within 200ms/500ms contract targets |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v05` | 10 journeys/s (~40 req/s) | 40 req/s (600/600 journeys) | 28.0ms | 31.4ms | v0.5.x search/filter/pagination gate; commit `506c651`; 0% errors; 2,400/2,400 checks passed; 0 dropped. Aggregate reconstructed from per-route summaries; worst route 29.5/34.2ms |
| 2026-10-26 | Host (4 CPU, native, debug build; isolated `haven_load_test` DB) | `v06` | 1 journey/s @ 16 workers (~172 req/s) | 171.3 req/s (60/60 journeys) | 17.8ms | 23.4ms | v0.6.x save/follow + collection pagination gate; commit `506c651`; 0% errors; 10,320/10,320 checks passed; 0 dropped. Isolated write outliers of 329–365ms (max single request 365.2ms); aggregate reconstructed from per-route summaries |


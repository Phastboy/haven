# Load Test Scorecard

| Date | Environment | Scenario | Target RPS | Actual RPS | p95 (ms) | p99 (ms) | Notes |
|---|---|---|---|---|---|---|---|
| 2026-10-05 | QEMU (1vCPU, 2GB RAM, pinned) | `02-steady-state` | 630 iters/s (~750 req/s) | ~750 req/s | < 200ms | < 500ms | Baseline established; 0% error rate; physical capacity ceiling for 1vCPU/2GB |
| 2026-10-12 | Host (4 CPU, native) | `02-steady-state` | 630 iters/s (~750 req/s) | 744 req/s | 3.85ms | 5.65ms | v0.3.x maker identity & profile verification; 0% error rate; 100% checks passed; p95/p99 well within 200ms/500ms contract targets |


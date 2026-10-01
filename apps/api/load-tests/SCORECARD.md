# Performance Scorecard

Record the results of the performance test suite here after every meaningful run.
This file serves as a git-tracked history of Haven's true capabilities over time.

---

## Run Template

Copy this block to the top of the history when recording a new run.

## History

### Run: 2026-10-01 — git sha 8e22215 (Composite Index Optimized)

**Normal (500 RPS)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| Achieved RPS   | ≥ 500    | 341     | ❌     |
| p50            | < 10ms   | 649ms   | ❌     |
| p95            | < 50ms   | 2.0s    | ❌     |
| p99            | < 100ms  | 2.65s   | ❌     |
| Error rate     | < 0.1%   | 0.00%   | ✓      |
| Correctness    | 100%     | 100%    | ✓      |
| CPU peak       | < 80%    | 28.3%   | ✓      |
| RSS peak       | -        | 191 MB  | 📊     |
| DB conn peak   | ≤ 20     | 15      | ✓      |

*(Note: DB is now perfectly healthy. Remaining latency is caused by API-side N+1 auth queries blocking the event loop and connection pool)*

### Run: 2026-10-01 — git sha eea0a49 (First Contract Baseline)

**Normal (500 RPS)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| Achieved RPS   | ≥ 500    | 328     | ❌     |
| p50            | < 10ms   | 725ms   | ❌     |
| p95            | < 50ms   | 2.30s   | ❌     |
| p99            | < 100ms  | 2.68s   | ❌     |
| Error rate     | < 0.1%   | 0.00%   | ✓      |
| Correctness    | 100%     | 100%    | ✓      |
| CPU peak       | < 80%    | n/a     | ⚠️     |
| RSS peak       | -        | n/a     | ⚠️     |
| DB conn peak   | ≤ 20     | n/a     | ⚠️     |

*(System monitor failed to attach to the API process on this run. Data will be collected on the next test.)*

**High Load (1,000 RPS)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| Achieved RPS   | ≥ 1,000  | ???     | ⏳     |
| p95            | < 100ms  | ???     | ⏳     |
| p99            | < 250ms  | ???     | ⏳     |
| Error rate     | < 0.1%   | ???     | ⏳     |

**Spike (2,000 RPS burst)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| Spike p95      | < 1000ms | ???     | ⏳     |
| Spike errors   | < 1%     | ???     | ⏳     |
| Recovery p95   | < 100ms  | ???     | ⏳     |
| Recovery errs  | < 0.1%   | ???     | ⏳     |

**Soak (500 RPS × 30m)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| p95            | < 100ms  | ???     | ⏳     |
| Error rate     | < 0.1%   | ???     | ⏳     |
| RSS Growth     | ≈ 0      | ???     | ⏳     |

**Capacity Ceiling**
| Metric         | Actual  | 
| -------------- | ------- | 
| Max Sustained  | ??? RPS | 
| Bottleneck     | ???     | 

---

## History

*(No runs recorded yet)*

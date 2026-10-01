# Performance Scorecard

Record the results of the performance test suite here after every meaningful run.
This file serves as a git-tracked history of Haven's true capabilities over time.

---

## Run Template

Copy this block to the top of the history when recording a new run.

### Run: YYYY-MM-DD — git sha xxxxxxx

**Normal (500 RPS)**
| Metric         | Target   | Actual  | Status |
| -------------- | -------- | ------- | ------ |
| Achieved RPS   | ≥ 500    | ???     | ⏳     |
| p50            | < 10ms   | ???     | ⏳     |
| p95            | < 50ms   | ???     | ⏳     |
| p99            | < 100ms  | ???     | ⏳     |
| Error rate     | < 0.1%   | ???     | ⏳     |
| Correctness    | 100%     | ???     | ⏳     |
| CPU peak       | < 80%    | ???     | ⏳     |
| RSS peak       | -        | ???     | 📊     |
| DB conn peak   | ≤ 20     | ???     | ⏳     |

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

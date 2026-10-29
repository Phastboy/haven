# A4 Measurement State

Updated 2026-10-29. A4 remains **in progress** in
[`PERFORMANCE_AND_JOURNEY_CI_TASKS.md`](../../PERFORMANCE_AND_JOURNEY_CI_TASKS.md).
This file carries the current evidence, affected code, and next action so the
work can resume without relying on chat history.

## Current evidence

The pool acquisition signal and load-gate reporting were added in commits
`82e18eb` and `518d808`. The opt-in instrumentation is enabled only when
`HAVEN_POOL_ACQUIRE_LOG` is set. It records SQLx `acquired_after_secs` events
off the request path and reports per-phase count and p50/p95/p99. The duration
is connection acquisition time, including connection creation and pool waiting;
it is not a pure semaphore-wait measurement.

[CI 40852572648](https://github.com/Phastboy/haven/actions/runs/40852572648)
passed on `518d808`: lint, build, Rust/Go checks, messaging correctness, and all
546 narrowing fixture cases / 1,081 HTTP checks. Its low-load correctness sweep
captured 2,946 acquisitions (p50 0.126 ms, p95 0.182 ms, p99 0.308 ms).
Fixture SHA256:
`09fcc273f7d74b03012aa2045a1b47f22401f1a26b133da93daae2c02a31390e`.

[Sustained CI 40854183351](https://github.com/Phastboy/haven/actions/runs/40854183351)
used fixture SHA256
`4f4e70680faa6be6a19abe33cec0c8145b22fa420299afbce62095787cd3c1e0` on an
Ubuntu hosted runner reporting AMD EPYC 7763, 2 cores / 4 vCPUs. The app,
PostgreSQL and Go load generator shared that runner. At 100 journeys/sec:

| Phase | HTTP RPS | Page p95 | Unread p95 | Combined journey p95 | Pool acquire p95 | Dropped |
|---|---:|---:|---:|---:|---:|---:|
| Guest Public | 100.0 | 41.5 ms | — | 41.5 ms | 2.06 ms | 0 |
| Authenticated Public, dense | 200.0 | 18.7 ms | 8.4 ms | 25.7 ms | 1.58 ms | 0 |
| Authenticated Public, sparse | 200.0 | 19.3 ms | 8.2 ms | 25.7 ms | 1.53 ms | 0 |
| Authenticated Public, zero | 200.0 | 19.4 ms | 8.6 ms | 25.7 ms | 1.60 ms | 0 |
| Saved, dense | 154.3 achieved | 1,838.6 ms | 992.7 ms | 2,671.0 ms | 552.8 ms | 1,088 |

Saved dense is the first failed correctness phase. The gate stopped before Saved
sparse/zero, Inquired, and all contract/mixed/spike/soak/capacity stages. Semantic
checks for completed requests passed. The long page and unread times coincide
with high pool acquisition latency, so pool contention is a measured bottleneck
under dense Saved load; the feed query may also contribute. This run does not
isolate those costs causally.

## Changes and effects

- `crates/haven-web/src/main.rs` enables SQLx acquisition events only when the
  telemetry environment variable is set; normal app runs leave it disabled.
- `crates/haven-web/src/pool_acquire.rs` collects acquisition durations in
  memory and flushes them in batches every 250 ms. The HTTP request path does not
  write or flush the telemetry file for each acquire.
- `crates/haven-web/load-tests/journey-correctness.py` adds acquisition
  percentiles to its private run result. The live correctness sweep confirmed
  nonempty telemetry.
- `crates/haven-web/load-tests/v07-gate.py` adds per-phase acquisition
  percentiles. Commit `76c8847` changes failure handling to finish all
  correctness strata after a correctness failure, then stop before contract,
  mixed, spike, soak and capacity phases. `gate-status.json` records completed
  and skipped stages. The gate still fails when any correctness phase fails.
- `.github/workflows/haven-rust-ci.yml` enables acquisition telemetry only for
  the isolated v0.7 load gate.

## Pending run and next action

[CI 40856357413](https://github.com/Phastboy/haven/actions/runs/40856357413)
was dispatched on `76c8847` to complete the remaining correctness strata after
the Saved dense failure. Per the user's instruction, this run was not watched.
At the next session:

1. Check the run's final status and download its `v07-gate` artifact.
2. Read `gate-status.json`; confirm each Public, Saved, and Inquired history has
   a phase summary or an explicit infrastructure failure.
3. Compare page, unread, combined journey, and pool acquisition percentiles by
   class/history/filter/page depth. Preserve failed and skipped-stage data.
4. Add the outcome to `load-tests/SCORECARD.md` and update A4 in the checklist.
   Keep A4 unchecked until a comparable before/after run and cover-delivery scope
   are addressed.
5. Use the dense personal-feed results to choose the A5/A6 investigation. Do not
   infer that a larger pool alone fixes the underlying query cost.

## Still unmeasured

- Actual Cloudinary image transfer is external to the Go HTTP runner. Page
  latency includes cover metadata/database work and HTML URL rendering, but not
  CDN bytes or browser image decode.
- The historical pre-fix run 40719487321 is not a controlled before/after pair:
  its fixture and request accounting differ from the current authenticated
  page-plus-unread journey. Do not compare those values as a causal improvement.
- Saved sparse/zero, all Inquired strata, and the other requested filter/page
  combinations await the pending diagnostic artifact.
- No contract, mixed, spike/recovery, soak, capacity, or release gate passed.
  A4, A5, A6, B3–B7, and v0.7 release completion remain open.

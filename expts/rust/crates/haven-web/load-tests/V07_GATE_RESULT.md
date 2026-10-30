# v0.7 full release gate — 2026-10-30

> Release decision update — 2026-10-30: the owner accepts v0.7 for release
> with documented performance/verification debt deferred to the development
> freeze. See [release acceptance](../../../V0.7_RELEASE.md) and
> [debt register](../../../V0.7_DEBT.md). The failed/omitted measurements below
> are unchanged. Historical “blocked” statements describe the technical gate;
> they no longer prohibit the scoped release exception. No performance pass is claimed.


Technical outcome: **Failed.** Guest Public failed the first contract-rate stage.
The owner subsequently accepted the explicit release exception; v0.7.0 is
published with the unresolved performance and verification debt. This run does
not establish a full performance pass or verified production deployment.

- Run: [41072121846](https://github.com/Phastboy/haven/actions/runs/41072121846)
- Branch: `plan/v0.7-discovery`
- Revision: `4c379aafb4b59b109ee11e180b438cf640ec1eb9`, containing `dfb79ad`.
- Dispatch: 2026-10-30 14:30:27 UTC. Gate workload: 14:39:22–14:50:27 UTC.
- Standard checks, messaging correctness and narrowing sweep passed.
- Hardware: GitHub runner, 4 logical CPUs, AMD EPYC 7763; server and runner
  share the host, with PostgreSQL in a service container. Pool size: 20.
- Raw JSON, revision and hardware: [evidence/41072121846](evidence/41072121846).
  Full downloaded `v07-gate` artifact: `/tmp/haven-v07-gate-41072121846`.
  GitHub artifact retention is seven days; local summaries are retained here.

## Correctness

Each stratum offered 100 journeys/s for 60 seconds with 100% semantic checks.
All 60,000 journeys completed, all 114,000 HTTP checks passed, no request failures
or dropped arrivals. Authenticated journeys include the page followed by unread.
The final column is the worst per-case combined journey p95, not aggregate HTTP p95.

| Stratum | Completed journeys | HTTP checks passed | Worst journey p95 ms |
|---|---:|---:|---:|
| inquired-dense | 6,000 | 12,000 | 21.84 |
| inquired-sparse | 6,000 | 12,000 | 5.54 |
| inquired-zero | 6,000 | 12,000 | 4.16 |
| public-all | 6,000 | 6,000 | 46.62 |
| public-authenticated-dense | 6,000 | 12,000 | 55.13 |
| public-authenticated-sparse | 6,000 | 12,000 | 51.47 |
| public-authenticated-zero | 6,000 | 12,000 | 58.80 |
| saved-dense | 6,000 | 12,000 | 9.79 |
| saved-sparse | 6,000 | 12,000 | 7.96 |
| saved-zero | 6,000 | 12,000 | 6.14 |

## Contract blocker

`contract-public-all` offered 1,010 guest Public journeys/s for 60 seconds.
It scheduled 60,600, started/completed 19,630 and dropped 40,970 (67.6%).
Achieved throughput including drain was 321.79 journeys/s (one HTTP request per
journey), below the 1,000/s minimum. All 19,630 completed HTTP checks and all
1,934 sampled semantic cases passed; no completed request failed.

HTTP p95/p99 were 933.28/1,034.80 ms. Worst route p95/p99 were
1,044.67/1,111.46 ms (`public/none/free/later`), exceeding the 200/500 ms
route budgets. Pool acquisition p95/p99 were 354.08/393.15 ms (wait plus
connection creation). These show load failure and pool contention, but do not
isolate its underlying query or resource cause. This is not a measured capacity
ceiling. Do not lower targets or restart the concluded Saved investigation.

## Skipped stages and measurement limits

Nine authenticated/personal contract stages, mixed traffic, spike/recovery,
30-minute soak and capacity were explicitly `not_run_after_failure`.
Saved and Inquired contract-rate performance is therefore unverified here.

Telemetry has 133 samples spanning about 662 seconds. Observed peak server RSS
was 32,236 KiB and load-runner RSS 24,600 KiB. No PostgreSQL processes were captured
by the host process sampler (the database runs in a container); DB process CPU/RSS
is unavailable. Short-run samples do not establish soak stability or CPU headroom.
Pool acquisition instrumentation was populated for every measured stage. This
HTTP runner includes server cover/status work but excludes external image transfer
and browser decode. Cumulative journey coverage and enforcement remain unfinished.

## Disposition

v0.7.0 was published under the owner-approved exception. Public diagnosis and
its rejected equality experiment are recorded in
[V0.7_DEBT.md](../../../V0.7_DEBT.md). Further maintenance and full-gate
verification are deferred to the development freeze; this report does not
authorize a new investigation or rerun.

# Load Test Scorecard

> Release decision update — 2026-10-30: the owner accepts v0.7 for release
> with documented performance/verification debt deferred to the development
> freeze. See [release acceptance](../../../V0.7_RELEASE.md) and
> [debt register](../../../V0.7_DEBT.md). The failed/omitted measurements below
> are unchanged. Historical “blocked” statements describe the technical gate;
> they no longer prohibit the scoped release exception. No performance pass is claimed.


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
prevents offered load. No v0.7 contract or capacity gate is claimed here.

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

A separate 10 RPS Inquired diagnostic on the initial binary returned 400/400
correct responses, zero dropped arrivals, p95 1,224.77 ms and p99 3,005.60 ms.
The dense broad query also exposed a maker join executing a sequential scan of
`user` for each candidate (105,948 rejected join rows for 21 results). Bounding
membership/facets/order to 21 rows in a materialized CTE before the maker join
changed that join to primary-key lookups: 20.43 ms to 1.07 ms in the isolated SQL
comparison. This preserves visibility, membership and pagination semantics.

### 2026-10-28 Public rerun after planning fix — contract failure

Release `3c54037`, passing [CI run 40716208008](https://github.com/Phastboy/haven/actions/runs/40716208008),
same fixture/host as above. At 100 offered RPS: 99.85 achieved, p95 118.12 ms,
p99 160.82 ms; 6,000/6,000 checks correct, no errors/drops (100% semantic checks).
At 1,010 offered RPS: 197.29 achieved, p95 1,279.97 ms, p99 1,374.91 ms;
12,154/12,154 status/check outcomes passed, 48,446 dropped arrivals. Semantic
checking was stratified 10%, seed 42; per-case counts remain in the raw JSON.
**Contract failed.** Host load rose to about 17 on four logical CPUs. These are
shared-desktop measurements, not evidence of isolated production capacity.
The next run uses an opt-in CI workload with a fresh database, the complete
correctness/contract/mixed/spike/30-minute-soak/capacity sequence, and exported
CPU/RSS/descriptor telemetry. The runner and database share that clean CI host;
its hardware and source SHA are exported with results. No release bump yet.

### Personal-feed investigation — unread navigation work

CI run 40719487321 (`0d0033a`) passed build/lint/regressions but stopped at
Inquired correctness: Public 100 RPS / p95 41 ms / p99 45 ms; Saved 94.69 RPS /
p95 2,939 ms / p99 3,615 ms; Inquired 67.82 RPS / p95 4,433 ms / p99 5,163 ms,
with 1,583 dropped arrivals. All 16,418 completed responses passed checks.
Contract/mixed/spike/soak/capacity stages did not execute.

Read-only local EXPLAIN ANALYZE comparisons on the existing isolated fixture
compared actors 0 (dense) and 8 (sparse). Warm-buffer execution times:

| Query | Dense ms | Sparse ms |
|---|---:|---:|
| Navigation unread check | 31.945 | 0.089 |
| Saved membership, broad first page | 0.399 | 0.250 |
| Inquired membership, broad first page | 0.982 | 0.260 |
| Saved empty category | 0.346 | 0.110 |
| Inquired empty category | 0.057 | 0.070 |
| Saved dominant category | 0.518 | 0.165 |
| Inquired dominant category | 0.769 | 0.340 |

The unread check visited 6,484 conversations and 26,369 shared buffers to return
false for the dense actor. It executes in the authenticated page layout, so both
personal collections incur this cost. Cold measurements were 51.533 ms dense /
0.143 ms sparse. Feed measurements isolate membership/order/visibility/facets,
not the complete HTTP request, maker/cover loading, or every fixture case.
These local numbers identify repeated work; they do not attribute all CI latency.
Raw plans and the reproduction script are under `/tmp/haven-v07-load/` as
`compare-personal.txt`, `compare-personal-warm.txt`, and `compare-personal.py`.

Next optimization to evaluate: maintain indexed per-participant unread state in
the message/read transaction so the navigation can find an unread conversation
without walking every read conversation. Preserve latest-sender semantics,
participant authorization, and read/send concurrency; verify summary backfill and
fixture consistency before rerunning HTTP gates. No release gate is claimed.

### 2026-10-29 complete correctness matrix — dense personal feeds still fail load

[CI run 40856357413](https://github.com/Phastboy/haven/actions/runs/40856357413)
on `76c8847` completed the full 10-stratum correctness matrix. Nine strata
passed; Inquired dense failed its arrival gate. All completed responses passed
semantic checks (100%). At 100 target journeys/s, each passing stratum completed
6,000/6,000 journeys with no drops. Inquired dense completed 4,027/6,000,
dropped 1,973, and completed 8,054/8,054 HTTP checks correctly.

| Class / history | Journey p95 | Pool-acquisition p95 | Result |
|---|---:|---:|---|
| Public / all | 34 ms | 1.83 ms | 6,000/6,000, pass |
| Authenticated Public / dense, sparse, zero | 34–35 ms | 1.08–1.14 ms | each 6,000/6,000, pass |
| Saved / dense | 747 ms | 106 ms | 6,000/6,000, pass correctness |
| Saved / sparse, zero | 2.1–2.8 ms | 0.13–0.14 ms | each 6,000/6,000, pass |
| Inquired / dense | 3,007 ms | 745 ms | 4,027/6,000; 1,973 dropped |
| Inquired / sparse, zero | 2.5–3.5 ms | 0.13–0.14 ms | each 6,000/6,000, pass |

Dense personal feeds show severe connection-acquisition delay alongside high
request latency; this run does not establish the causal split between queued
pool demand and query execution. Contract, mixed, spike, soak and capacity
stages did not run. The app, PostgreSQL and generator shared a 2-core/4-vCPU AMD
EPYC hosted runner. Fixture SHA256:
`fc4e44960090d549551c2d9b4963e3986a9a92c2209b22827ea15bb6611117d5`. The
`v07-gate` artifact retains per-stratum summaries and telemetry. Investigate
dense Inquired feed/unread query work, retaining dense Saved as a regression
case; then rerun correctness and contract gates. Actual Cloudinary delivery and
a controlled before/after run remain unmeasured. A4 remains open.

### 2026-10-29 local Inquired summary comparison — diagnostic only

On the same disposable 10k-person/100k-offer dense fixture, replacing read-time
latest-message probes with migration 020's maintained per-person/per-offer
activity rows changed the 100 journey/s correctness replay from 2,323/5,999
completed journeys, 3,676 drops and 5,648 ms journey p95 to 6,000/6,000, zero
drops and 1,544 ms p95. Pool-acquisition p95 fell from 1,417 ms to 436 ms.
All 12,000 completed HTTP checks passed after the change.

Read-only plans used `idx_person_inquired_offer_activity`, scanning 21 summary
rows for broad first page and 47 for combined later page; isolated execution was
1.12 ms and 0.95 ms. PostgreSQL CPU averaged 2.83 cores before and 1.76 after;
host CPU remained near saturation. The local host was the 2-core/4-vCPU Intel
i5-3437U machine, and this sequential comparison is not a contract or CI gate.
The 1,544 ms after p95 remains above budget. See `A4_STATE.md` for route-level
results and artifacts. Clean-CI correctness and performance validation remain
pending; no release gate is claimed.

### 2026-10-29 post-summary local release correctness rerun — diagnostic only

With migration 020 present and 80,040 activity rows backfilled, the release
runner passed 9/10 correctness strata on the same local 10k-person fixture.
Dense Inquired now completed 6,000/6,000 journeys with 12,000/12,000 HTTP
checks, no drops, 74 ms journey p95, and 4 ms pool-acquisition p95. Dense Saved
failed independently: 3,254/6,000 completed, 2,746 dropped, 3,964 ms journey
p95, and 945 ms acquisition p95. Contract stages did not run. This reused
fixture/local-host run is diagnostic, not a clean-CI gate. Host load averaged
14.9 and peaked at 35.9; minimum available memory was 0.66 GiB. Representative
Saved query plans used the existing indexes and completed in 22 ms (broad) and
75 ms (rare category) in isolation. The dense Saved result is inconclusive as a
code regression; rerun on clean CI before changing its query. Reports:
`/tmp/haven-a4-clean-replay/`; see `A4_STATE.md`.

### 2026-10-30 clean CI rerun after Inquired and Saved feed changes

[CI run 41026033154](https://github.com/Phastboy/haven/actions/runs/41026033154)
on `07ba7a0` passed formatting, Clippy, build, Rust/script tests, messaging
correctness, and documentation. All ten correctness strata ran; nine completed
6,000/6,000 journeys with no drops, and all completed checks passed. Dense Saved
completed 5,017/6,000 journeys and dropped 983 arrivals; its 10,034/10,034
completed HTTP checks passed. Dense Inquired completed 6,000/6,000 with no
drops, at 14.7 ms journey p95 and 0.2 ms pool-acquisition p95.

| Class / history | Journey p95 | Pool-acquisition p95 | Result |
|---|---:|---:|---|
| Public / all | 40.9 ms | 2.0 ms | 6,000/6,000, pass |
| Authenticated Public / dense, sparse, zero | 43.0–43.1 ms | 1.5–1.6 ms | each 6,000/6,000, pass |
| Saved / dense | 2,323 ms | 549 ms | 5,017/6,000; 983 dropped |
| Saved / sparse, zero | 2.4–3.4 ms | 0.2 ms | each 6,000/6,000, pass |
| Inquired / dense | 14.7 ms | 0.2 ms | 6,000/6,000, pass |
| Inquired / sparse, zero | 2.4–3.4 ms | 0.2 ms | each 6,000/6,000, pass |

Dense Saved page p95 ranged from 956 ms (empty) to 3,270 ms (combined later
page); unread follow-ups were about 948–972 ms. The new Saved covering index did
not resolve its arrival failure. This is a clean hosted-runner reproduction, but
pool acquisition and route latency are correlated measurements, not isolated
causal costs. Contracts, mixed, spike, soak and capacity did not run after the
correctness failure. The fixture SHA256 was
`bc299a54f174cf27bad312ee5fe9aa3e94946235e266485122f5dd038e8deca5`; app,
PostgreSQL and generator shared a 2-core/4-vCPU AMD EPYC 7763 host. See
`A4_STATE.md` for per-route summaries and artifact location. A4 remains open.

### 2026-10-30 local dense Saved investigation — diagnostic only

On a disposable 10k-person/100k-offer, 530-case fixture using migration 021,
the Saved covering index produced an index-only plan for broad pages (21
membership rows, zero heap fetches). Empty-category execution was 1.36 ms;
rare-category execution was 18.62 ms on a cold cache; combined category/title/
price first/later pages were 2.31/2.96 ms. These plans did not reproduce the
loaded latency and the local fixture hash differs from CI.

The same fixture under a local 100-journey/s Saved-dense replay completed
3,466/6,000 journeys, dropped 2,534, and passed all 6,932 completed HTTP checks;
journey p95 was 3.44 s and pool-acquisition p95 873 ms. During the replay,
`pg_stat_activity` sampled 10–14 active personal-feed statements, with some
active for over 1.6 seconds and no lock or I/O wait event reported. A temporary
change to skip Saved-status lookup did not provide a reliable before/after
result: the second run had 3.16 s journey p95 but fewer completions (3,395) and
1,047 ms acquisition p95. That change was reverted. This is diagnostic evidence
only; profile query planning and execution on a comparable clean host before
another Saved-feed rewrite. See `A4_STATE.md` for commands and the artifact path.

### 2026-10-30 final Saved verification — bottleneck resolved

Fix `dfb79ad`, CI [41045410575](https://github.com/Phastboy/haven/actions/runs/41045410575),
passed the full 530-case sweep and Saved at 25/50/75/100 journeys/s. At 100,
all 6,000 journeys and 12,000 HTTP checks passed with zero drops; HTTP p95 was
3.07 ms versus 1,620.93 ms in the instrumented baseline. Worst per-case full
journey p95 was 4.60 ms. Price-filtered Saved statements were responsible for
99.46% of baseline execution time; dependent offer lookups removed the broad
scan work. See [SAVED_PROFILE_RESULT.md](SAVED_PROFILE_RESULT.md) for evidence
and limitations. The investigation is concluded; broader release gates remain.


## Full release gate — 2026-10-30

[CI 41072121846](https://github.com/Phastboy/haven/actions/runs/41072121846)
on `4c379aa` failed at guest Public contract load. All ten correctness strata
passed (60,000 journeys, 114,000 HTTP checks, zero failures/drops). Public
contract load achieved 321.79/s with 40,970 dropped arrivals; HTTP p95 933.28 ms,
pool acquisition p95 354.08 ms. Remaining nine contract stages and mixed,
spike/recovery, soak and capacity were not run. v0.7 remains blocked; no tasks
closed or version bumped. See [V07_GATE_RESULT.md](V07_GATE_RESULT.md) for retained evidence and instrumentation gaps.

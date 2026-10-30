# A4 Measurement State

**Resume here first:** [NEXT_SESSION.md](../../../../NEXT_SESSION.md) records the
deferred full v0.7 release-gate run and its state. The Saved investigation is
finished; the next session is release verification.

Updated 2026-10-30. A4 remains **in progress** in
[`PERFORMANCE_AND_JOURNEY_CI_TASKS.md`](../../PERFORMANCE_AND_JOURNEY_CI_TASKS.md).
This file carries the current evidence, affected code, and next action so the
work can resume without relying on chat history.

The dense Saved investigation is concluded. Fix `dfb79ad` passed CI 41045410575:
6,000/6,000 journeys at 100 journeys/s, zero drops, all 12,000 HTTP checks
correct, and 3.07 ms HTTP p95. See [SAVED_PROFILE_RESULT.md](SAVED_PROFILE_RESULT.md)
for the cause, single verification, and remaining release gates. Older results
below are historical; some mislabeled HTTP p95 as journey p95.

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

## Run queued at the previous checkpoint

The previous checkpoint's immediate steps (inspect run status/artifact, check
matrix coverage, and record the outcome) are complete. See the diagnostic result
below. The dense personal-feed results now direct the next investigation; do not
infer that a larger pool alone fixes the underlying query cost.

## Diagnostic run result — 2026-10-29

[CI 40856357413](https://github.com/Phastboy/haven/actions/runs/40856357413)
completed on `76c884712f914e045ff55f94668a65914d48c329` and failed as expected
after finishing the correctness matrix. The `v07-gate` artifact reports all
Public, Saved and Inquired sparse/zero strata plus Public all and authenticated
Public dense as passed. Saved dense also passed correctness. Inquired dense was
the single failed stratum; contracts, mixed, spike, soak and capacity were
skipped after that correctness failure.

All 10 strata report 100% semantic correctness for completed requests. At 100
target journeys/s over 60 seconds, the nine passing phases completed all 6,000
scheduled journeys with no drops. Inquired dense completed 4,027/6,000 journeys,
dropped 1,973 arrivals, and had no failed completed requests. This is an
arrival/load failure, not a semantic correctness failure.

| Class / history | Journey p95 | Pool acquisition p95 | Completed / scheduled | Dropped |
|---|---:|---:|---:|---:|
| Public / all | 34.3 ms | 1.83 ms | 6,000 / 6,000 | 0 |
| Public authenticated / dense | 35.3 ms | 1.10 ms | 6,000 / 6,000 | 0 |
| Public authenticated / sparse | 35.1 ms | 1.14 ms | 6,000 / 6,000 | 0 |
| Public authenticated / zero | 34.1 ms | 1.08 ms | 6,000 / 6,000 | 0 |
| Saved / dense | 747.5 ms | 106.1 ms | 6,000 / 6,000 | 0 |
| Saved / sparse | 2.75 ms | 0.133 ms | 6,000 / 6,000 | 0 |
| Saved / zero | 2.10 ms | 0.139 ms | 6,000 / 6,000 | 0 |
| Inquired / dense | 3,007.3 ms | 745.0 ms | 4,027 / 6,000 | 1,973 |
| Inquired / sparse | 3.47 ms | 0.133 ms | 6,000 / 6,000 | 0 |
| Inquired / zero | 2.52 ms | 0.136 ms | 6,000 / 6,000 | 0 |

Sparse/zero page and unread routes stayed in low single-digit milliseconds; the
authenticated Public page routes were about 34–35 ms p95, with unread routes
around 8–9 ms. Dense Saved page and unread routes both degraded (roughly 747 ms
and 440 ms p95 respectively). Dense Inquired routes ranged from about 1.2 s
unread p95 to 3.63 s combined-filter/later-page p95. Pool acquisition latency
rose sharply in both dense personal feeds, reaching 745 ms p95 for Inquired.
These measurements show severe pool-acquisition delay alongside slow feed
requests; they do not isolate whether query execution, queued demand, or both
caused that delay. The earlier run's dense Saved pool contention was therefore
not cleared by this run; it was reproduced at lower severity.

The artifact fixture SHA256 is
`fc4e44960090d549551c2d9b4963e3986a9a92c2209b22827ea15bb6611117d5`. Hardware
was a shared 2-core / 4-vCPU AMD EPYC 9V74 hosted runner, with app, PostgreSQL,
and generator on the same host. The `v07-gate` artifact was downloaded to
`/tmp/haven-a4-40856357413`; it contains the per-stratum JSON/logs, gate status,
host details and acquisition telemetry.

Next: investigate dense personal-feed query work and the associated acquisition
delays, prioritizing Inquired dense and comparing its query plans and individual
feed/unread routes. Retain dense Saved as a second regression case. Rerun the
correctness matrix after any fix, then run the contract stages separately. This
run still does not measure actual Cloudinary transfer/decode or provide a
controlled before/after comparison; no A4 or release gate is complete.

### Query-path inspection

Static inspection confirms that `/unread-status` calls `unread_exists`, whose
predicate is `user_id = $1 AND is_unread` and is backed by the partial index
`idx_participant_unread_user` (`migrations/019_indexed_unread_state.sql`). The
prior isolated EXPLAIN recorded about 31.9 ms warm for the dense unread check.
In this run, however, both page and unread routes degraded in dense personal
strata while connection acquisition also rose sharply. That points to shared
database/pool saturation under load, but does not show whether the personal-feed
query, unread query, or their combined demand is the initiating cost.

The Inquired feed in `crates/haven-db/src/offers/personal.rs` joins the focal
person's offer-tagged messages and uses a correlated latest-message lookup for
each candidate (`x.id = (SELECT newer.id ... ORDER BY created_at DESC, id DESC
LIMIT 1)`). Migration 018 supplies sender/offer/conversation/time indexes for
this lookup.

A disposable PostgreSQL 18.6 cluster and `haven_a4_replay` database were created
under `/tmp`; the current narrowing generator seeded 10,000 people, 100,000
offers and 530 cases, including 6,484 conversations and 10,000 inquired offers
(40,000 tagged messages including replies) for the dense-history actor. The
fixture SHA256 is
`9e46873c5d2031249748c78c4f6851f6c01df4bf6f51747c1a248c1cc3935e48`. It has the
same scale and dense-history construction as CI, but is not the exact CI fixture
(CI artifact SHA256
`fc4e44960090d549551c2d9b4963e3986a9a92c2209b22827ea15bb6611117d5` and 546
cases).

Read-only `EXPLAIN (ANALYZE, BUFFERS)` on this fixture showed the broad Inquired
first page at 1.19 ms and the combined-filter later page at 1.36 ms. The broad
plan streamed `idx_message_sender_activity`, checked 61 candidate rows through
`idx_message_sender_offer_latest`, and returned 21 offers. The combined later
page used the same latest-message index for its probes. The empty category case
used `idx_offer_category_created` and took 0.16 ms. The unread predicate used
`idx_participant_unread_user` and took 0.03 ms; the Saved broad page took 0.50
ms. These isolated plans do not show an obviously expensive operator on this
fixture.

A 60-second local replay at 100 target journeys/s (same visible 2-core / 4-vCPU
class as CI, but an Intel i5-3437U host with 5.7 GiB RAM) reproduced overload:
2,255/6,000 journeys completed, 3,745 dropped, and 4,510/4,510 completed HTTP
checks passed. Journey p95 was 6.06 s. With SQLx acquisition telemetry enabled,
12,826 samples had p50 914 ms, p95 1,558 ms and p99 1,894 ms. The local numbers
are worse than CI and are not a comparable benchmark, but they confirm that
pool-acquisition delay returns under concurrent dense load even though the
feed and unread plans are fast in isolation. This points to contention or host
saturation under the complete workload; it does not identify which DB calls or
host resource first saturate.

The read-only SQL used for plans is `/tmp/haven-a4-explain.sql`; results are in
`/tmp/haven-a4-explain-results.txt`. The local runner summary, fixture and pool
samples are in `/tmp/haven-a4-inquired-dense-local-instrumented.json`,
`/tmp/haven-a4-replay-cases.json`, and `/tmp/haven-a4-local-pool.txt`. Haven and
the disposable PostgreSQL server have been stopped. Their data remains isolated
under `/tmp/haven-a4-pgdata` and `haven_a4_replay`; no query rewrite is justified
yet.

### Concurrent activity and resource samples

The instrumented 60-second replay completed 2,323/5,999 journeys, dropped
3,676, and passed all 4,646/4,646 completed HTTP checks. Journey p95 was 5.65 s.
Pool acquisition had p50 884 ms, p95 1,417 ms and p99 1,971 ms across 13,140
samples. PostgreSQL activity samples identify the resource pressure more
clearly:

- Host CPU averaged 97.7% and reached 100% in sampled intervals.
- PostgreSQL processes averaged about 283% CPU (2.83 cores) and reached 351%;
  their aggregate sampled RSS peaked near 2.95 GiB. Haven used about 10% of one
  core on average and the Go runner about 8%.
- Across 23 two-second `pg_stat_activity` samples, active statements were the
  personal-feed `WITH matching` query. There were no lock waits; one sample
  showed `IPC/ExecuteGather`. Active query age reached 2.52 s.
- The indexed unread statement remains fast in isolation (0.03 ms in the plan).
  Its multi-second HTTP latency during overload is consistent with waiting for
  saturated database capacity, rather than an expensive unread index scan.

This evidence points to PostgreSQL CPU saturation from dense personal-feed work
as the immediate source of pool pressure. The active query samples do not
attribute CPU among planning, latest-message probes, membership/filter scans,
or other backend work, and the local host is older and slower than the CI EPYC.
Treat this as a diagnostic reproduction, not a CI performance result.

Host samples, PostgreSQL sessions, pool acquisitions, app/load logs, and summary
are in `/tmp/haven-a4-observed-before-summary/`. The before-run app and temporary
server were stopped after capture.

### Maintained Inquired activity — local before/after

The CI dense Inquired arrival failure and local CPU samples justified the
maintained per-person/per-offer activity fallback already described in
`V0.7_PLAN.md`. Migration `020_person_inquired_offer_activity.sql` backfills the
latest qualifying sent message, maintains it in the message insert transaction,
and adds an ordered `(person_id, last_inquiry_at DESC, offer_id DESC)` index.
Inquired reads now page from that index; Saved continues to use its existing
save-time query.

The same disposable fixture and 100-journey/s, 60-second local Inquired replay
then completed 6,000/6,000 journeys with no drops and passed 12,000/12,000 HTTP
checks. Journey p95 fell from 5.65 s to 1.54 s (73% lower); pool-acquisition p95
fell from 1.42 s to 436 ms (69% lower). This is a directional local comparison,
not a clean CI result: the fixture and host are shared across sequential runs,
and the summary table was warm after migration/backfill. The after-run still had
high latency and host CPU saturation, so it does not meet the contract budget.

| Route | Before p95 | Summary p95 |
|---|---:|---:|
| Inquired broad page | 5.28 s | 1.57 s |
| Inquired combined later page | 6.68 s | 1.68 s |
| Inquired empty page | 2.86 s | 0.88 s |
| Broad unread follow-up | 2.48 s | 0.79 s |

The backfill created 80,040 rows, including all 10,000 inquired offers for the
dense focal person. The existing semantic regression passed, and the local
runner passed all completed semantic checks. Read-only plans used
`idx_person_inquired_offer_activity`, scanning 21 summary rows for broad first
page and 47 for the combined later page; isolated execution was 1.12 ms and
0.95 ms respectively. Under load, PostgreSQL average CPU fell from 2.83 to 1.76
cores and aggregate sampled RSS peak from 2.95 GiB to 1.20 GiB. Host CPU remained
near saturation (97.3% average after), but average active feed-query age fell
from a 2.52 s maximum to 309 ms; no lock waits were sampled.

After-run artifacts are in `/tmp/haven-a4-observed-after-summary/`, and the
summary query plans are in `/tmp/haven-a4-summary-explain-results.txt`. The app
is stopped; the disposable PostgreSQL server is stopped after diagnostics. Next,
validate migration and dense-history performance on a clean CI host, then rerun
the full correctness matrix including Saved dense. Keep A4 unchecked until the
contract stages, external cover-delivery scope and controlled CI comparison are
complete.

### Post-summary release correctness rerun — local diagnostic

After migration 020 was present in the disposable database (80,040 backfilled
rows), the release runner and app repeated all ten correctness strata from the
same 10k-person fixture. Public, all three authenticated Public strata, Saved
sparse/zero, and Inquired dense/sparse/zero passed with no dropped journeys and
all HTTP/semantic checks passing. Dense Inquired completed 6,000/6,000 journeys
at 200 achieved journeys/s, with 74 ms journey p95 and 4 ms pool-acquisition
p95. This confirms the former Inquired-dense correctness arrival failure is
resolved in this local replay.

Dense Saved failed locally: 3,254/6,000 completed, 2,746 dropped, 3,964 ms
journey p95 and 945 ms pool-acquisition p95. The gate stopped before contract
stages. Every completed HTTP and semantic check passed. The shared host was
heavily contended during this run (load average 14.9 on average, 35.9 peak, and
0.66 GiB minimum available memory). Read-only plans for representative Saved
queries completed in 22 ms for the broad first page and 75 ms for the rare
category page; both used the existing Saved ordering/membership indexes. This
does not explain the multi-second loaded latency, so it is not sufficient
evidence for another query rewrite. The failure is inconclusive as a Saved code
regression; reproduce dense Saved on a clean CI host before changing its query.

Reports are in `/tmp/haven-a4-clean-replay/`; the Saved plans were captured
read-only, and the app and disposable PostgreSQL server were stopped afterward.
Run the contract stages after the complete correctness matrix passes on CI.

### Focused clean-CI rerun after migrations 020 and 021 — 2026-10-30

[CI 41026033154](https://github.com/Phastboy/haven/actions/runs/41026033154)
ran on `07ba7a086379abe32d5a4a8b0b06ce0aa1ff6f70`, containing the maintained
Inquired activity migration 020 and the Saved index-only feed migration 021.
Formatting, Clippy, build, Rust and script tests, messaging correctness, and
documentation checks passed. All ten correctness strata ran. Nine completed
6,000/6,000 journeys with no drops; every completed HTTP and semantic check
passed. Dense Saved was the sole arrival failure: 5,017/6,000 completed, 983
dropped, with 10,034/10,034 HTTP checks passing. Dense Inquired completed
6,000/6,000 with no drops.

| Class / history | Journey p95 | Pool acquisition p95 | Completed / scheduled | Dropped |
|---|---:|---:|---:|---:|
| Public / all | 40.9 ms | 2.0 ms | 6,000 / 6,000 | 0 |
| Authenticated Public / dense, sparse, zero | 43.0–43.1 ms | 1.5–1.6 ms | each 6,000 / 6,000 | 0 |
| Saved / dense | 2,323 ms | 549 ms | 5,017 / 6,000 | 983 |
| Saved / sparse | 3.4 ms | 0.2 ms | 6,000 / 6,000 | 0 |
| Saved / zero | 2.4 ms | 0.2 ms | 6,000 / 6,000 | 0 |
| Inquired / dense | 14.7 ms | 0.2 ms | 6,000 / 6,000 | 0 |
| Inquired / sparse | 3.4 ms | 0.2 ms | 6,000 / 6,000 | 0 |
| Inquired / zero | 2.4 ms | 0.2 ms | 6,000 / 6,000 | 0 |

Within dense Saved, the broad first/later page p95s were 1,783/1,786 ms, with
unread follow-ups at 949/948 ms. Combined-filter first/later page p95s were
3,138/3,270 ms; their unread follow-ups were 969/955 ms. Empty-page p95 was
956 ms. These delays coincide with 549 ms acquisition p95. The new covering
index did not clear dense Saved under load; these results still do not isolate
the relative cost of query work and pool demand.

The fixture SHA256 was
`bc299a54f174cf27bad312ee5fe9aa3e94946235e266485122f5dd038e8deca5`. The
runner reported 2 cores / 4 vCPUs on AMD EPYC 7763 with 16 GiB RAM; the app,
PostgreSQL, and load generator shared the hosted runner. The `v07-gate`
artifact is in `/tmp/haven-a4-41026033154`. Correctness stages after Saved
dense continued, confirming the Inquired improvement, but contract, mixed,
spike, soak, and capacity stages were skipped because Saved dense failed.

Next: investigate dense Saved query and pool behavior using the artifact and
read-only plans, then rerun the full correctness matrix. Keep A4 open until all
required contract and later stages pass, cover-transfer scope is addressed, and
a controlled before/after comparison is recorded.

### Local dense Saved query investigation — diagnostic only, 2026-10-30

The CI `v07-gate` artifact does not include PostgreSQL plans or activity samples.
I rebuilt a disposable PostgreSQL 18.6 fixture with the release seeder and
migrations through 021: 10,000 people, 100,000 offers, and 530 cases. Its case
file hash was `5a9fd0f375cbb0e02125ef29fff8eff194ed9050c05f2383534cecc932eda16d`,
so it matches CI's scale and generation rules but not the exact CI fixture hash.

Read-only `EXPLAIN (ANALYZE, BUFFERS)` on representative Saved queries showed
that the new covering index works as intended for broad pages. The broad plan
used an index-only scan of 21 membership rows with zero heap fetches. Selective
plans chose the offer facet indexes and looked up Saved membership by its unique
index. The isolated queries did not reproduce the loaded latency:

| Query | Plan shape | Execution time |
|---|---|---:|
| Broad first page | ordered Saved index-only scan; 21 offer lookups | 1.48 ms |
| Empty category | category index; no Saved membership probes | 1.36 ms |
| Rare category | category index; 97 candidates, then membership lookup and sort | 18.62 ms |
| Combined category/title/price first page | Saved index-only scan; 48 candidates | 2.31 ms |
| Combined category/title/price later page | same keyset index path; 48 candidates | 2.96 ms |

A local 100-journey/s, 60-second Saved-dense replay completed 3,466/6,000
journeys, dropped 2,534, and had 3.44 s journey p95. All 6,932/6,932 completed
HTTP checks passed. During the replay, `pg_stat_activity` samples showed roughly
10–14 active personal-feed statements, some active for over 1.6 seconds, with no
lock or I/O wait event reported. This is consistent with concurrent database
work driving the delay; it does not by itself prove the CPU cost of an
individual plan. The local replay used the same fixture for EXPLAIN and load,
but the host was not constrained or matched to CI.

I temporarily skipped the Saved-status batch lookup on Saved pages, since every
result already comes from that user's Saved membership. The second local replay
did not show a reliable improvement: journey p95 moved from 3.44 s to 3.16 s,
but completed journeys fell from 3,466 to 3,395 and pool-acquisition p95 rose
from 873 ms to 1,047 ms. Because the sequential runs were unisolated and the
outcomes conflict, I reverted that code change. Fixture files, runner summaries,
pool logs, and database files remain under `/tmp/haven-a4-analysis-*` for this
session. The EXPLAIN output is summarized above but was not retained as a raw
plan file.

Current conclusion: the covering index avoids heap reads and representative
isolated plans are fast, while concurrent Saved feed statements run slowly under
the load gate. The source of the per-query CPU cost is still unisolated. Next,
capture per-statement planning/execution totals during a dense Saved replay on a
clean runner (or use a comparable isolated host) before another query rewrite.
Keep A4 open.

## Still unmeasured

- Actual Cloudinary image transfer is external to the Go HTTP runner. Page
  latency includes cover metadata/database work and HTML URL rendering, but not
  CDN bytes or browser image decode.
- The historical pre-fix run 40719487321 is not a controlled before/after pair:
  its fixture and request accounting differ from the current authenticated
  page-plus-unread journey. Do not compare those values as a causal improvement.
- CI 41045410575 passed the full 530-case narrowing sweep and dense Saved
  through 100 journeys/s. Contract-rate and later stages remain unverified.
- No contract, mixed, spike/recovery, soak, capacity, or release gate passed.
  A4, A5, A6, B3–B7, and v0.7 release completion remain open.

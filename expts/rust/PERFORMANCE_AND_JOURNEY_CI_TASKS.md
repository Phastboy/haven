# Performance fixes and cumulative journey CI

Agreed 2026-10-28. This is the execution checklist for personal-feed performance
fixes and a dedicated journey performance workflow on every push. Task status is
recorded below. Check an item only after its completion criteria are met; record the
commit, CI run/report, and relevant measurements beneath the item. Implementation
without verification remains pending. If measurement makes a conditional change
unnecessary, record that evidence and the decision before closing the item.

Product scope remains governed by the repository-root `ROADMAP.md` and
`docs/JOURNEY.md`. Targets remain governed by
[PERFORMANCE_CONTRACT.md](crates/haven-web/load-tests/PERFORMANCE_CONTRACT.md).
This checklist does not mark a product stage or release gate complete.

## Track A — prioritized fixes

- [x] **A1. Indexed unread state — medium.** Maintain per-participant unread state
  atomically with message sends and reads; provide a consistent backfill for
  existing conversations. Preserve participant authorization and current
  latest-sender/read semantics. Verify concurrent sends/reads, empty histories,
  and fixture consistency. Query plans must demonstrate that checking for unread
  messages no longer walks every read conversation in a dense history.
  Evidence: `81f5306` + lint fix `9c50d0e`,
  [CI 40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401).
  Five added database regressions pass, including populated-schema backfill,
  concurrent sends/reads, unauthorized access and bulk fixture refresh. The dense
  plan test seeds 2,048 read conversations and asserts use of
  `idx_participant_unread_user`, with no sequential scan or message-index walk.
  End-to-end HTTP improvement remains unmeasured (A4).
- [x] **A2. Independent unread loading — medium; follows A1.** Render the page
  without awaiting unread status. Fetch it through a small authenticated endpoint
  with private response handling. Keep the dot hidden while initial status is
  unknown; endpoint failure must leave the page usable. Verify page rendering
  completes independently of a delayed or failed unread request.
  Evidence: `24b02db`,
  [CI 40838910534](https://github.com/Phastboy/haven/actions/runs/40838910534).
  Four added HTTP regressions pass: blocked/failing unread storage while pages
  complete, private 401 for guests/invalid sessions, and session-only identity.
  Four script tests cover delayed status, false status, malformed/failed responses
  and no request without signed-in navigation. Both navigation indicators begin
  hidden; public offer detail receives the same independent script. Workspace and
  browser-script checks pass. Refresh/reuse remains A3.
- [x] **A3. Reuse and refresh unread status — small–medium; follows A2.** Reuse
  status during navigation where the runtime supports it. Refresh after sending
  or reading messages and on returning to the app. Prevent duplicate concurrent
  checks and clear person-specific state on sign-out/account changes. Verify
  refresh behavior and request counts across navigation.
  Evidence: `3d4d2c4` + CI setup fix `32bb582`,
  [CI 40841600614](https://github.com/Phastboy/haven/actions/runs/40841600614).
  Ten unread script tests verify runtime reuse, thread/live-content refresh,
  single-flight/follow-up counts, return-event coalescing, bfcache, account-switch
  races, sign-out and retry. Private responses include session identity; stale
  responses cannot populate another person's marker. No persistent browser cache.
  All 232 Rust and 15 total script tests plus workspace/Go checks pass.
- [ ] **A4. Measure complete authenticated requests — small; follows A1–A3.**
  Repeat Public, Saved, and Inquired measurements with sparse/dense histories,
  broad/selective/empty filters, and first/later pages. Include the unread
  endpoint, cover loading, saved-status batches, database pool waits, and total
  HTTP latency. Record before/after evidence and identify remaining bottlenecks.
- [ ] **A5. Bound Inquired candidate work if measurements require it —
  medium–large; follows A4.** Measure repeated latest-message checks under dense
  histories, repeated offer messages, and selective filters. Optimize only if
  remaining cost warrants it; evaluate the maintained-summary fallback already
  described in `V0.7_PLAN.md`. Preserve deduplication, membership, visibility,
  activity ordering, and cursor behavior. Record passing evidence or the
  evidence-based decision that no further change is needed.
- [ ] **A6. Consolidate inbox latest-message lookups — small–medium.** Retrieve
  preview and sender together instead of repeating latest-message lookups.
  Preserve unread semantics, empty conversations, authorization, ordering, and
  pagination. Compare query plans and verify behavior.
- [ ] **A7. Complete v0.7 verification and release evidence — medium; follows
  applicable fixes and Track B.** Pass regressions and all required load, mixed,
  spike/recovery, soak, and capacity gates. Record results in `SCORECARD.md`, then
  update release/product completion records and bump the workspace version to
  `0.7.0`. Tags remain deferred. Do not claim completion from implementation alone.

Keep initial offer pages at 20 items. Pagination already bounds returned feed
rows; it does not by itself bound database work required to find those rows.

## Track B — dedicated cumulative journey CI

Start B1 alongside A1. Build on the existing Go runner and CI release artifacts.

- [x] **B1. Inventory every supported journey — medium.** Map the current
  Offerer/Responder journeys in `docs/JOURNEY.md` to scenarios and important
  endpoints. Audit existing discovery, search/filter/pagination, save/follow,
  authorization, messaging, and unread coverage. Record missing coverage
  explicitly; do not report “all journeys passed” while supported journeys are
  absent. Future, unimplemented product stages are excluded.
  Evidence: `81f5306`,
  [inventory](crates/haven-web/load-tests/JOURNEY_COVERAGE.md), and
  [CI 40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401).
  Source audit maps 14 supported journey groups to lane, stage, endpoints,
  current scenarios and missing coverage. No full cumulative journey pass is
  claimed; contracts/scenarios/workflow/enforcement remain B2–B7.
- [x] **B2. Define journey and endpoint contracts — medium; follows B1.** For
  each supported journey, specify actions, expected results, request counts,
  fixture needs, throughput/latency requirements, and sparse/dense/empty/
  unauthorized cases where applicable. Distinguish HTTP RPS from journeys/sec.
  Use the performance contract; do not lower targets to match current results.
  Evidence: `24b02db`,
  [contracts](crates/haven-web/load-tests/JOURNEY_CONTRACTS.md), and
  [CI 40838910534](https://github.com/Phastboy/haven/actions/runs/40838910534).
  All 14 inventory groups have explicit action sequences, expected outcomes,
  primary/follow-up request counts and fixture/negative cases. The declared mix
  totals 100% of HTTP requests; endpoint, mixed, spike/recovery, soak and capacity
  requirements retain the absolute contract. Definition is complete; scenarios,
  adapters, telemetry, cumulative workflow and enforcement remain B3–B7.
- [ ] **B3. Complete reproducible fixtures and scenarios — medium–large; follows
  B2.** Fill the coverage gaps, including messaging and unread behavior. Use
  isolated databases and deterministic fixture truth. Isolate mutable sessions
  between workers/scenarios so writes do not invalidate other checks. Verify
  expected IDs/order, permissions, pagination, and state transitions.
- [ ] **B4. Add a dedicated workflow on every push — medium; follows B3.** Build
  release artifacts, validate migrations/fixtures, and run the cumulative suite
  for the pushed revision. New journeys add coverage; existing journeys remain.
  Run correctness, per-endpoint load, mixed journeys, spike/recovery, soak, and
  capacity stages. Keep measurement workloads isolated from competing test jobs.
  Clearly label stages not run after an earlier failure.
- [ ] **B5. Publish complete evidence on success and failure — medium; with
  B4.** Export a readable summary and raw reports containing commit/fixture IDs,
  hardware and sharing, duration, per-endpoint latency, offered/achieved load,
  errors, dropped arrivals, semantic sample/check counts, CPU/RSS, database pool
  waits, and capacity/recovery results. Keep session credentials out of artifacts.
  A failed or canceled run must never count as a pass.
- [ ] **B6. Enforce regressions and protect delivery — medium; follows B4–B5.**
  Compare against accepted results on comparable hardware as well as absolute
  contracts. Define explicit regression tolerances; never silently replace a
  better baseline. Capacity discovery may reach saturation, but must fail the
  gate if accepted minimum capacity/recovery requirements are missed. Require
  the journey gate before merge and deployment; verify actual repository and
  deployment enforcement, not just workflow configuration.
- [ ] **B7. Measure runtime and document the normal procedure — small; follows
  a complete run.** Record runner cost, duration, reproduction commands, failure
  triage, baseline review, and how a journey change updates coverage. Preserve
  the local lightweight CLI-only constraint; run Rust builds/integration and
  sustained performance verification in CI. Split independent jobs only when
  isolation and comparable measurements are retained.

## Scale and completion record

The current v0.7 schedule contains about 54 minutes of timed workloads before
the capacity ladder, plus build, fixture preparation, and drain time. An hour or
more per cumulative push run is a planning estimate, not a measured guarantee.
Do not shorten coverage or weaken targets silently to meet that estimate.

For every completed task, append an evidence entry here:

| Task | Commit / CI run | Evidence and result | Completed |
|---|---|---|---|
| A1 | `81f5306`, `9c50d0e` / [40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401) | Workspace CI passes; five new state/backfill/concurrency/authorization/fixture/plan regressions; 2,048-read-history partial-index assertion | 2026-10-29 |
| B1 | `81f5306` / [40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401) | 14 journey groups inventoried with explicit scenario/endpoint gaps and B2 handoff; full cumulative coverage remains pending | 2026-10-29 |
| A2 | `24b02db` / [40838910534](https://github.com/Phastboy/haven/actions/runs/40838910534) | Independent/private unread endpoint; four HTTP and four script regressions pass with workspace checks | 2026-10-29 |
| B2 | `24b02db` / [40838910534](https://github.com/Phastboy/haven/actions/runs/40838910534) | Contracts define all 14 groups, request accounting, fixtures, absolute endpoint/tier targets and 100% HTTP allocation | 2026-10-29 |
| A3 | `3d4d2c4`, `32bb582` / [40841600614](https://github.com/Phastboy/haven/actions/runs/40841600614) | Ten unread reuse/refresh/account-isolation script regressions; 232 Rust and 15 total script checks pass | 2026-10-29 |


## Continuation point — 2026-10-29

A1–A3 and B1–B2 are complete as recorded above. A3 and the first B3 tranche passed
[CI 40841600614](https://github.com/Phastboy/haven/actions/runs/40841600614) at
`32bb582`. The initial run passed Rust/script/Go checks but failed before the live
scenario because psql was not given its connection settings; the rerun fixes
that setup. No full load gate or v0.7 completion is claimed.

### B3 verified partial progress

- Required category payloads are supplied by legacy create/edit and concurrent
  idempotency journeys; stateful transport regression and Go vet pass.
- `--scenario messaging` reserves three distinct people per worker and checks a
  17-request inquiry/read/reply/unread/access sequence. Status checks verify
  private headers, session identity and state. Thread body/order and inbox
  identity are checked; outsiders cannot read/write; guests receive unread 401.
  Semantic counts are exported and conversation IDs are normalized from labels.
- Fresh-database CI ran six people/two workers/four journeys: **68/68 HTTP checks
  and 4/4 semantic round trips passed**, zero failures/drops. Artifact
  `messaging-correctness` contains the revision, fixture hash, JSON summary and
  runner report; sessions remain private. Fixture SHA256:
  `e63391f07c712d6eb651f508f61fe29e88c42811a381d3d9a1715e6558fa3732`.
  This is low-load correctness evidence, not a performance-gate result.

**B3 remains unchecked.** Next fixture/scenario work includes authenticated Public
and zero-history strata, dense/read-only and isolated mutable truth, inbox/thread
paging and SSE, remaining auth/profile/media journeys/adapters and complete pool
telemetry. The legacy write journey still needs a real-application round trip.
A4 is now unblocked by A1–A3 and must measure complete authenticated requests.
B4–B7 still need cumulative every-push coverage, complete success/failure evidence,
regression baselines, actual merge/deployment enforcement and measured procedure.


## B3 authenticated/zero-history tranche — pending CI verification

The narrowing fixture now reserves one genuine zero-history person, excluding
that person's offers from every seeded history and asserting zero conversations
and saves after generation. Dense selection first guarantees at least 6,484
distinct offer makers, then fills to 10,000 offers; generated participant counts
are checked against that minimum. Authenticated Public, Saved and Inquired each have
explicit dense/sparse/zero groups, person identity and expected unread truth.
Independent oracle regressions cover public cursor/result equivalence and empty
personal truth. Old guest-only/incomplete fixtures are rejected by the runner.

`--fixture-sweep` visits every selected fixture case exactly once with 100%
semantic checks. Authenticated cases issue the page request and then a separately
measured unread request; `journey_latencies` reports their combined duration
without counting another HTTP request. `--history` isolates a material history
stratum. Local Go checks pass; Rust and full 10k-person/100k-offer verification
run only in CI. The new opt-in `narrowing_review` publishes credential-free case,
stratum, HTTP/semantic and timing evidence through `journey-correctness.py`.

The legacy narrowing mix now splits its former 70% Public journey share into
35% guest and 35% authenticated Public, retaining 15% Saved/15% Inquired. This is
journey weighting, not the cumulative HTTP mix specified by B2. Full legacy gates
now measure all ten class/history groups separately and apply latency limits per
endpoint label, preventing zero histories or fast status calls from hiding dense
page regressions. Authenticated contract phases retain ≥1,000 page journeys/sec
and also issue ≥1,000 status requests/sec; targets were not lowered. Timed stages
therefore grow from about 40 to 54 minutes before the capacity ladder.

A4 remains pending: these are low-load correctness/measurement preparations.
Cover and saved-status database work are included in complete page-body latency;
external cover delivery, isolated pool waits, sustained before/after comparisons
and bottleneck attribution have not yet been measured. B3 remains pending for the
other fixture/scenario gaps listed above; no release gate is claimed.

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
- [ ] **A2. Independent unread loading — medium; follows A1.** Render the page
  without awaiting unread status. Fetch it through a small authenticated endpoint
  with private response handling. Keep the dot hidden while initial status is
  unknown; endpoint failure must leave the page usable. Verify page rendering
  completes independently of a delayed or failed unread request.
- [ ] **A3. Reuse and refresh unread status — small–medium; follows A2.** Reuse
  status during navigation where the runtime supports it. Refresh after sending
  or reading messages and on returning to the app. Prevent duplicate concurrent
  checks and clear person-specific state on sign-out/account changes. Verify
  refresh behavior and request counts across navigation.
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
- [ ] **B2. Define journey and endpoint contracts — medium; follows B1.** For
  each supported journey, specify actions, expected results, request counts,
  fixture needs, throughput/latency requirements, and sparse/dense/empty/
  unauthorized cases where applicable. Distinguish HTTP RPS from journeys/sec.
  Use the performance contract; do not lower targets to match current results.
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

The current v0.7 schedule contains about 40 minutes of timed workloads before
the capacity ladder, plus build, fixture preparation, and drain time. An hour or
more per cumulative push run is a planning estimate, not a measured guarantee.
Do not shorten coverage or weaken targets silently to meet that estimate.

For every completed task, append an evidence entry here:

| Task | Commit / CI run | Evidence and result | Completed |
|---|---|---|---|
| A1 | `81f5306`, `9c50d0e` / [40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401) | Workspace CI passes; five new state/backfill/concurrency/authorization/fixture/plan regressions; 2,048-read-history partial-index assertion | 2026-10-29 |
| B1 | `81f5306` / [40837141401](https://github.com/Phastboy/haven/actions/runs/40837141401) | 14 journey groups inventoried with explicit scenario/endpoint gaps and B2 handoff; full cumulative coverage remains pending | 2026-10-29 |


## Continuation point — 2026-10-29

A1 and B1 are complete as recorded above. CI initially rejected an unnecessary
raw-string delimiter in the test oracle; `9c50d0e` fixes that lint and the rerun
passes. The new state/read queries use parameterized SQLx calls; unchanged macro
queries retain their existing offline metadata.

Next: A2 independent unread loading and B2 journey/endpoint contracts. Keep the
unread dot hidden until its authenticated result is known, and make request
failure independent of page rendering. A3 must then cover refresh, coalescing and
person-specific state. Do not rerun the full release gates or mark v0.7 complete
until applicable fixes and cumulative CI requirements are verified.

The B1 audit also identifies a legacy load-workload issue: create requests omit
v0.7's required category. Fix and verify it with B3's scenario work. Public v0.7
fixture cases currently represent guests; A4 needs authenticated Public cases.


## A2/B2 implementation — pending verification

A2 removes unread I/O from the root layout. The new session-bound
`GET /unread-status` is private/no-store (including failures), varies by Cookie,
and returns 401 rather than redirecting unauthenticated API clients. A deferred
same-origin script updates hidden desktop/mobile indicators together and treats
unknown, malformed or failed responses as unavailable enhancement. Both regular
and public offer-detail pages can render independently. Four HTTP regression
tests cover blocked/failing storage, session identity and private rejection;
four browser-script tests cover delay, read state, failures and guest pages.
Local formatting, diff checks and script tests pass; Rust verification is in CI.
A3 refresh/reuse remains separate.

B2's [journey contracts](crates/haven-web/load-tests/JOURNEY_CONTRACTS.md) specify
all J01–J14 action sequences, primary/follow-up request counts, fixture and
negative-state requirements, per-endpoint targets and cumulative mix/tier rules.
Implementation, adapters, complete telemetry and enforcement remain B3–B7.
Completion evidence will be recorded after verification.

# Performance fixes and cumulative journey CI

Agreed 2026-10-28. This is the execution checklist for personal-feed performance
fixes and a dedicated journey performance workflow on every push. All tasks are
pending. Check an item only after its completion criteria are met; record the
commit, CI run/report, and relevant measurements beneath the item. Implementation
without verification remains pending. If measurement makes a conditional change
unnecessary, record that evidence and the decision before closing the item.

Product scope remains governed by the repository-root `ROADMAP.md` and
`docs/JOURNEY.md`. Targets remain governed by
[PERFORMANCE_CONTRACT.md](crates/haven-web/load-tests/PERFORMANCE_CONTRACT.md).
This checklist does not mark a product stage or release gate complete.

## Track A — prioritized fixes

- [ ] **A1. Indexed unread state — medium.** Maintain per-participant unread state
  atomically with message sends and reads; provide a consistent backfill for
  existing conversations. Preserve participant authorization and current
  latest-sender/read semantics. Verify concurrent sends/reads, empty histories,
  and fixture consistency. Query plans must demonstrate that checking for unread
  messages no longer walks every read conversation in a dense history.
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

- [ ] **B1. Inventory every supported journey — medium.** Map the current
  Offerer/Responder journeys in `docs/JOURNEY.md` to scenarios and important
  endpoints. Audit existing discovery, search/filter/pagination, save/follow,
  authorization, messaging, and unread coverage. Record missing coverage
  explicitly; do not report “all journeys passed” while supported journeys are
  absent. Future, unimplemented product stages are excluded.
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
| Pending | — | No task completion claimed | — |


## Work in progress — 2026-10-29

- **A1 implemented, pending CI verification.** Migration 019 backfills the existing
  latest-sender/read predicate and adds a partial unread-person index. Participant
  read changes and conversation activity changes refresh it within the transaction.
  Sends and reads serialize on the conversation row after membership verification;
  message timestamps are assigned after locking. Added regression tests for
  unauthorized writes, empty/latest-sender state, concurrent send/read, populated
  migration backfill, bulk fixture refresh and dense read-history query plans.
  Local formatting and diff checks are the only completed checks; no performance
  improvement or task completion is claimed yet.
- **B1 source audit drafted, pending evidence record.** See
  [journey coverage inventory](crates/haven-web/load-tests/JOURNEY_COVERAGE.md)
  for all supported lanes, endpoints, current scenarios, explicit gaps and B2
  handoff. Messaging/unread, authenticated Public, account/profile/media and
  lifecycle coverage are incomplete. The legacy write workload also omits the
  required create category. No cumulative journey pass is claimed.

# START HERE — v0.7 release-gate checkpoint

Updated: 2026-10-30.
Status: **v0.7 PR preparation; wait for owner instruction before checking reviews.**

The authorized full run is complete and its artifact has been reviewed. See
[`V07_GATE_RESULT.md`](crates/haven-web/load-tests/V07_GATE_RESULT.md) for retained
evidence, skipped stages and measurement limits. Do not dispatch a duplicate.

## Current state

- The dense Saved investigation is **finished**. Do not restart local exploratory
  replays or repeat the discarded Saved-status lookup experiment.
- `plan/v0.7-discovery` was fast-forwarded locally and remotely to
  `4c379aafb4b59b109ee11e180b438cf640ec1eb9`.
- The workspace remains checked out on `codex/a4-inquired-ci` at the same commit.
- The verified query fix is `dfb79ad`; profiling infrastructure is `4bd0073`;
  final evidence is `4c379aa`.
- [Verification CI 41045410575](https://github.com/Phastboy/haven/actions/runs/41045410575)
  passed standard checks and the complete 530-case narrowing sweep (1,049 HTTP
  checks). Dense Saved passed 25/50/75/100 journeys/s, with all 30,000 HTTP
  checks correct and no dropped arrivals across the four rates.
- At 100 journeys/s: 6,000/6,000 completed, 12,000/12,000 HTTP checks passed,
  HTTP p95 3.07 ms, worst per-case page-plus-unread journey p95 4.60 ms.
- This is verification through 100 journeys/s, **not** a release-gate pass or a
  measured capacity ceiling.

Authoritative evidence:
[`SAVED_PROFILE_RESULT.md`](crates/haven-web/load-tests/SAVED_PROFILE_RESULT.md).
Older A4 and scorecard entries are historical. Some called the runner's
`overall` HTTP-request p95 a journey p95; use `journey_latencies` for journeys.

## Next task — wait for owner direction after opening the PR

The owner authorized a v0.7 PR targeting main, then explicitly instructed us to
wait until asked before checking reviews. Do not merge, enable auto-merge, tag,
deploy, poll reviews or resume performance work. Standard PR CI may run normally.

The isolated worktree `/tmp/haven-v07-release` uses `plan/v0.7-discovery` and
packages version 0.7.0 with the documented release exception and deferred debt.
Original worktree staging is preserved. PR: [#43](https://github.com/Phastboy/haven/pull/43), targeting main.
The Public experiment branch is excluded. Release acceptance and debt remain in
V0.7_RELEASE.md and V0.7_DEBT.md. No full performance pass is claimed.

## Stateful execution record

Update this table in place as the next session progresses.

| Field | Current value |
|---|---|
| Full-gate status | Failed at `contract-public-all` |
| Tested branch | `plan/v0.7-discovery` |
| Expected starting SHA | `4c379aafb4b59b109ee11e180b438cf640ec1eb9` — verify remote before dispatch |
| Actual tested SHA | `4c379aafb4b59b109ee11e180b438cf640ec1eb9` (remote verified; contains `dfb79ad`) |
| Run ID / URL | [41072121846](https://github.com/Phastboy/haven/actions/runs/41072121846) |
| Dispatch time | 2026-10-30 14:30:27 UTC |
| Artifact location | `/tmp/haven-v07-gate-41072121846`; retained JSON in `crates/haven-web/load-tests/evidence/41072121846` |
| Correctness matrix | All 10 strata passed: 60,000 journeys; 114,000 HTTP checks; zero failures/drops |
| Contract-rate stages | Guest Public failed; remaining 9 skipped |
| Mixed traffic | Not run after contract failure |
| Spike / recovery | Not run after contract failure |
| 30-minute soak | Not run after contract failure |
| Capacity result | Not run; no capacity ceiling established |
| Release decision / blocker | Owner accepts release with documented gate exception; Public failure remains V07-D01 debt, packaging pending |

## Workspace preservation

At handoff, existing staged/unstaged changes include `ROADMAP.md`,
`docs/JOURNEY.md`, `PERFORMANCE_AND_JOURNEY_CI_TASKS.md`, `V0.7_PLAN.md`,
`A4_STATE.md`, and `SCORECARD.md`; untracked v0.8 agreement/freeze documents also
exist. Preserve their content and staging. Do not reset, stash, or include them
in a commit merely to dispatch CI. These local edits are not part of the remote
revision tested by the workflow.

The current task checklist is
[`PERFORMANCE_AND_JOURNEY_CI_TASKS.md`](PERFORMANCE_AND_JOURNEY_CI_TASKS.md).
A4/A7 and cumulative journey CI work remain open. External image transfer is
outside the server HTTP runner; a successful server gate alone does not measure
Cloudinary transfer or browser decode. Do not bump the version, tag, deploy, or
claim v0.7 complete solely from the Saved fix.

## Public diagnostic execution — authorized 2026-10-30

- Branch `codex/public-contract-profile`, commit `8a37c00`, isolated worktree
  `/tmp/haven-public-contract-profile`; existing application code unchanged.
- [CI 41076712717](https://github.com/Phastboy/haven/actions/runs/41076712717)
  dispatched with `public_profile=true`, full gate and Saved profile false.
- Fixed four stages: JIT on/off × 100/1,010 guest Public journeys/s, 30 seconds
  each, 100% semantic checks, 200 workers; 90-second subprocess deadline.
- Capture pg_stat_statements planning/execution/JIT, host and DB container
  CPU/RSS, pool acquisition. Stop after these stages, preserve failed workloads,
  review evidence before any application rewrite or gate rerun.
- Initial run failed after the first workload because `docker stats` ignored
  termination; no comparison conclusion. Artifact `/tmp/haven-public-41076712717`.
- Cleanup fixed in `391d0f4` (kill fallback); bounded retry
  [CI 41078987201](https://github.com/Phastboy/haven/actions/runs/41078987201).
- Status: complete; all four stages captured. See
  [`PUBLIC_PROFILE_RESULT.md`](crates/haven-web/load-tests/PUBLIC_PROFILE_RESULT.md).
- JIT ruled out: all statement JIT counters zero; both contract-rate variants fail.
  Price-filtered Public first/later query shapes account for ~98.6% of captured
  execution time under load; mean execution ~291–311 ms, pool p95 ~411–453 ms.
- Next bounded task: capture actual plans for free-price first/later cases and
  fast controls, distinguish scan/index/join cause, then verify one minimal fix.
  No application change or full-gate rerun yet. Existing workspace staging preserved.

## Public actual-plan capture — authorized 2026-10-30

- Diagnostic commit `e38a9a9` on `codex/public-contract-profile`.
- [CI 41084281841](https://github.com/Phastboy/haven/actions/runs/41084281841)
  captures exact guest free-price and dominant-category first/later fixture
  plans using parameterized PREPARE/EXECUTE, ANALYZE/BUFFERS/SETTINGS JSON,
  followed by the same bounded diagnostic workload. No application fix yet.
- Complete. Retained plans: `crates/haven-web/load-tests/evidence/41084281841`.
  Free-price first/later queries parallel-scan 100k offers and sort; 21–22 ms
  isolated vs 0.6–0.8 ms for ordered-index controls. Only 21 maker lookups;
  the maker join is not the excessive work in these plans.
- Candidate `ada6524`: emit `price = 0` for a zero max (equivalent under the
  database nonnegative constraint), allowing existing currency/price/feed index.
  No new index or target change. Capture original/equality plans on same fixture.
- Verification [CI 41086635067](https://github.com/Phastboy/haven/actions/runs/41086635067)
  on `ada6524`: standard checks, full narrowing sweep, two 60-second Public
  stages at 100 and 1,010 journeys/s, 100% semantics. Completed: correctness
  passed; contract load failed. Candidate excluded; see V0.7_DEBT.md.

## Owner direction — release first, maintenance deferred (2026-10-30)

The owner stopped further performance investigation and directed that performance
patches and bug fixes be deferred to the freeze phase, with proper documentation
of issues and debts so v0.7 can ship. No more diagnostic or fix runs are authorized
by the older continuation tasks above. Prepare honest release documentation; do
not represent the failed load gate or skipped stages as passed. The roadmap's
existing load-gate completion rule needs an explicit documented release exception.

Latest experiment: `ada6524` on `codex/public-contract-profile` passed all 530
narrowing cases / 1,049 HTTP checks in CI 41086635067, but did NOT fix performance:
26,977/60,600 contract journeys completed, 33,623 dropped; HTTP p95 666.02 ms.
Equality plans still parallel-scan and sort. Keep this ineffective candidate out
of the release. It is isolated from `plan/v0.7-discovery`. The proposed reversion
and extra index/statistics probe were interrupted; last confirmed branch HEAD
is `ada6524`, working tree clean except generated Python cache. No further probe
was dispatched. Artifacts: `/tmp/haven-public-41086635067` and
`/tmp/haven-public-correctness-41086635067`.

Next: reconcile release/checklist documentation with the owner's deferred-debt
policy; record known Public load failure, unrun stages, cumulative journey gaps,
and external-image measurement gap. Do not resume optimization from this file.

## Release documentation completed — 2026-10-30

V0.7_RELEASE.md and V0.7_DEBT.md now record the scoped owner exception, seven
open freeze debts, unchanged performance targets, accepted source `4c379aa`,
excluded experiment `ada6524`, immediate-triage exceptions and packaging checklist.
Root roadmap/journey, v0.7 plan, execution checklist, scorecard/result headers and
freeze rule are aligned. Historical measurements remain intact. Failed candidate
plans/results are retained under `load-tests/evidence/41086635067`. Documentation
links and whitespace checked; no application code, version, tag or deployment changed.

## PR preparation — 2026-10-30

Documentation and 0.7.0 package metadata prepared on plan/v0.7-discovery in
/tmp/haven-v07-release. PR opened: [#43](https://github.com/Phastboy/haven/pull/43), targeting main.
Release preparation commit: `09dfd6d`. Stop until the owner requests review inspection. No merge or deployment is authorized here.

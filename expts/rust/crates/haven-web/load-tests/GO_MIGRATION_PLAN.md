# Go Load-Test Migration and Pending Gates

**Status:** Go runner and feature journeys implemented; workload measurements remain pending.

**Branch:** `plan/go-load-tests-v05-v06-trivy`

This work replaces executable JavaScript load scenarios with a Go-native runner and adds
explicit end-to-end journeys for the v0.5.x and v0.6.x release gates. No load test has been
run against a seeded Haven instance in this implementation session. Do not close either
roadmap gate until an isolated run is recorded in `SCORECARD.md`.

## Implementation

- `go/cmd/haven-load` uses Go's standard `net/http`, an open-arrival scheduler, bounded
  workers, route-level latency samples, and JSON summaries with p50/p95/p99.
- `smoke`, `warmup`, `steady`, `authorization`, `v05`, and `v06` are Go scenarios.
- `v05` checks combined title/currency/price filters, preservation of filter parameters on
  pagination, free-offer filtering, and the filtered empty state.
- `v06` uses isolated actor/target groups to check save/follow idempotency and both saved and
  followed-person pagination, then removes the created relationships.
- `load-test-support` emits handles and public offer slugs alongside its owner offer IDs, and
  includes free offers for the v0.5 filter journey.
- QEMU orchestration and analysis consume the Go runner's JSON summary. The JavaScript
  scenario source files have been removed.

The fixture generator refuses non-local database hosts. Use a disposable local database;
the generated session file contains active credentials and must remain private.

## Remaining validation

1. Run `go test ./...` and `go vet ./...`; these validate the runner without generating
   traffic.
2. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features --locked`,
   and `cargo test --workspace --locked` for the fixture generator and workspace.
3. Generate a fixture in a disposable local database, start Haven against that same database,
   then run `smoke`, `authorization`, `v05`, and `v06` at low rates. Confirm state cleanup and
   the two pagination links before any sustained benchmark.
4. Run the approved normal, high-load, soak, spike, and capacity profiles in an isolated
   environment. Capture per-route p95/p99, throughput, errors, dropped journeys, CPU, RSS,
   and database connections in `SCORECARD.md`.
5. Keep v0.5.x and v0.6.x load gates open until those feature workloads pass their agreed
   criteria and have recorded measurements.

## Trivy finding

The supplied GitHub Actions output reports Debian `libssl3` 3.0.20-1~deb12u2 with HIGH
CVE-2026-84782 and identifies 3.0.22-1~deb12u2 as fixed. The final runtime base was moved
from Distroless Debian 12 to Distroless Debian 13 in a separate commit. No image was pulled
or built locally. The resolution is provisional until GitHub Actions scans the built image
and its output confirms the finding is gone; if not, use the reported package/version data
to choose the next runtime change. Keep the existing HIGH/CRITICAL gate and do not suppress
the CVE.

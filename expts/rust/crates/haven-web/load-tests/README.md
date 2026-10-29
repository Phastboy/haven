# Haven Load Tests

Load scenarios are implemented in Go under `go/`; no k6 or JavaScript runner is required.
The runner uses an open-arrival schedule, bounded workers, seeded sessions, and emits
machine-readable summaries with per-route p50, p95, and p99.

## Scenarios

- `smoke`: `GET /health`.
- `warmup`: warms the sign-in and authenticated offer-feed routes before measurement.
- `steady`: one-request mix of public feed, owner offer, landing, and public profile views.
- `authorization`: verifies that one signed-in person cannot view another person's owner
  offer-management page.
- `v05`: title search with currency/price filters, filtered pagination, Free results, and
  the filtered empty state.
- `v06`: idempotent save/follow and unsave/unfollow, plus pagination through collections of
  21 offers and followed people. Each worker needs an isolated group of 22 seeded users.
- `spike`: three 60-second open-arrival stages at 100, 2,000, and 100 journeys per second;
  the burst and recovery stages enforce their contract latency and error thresholds.
- `capacity`: the contract's ascending two-minute rate ladder, stopping when p95 exceeds
  250 ms or request errors exceed 1%.

## Prepare fixtures

Set `DATABASE_URL` to a disposable local test database, then generate sessions and offers:

```sh
cargo run -p load-test-support -- --users 10000
```

The fixture contains session tokens, handles, owner offer IDs, and public offer slugs. It is
ignored by git and should only be generated for an isolated test database. Keep the fixture
private because it contains usable session credentials.

## Run

Start Haven against the same database, then run the suite:

```sh
./run-all.sh
```

Run a single scenario by building the binary and specifying its load parameters:

```sh
(cd go && go build -o ../results/haven-load ./cmd/haven-load)
./results/haven-load --base-url http://127.0.0.1:3000 \
  --scenario v05 --rate 10 --duration 60s --workers 16 \
  --sessions data/sessions.json --summary results/v05.json

# Long contract profiles are run separately from run-all.sh.
./results/haven-load --base-url http://127.0.0.1:3000 \
  --scenario spike --workers 100 --sessions data/sessions.json \
  --summary results/spike.json
```

`--rate` is scheduled journeys per second; the summary also reports actual HTTP requests
per second. `--workers` bounds concurrency. For `v06`, the fixture must contain at least
22 users per worker (for example, use `--users 352` with 16 workers). The default 10,000
user fixture is intended for the QEMU benchmark and large pagination tests.

The Go tests run independently of Haven:

```sh
(cd go && go test ./... && go vet ./...)
```

## Results and safety

Summaries and logs are written to the ignored `results/` directory. Run against an isolated
database and never use production session fixtures. The TLS verification bypass option is
reserved for the QEMU harness's self-signed endpoint.

For a step-by-step database, fixture, and manual run procedure, see
[`LOAD_TEST_RUNBOOK.md`](LOAD_TEST_RUNBOOK.md). The runner prints and writes per-route p95/p99
automatically; updating the tracked scorecard remains a manual review step.

Do not mark the v0.5.x or v0.6.x roadmap gates complete until the agreed performance
criteria have recorded results in `SCORECARD.md`.


### Messaging correctness (B3, initial coverage)

The Go `messaging` scenario reserves three distinct people per worker: responder,
maker with a visible offer, and outsider. Generate a fresh ordinary fixture on a
new migrated disposable database (`load-test-support --users 6` for two workers).
Do not share these people with any concurrent mutation workload. Their unrelated
conversations must initially be read/empty. Fixture identities are generated;
assertions use the exported session/offer truth, and CI records its private-file
SHA256 rather than exposing credentials.

```sh
./results/haven-load --base-url http://127.0.0.1:3000 --scenario messaging \
  --sessions data/sessions.json --workers 2 --rate 2 --iterations 4 --duration 5s \
  --summary results/messaging.json
```

Each journey issues 17 explicit HTTP requests: inquiry, unread transitions,
responder/maker thread reads, inbox identity, maker reply, outsider read/write
404s, unchanged authorized thread and guest unread 401. Status calls are separate
requests, with their own latency, private-header and person/state assertions.
Report `semantic_cases.messaging/round-trip` checked/passed counts alongside HTTP
outcomes. This scenario only covers part of J06–J08/J14; SSE, message/inbox paging,
full fixture strata and all other missing inventory rows remain pending.

CI runs `journey-correctness.py` after builds, assets, and Go checks. It creates
`haven_messaging_ci` on the local disposable PostgreSQL service, starts the debug
application, seeds six people, and requires four passing journeys/68 requests.
The `messaging-correctness` artifact contains only result/summary/runner reports.
Sessions and server/seed logs remain in a separate private directory. Results
are low-load correctness evidence, not release performance measurements.


### Expanded narrowing correctness and timings

Regenerate old narrowing fixtures. The runner requires guest Public plus explicit
`dense`, `sparse` and `zero` strata for `public-authenticated`, `saved` and
`inquired`. Authenticated cases carry private person/unread truth. The reserved
zero-history person is excluded from other seeded actors' histories and verified
to have no conversations or saved offers.

```sh
./results/haven-load --base-url http://127.0.0.1:3000 --scenario v07 \
  --narrowing-fixture data/cases.json --query-class mixed --fixture-sweep \
  --semantic-percent 100 --summary results/narrowing-correctness.json

./results/haven-load --base-url http://127.0.0.1:3000 --scenario v07 \
  --narrowing-fixture data/cases.json --query-class public-authenticated \
  --history dense --rate 100 --duration 60s --semantic-percent 100 \
  --summary results/public-authenticated-dense.json
```

A sweep is sequential correctness work and reports `target_journeys_per_second=0`;
it visits every selected case once, rather than hoping a timed sample reaches it.
It is not a throughput gate. Each authenticated journey explicitly fetches the
page and its private unread status. HTTP `routes` separate page/status latency;
`journey_latencies` measures their combined wall time without inflating HTTP
counts. The page body includes cover loading and saved-status database batches;
this HTTP client does not fetch external cover images or isolate DB pool waits.

Dispatch `haven-rust-ci.yml` with `narrowing_review=true` for the full fresh
10k-person/100k-offer release-binary sweep. Artifact `narrowing-correctness` contains
only revision, fixture hash, stratum counts, case checks, HTTP summaries and
runner timing reports. Private fixture/session data remain outside the artifact.
The legacy full load schedule separately gates guest Public and each authenticated
class/history combination; adding empty histories must not dilute dense work.

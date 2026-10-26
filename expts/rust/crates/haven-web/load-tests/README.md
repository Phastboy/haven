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

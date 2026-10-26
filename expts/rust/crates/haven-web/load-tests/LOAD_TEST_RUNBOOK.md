# Manual Go Load-Test Runbook

This runbook prepares an isolated local environment and explains how to inspect results.
It does not run tests or alter the existing ignored `data/sessions.json` fixture.

## Prepare an isolated database and fixture

Use a disposable PostgreSQL database on `localhost`, `127.0.0.1`, or `::1`. The fixture
generator rejects remote database hosts, but it cannot determine whether a local database
contains valuable data, so point it only at a database created for this run.

Create a new local database with your PostgreSQL role, then from `expts/rust`, set
`DATABASE_URL` to it and start Haven once. Haven applies SQLx migrations during startup. Keep
Haven running while generating the fixture:

```sh
export HAVEN_LOAD_TEST_DB=haven_load_test
createdb "$HAVEN_LOAD_TEST_DB"
export DATABASE_URL="postgres://<local-role>:<password>@127.0.0.1:5432/$HAVEN_LOAD_TEST_DB"
cargo run -p haven-web
```

In another terminal at `expts/rust`, use the same `DATABASE_URL` and generate a fresh fixture
to a separate ignored file. Sixteen v0.6 workers need 352 people: one actor plus 21 targets
per worker. This also provides enough matching offers for v0.5 filtered pagination.

```sh
export HAVEN_LOAD_TEST_DB=haven_load_test
export DATABASE_URL="postgres://<local-role>:<password>@127.0.0.1:5432/$HAVEN_LOAD_TEST_DB"
cargo run -p load-test-support -- --users 352 \
  --output crates/haven-web/load-tests/data/go-sessions.json
```

The generator creates accounts, sessions, offers, and offer image records in that database.
The generated session file contains active credentials and is written with owner-only
permissions. Do not use a production or shared development database, publish the fixture, or
commit it. The existing 100-person fixture is left untouched because it predates the handles
and offer slugs the Go scenarios need.

## Build and run feature journeys

From `expts/rust`:

```sh
mkdir -p crates/haven-web/load-tests/results
(cd crates/haven-web/load-tests/go && \
  go build -o ../results/haven-load ./cmd/haven-load)
```

Run a single request smoke check first, then the pending v0.5 and v0.6 journeys:

```sh
cd crates/haven-web/load-tests
./results/haven-load --base-url http://127.0.0.1:3000 \
  --scenario smoke --iterations 1 --summary results/smoke.json

./results/haven-load --base-url http://127.0.0.1:3000 \
  --scenario v05 --rate 10 --duration 60s --workers 1 \
  --summary results/v05.json

./results/haven-load --base-url http://127.0.0.1:3000 \
  --scenario v06 --rate 1 --duration 60s --workers 16 \
  --sessions data/go-sessions.json --summary results/v06.json
```

The `v06` journey uses isolated groups of 22 people per worker, checks idempotent save and
follow actions and both collection pagination links, then removes its saved/followed
relationships. Run against the same database Haven and the fixture generator use.

For the full local suite, set the fixture path and worker count before running `./run-all.sh`:

```sh
SESSIONS_FILE="$PWD/data/go-sessions.json" WORKERS=16 ./run-all.sh
```

Stress, soak, spike, and capacity profiles are separate long-running runs. Their commands and
contract thresholds are in [`PERFORMANCE_CONTRACT.md`](PERFORMANCE_CONTRACT.md). Do not run
those profiles on a shared or production service.

## Read p95 and p99

The runner reports per-route p50, p95, and p99 in the terminal. It also writes the same
measurements to the `routes` array in the JSON file passed with `--summary`; values are
nanoseconds (`p95_ns`, `p99_ns`). For example, from the load-test directory:

```sh
jq -r '.routes[] | [.route, (.p95_ns / 1000000), (.p99_ns / 1000000)] | @tsv' \
  results/v06.json
```

For spike and capacity profiles, each stage has its own summary under `stages`; inspect those
to distinguish burst/recovery or capacity-step latency:

```sh
jq -r '.stages[] | .journeys_per_second as $rate |
  .summary.routes[] | [$rate, .route, (.p95_ns / 1000000), (.p99_ns / 1000000)] | @tsv' \
  results/capacity.json
```

The summary also records actual HTTP requests per second, request checks, failures, and
dropped journeys. The runner does **not** update `SCORECARD.md` automatically. After reviewing
the environment and results, manually record the run's date, commit, target and actual
throughput, p95, p99, errors, and relevant resource measurements there. Keep the JSON and
monitor output in the ignored `results/` directory as supporting evidence.

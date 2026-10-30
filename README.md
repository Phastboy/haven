# Haven

Haven is an identity-centered trade platform built around offers and the people
behind them. The supported implementation is Rust with Topcoat server-rendered
views and PostgreSQL, in [`expts/rust`](expts/rust).

- [`ROADMAP.md`](ROADMAP.md): product scope and release decisions.
- [`docs/JOURNEY.md`](docs/JOURNEY.md): Offerer and Responder journeys.
- [`ENGINEERING.md`](ENGINEERING.md): engineering principles.
- [`v0.7 release record`](expts/rust/V0.7_RELEASE.md) and
  [`deferred debt`](expts/rust/V0.7_DEBT.md): acceptance and known limitations.

The TypeScript/Bun API and Astro frontend have been retired. See
[retirement scope](docs/TYPESCRIPT_RETIREMENT.md); their source remains in Git
history. Bun, npm and the former TypeScript workspace are not required.

## Rust workspace

| Crate | Responsibility |
|---|---|
| `haven-domain` | Domain types, rules and ports |
| `haven-db` | PostgreSQL repositories and migrations |
| `haven-web` | Topcoat SSR application and browser enhancements |
| `load-test-support` | Reproducible synthetic fixtures |

The workspace pins Rust in `expts/rust/rust-toolchain.toml`. Use the Topcoat CLI
version matched by the workspace Dockerfile and CI (currently 0.11.0). Go powers
the HTTP load runner; Node runs the retained browser-enhancement tests.

## Local database and application

From the repository root, start the retained development database:

```sh
docker compose up -d db
```

The database listens on `127.0.0.1:5433`. The existing `postgres_data` volume and
credentials are unchanged. The init script creates `haven_rust` only on a fresh
volume. To ensure that database exists on an older volume without removing data:

```sh
docker compose exec -T db psql -U user -d mydb -v ON_ERROR_STOP=1 -f /docker-entrypoint-initdb.d/01-create-databases.sql
```

Configure and run from the Rust workspace:

```sh
cd expts/rust
cp .env.example .env  # first setup only; preserve an existing .env
# Adjust .env for your environment.
cargo run -p haven-web
```

The application loads the workspace `.env` and applies its SQLx migrations at
startup. The example configuration uses `http://localhost:3000`. Cloudinary and
Google credentials are configured in that example; preserve real credentials
locally and never commit them. Database Compose provides only PostgreSQL; it does
not run the application or a reverse proxy.

## Verification and delivery

Rust builds, database integration checks and sustained performance verification
run in GitHub Actions. The workflows under `.github/workflows/haven-rust-*`
provide CI, security checks, container smoke tests and tagged-release delivery.
See [`CICD_PLAN.md`](expts/rust/CICD_PLAN.md) for details.

The browser scripts remain part of the Rust application and can be checked with:

```sh
node --test expts/rust/crates/haven-web/static/*.test.cjs
```

Build the Rust container using the workspace as its context:

```sh
docker build -t haven-rust -f expts/rust/Dockerfile expts/rust
```

Load-test procedures and measurement limits are in the
[runner documentation](expts/rust/crates/haven-web/load-tests/README.md).
The v0.7 performance exception and freeze-phase debts remain in force; retirement
of the old stack does not resolve them.

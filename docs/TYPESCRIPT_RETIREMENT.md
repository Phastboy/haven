# TypeScript stack retirement

Rust is Haven's sole supported application implementation. The former Elysia/Bun
API and Astro/Solid frontend are removed from the active tree. Their source,
Drizzle migrations, tests and performance history remain recoverable from Git
history, including the `v0.7.0` tag before this retirement.

## Removed

- `apps/api` and `apps/web`, including their implementation-specific agent skills.
- Root Bun workspace manifest/lockfile, TypeScript and Oxlint configuration, and
  the Husky hook that ran Bun checks.
- TypeScript Dockerfiles and the root Compose migrate/API/web/nginx services.
- The old root nginx configuration, TypeScript environment example, root Docker
  ignore file and obsolete generated-Prisma attributes.

## Retained

- Rust crates, migrations, assets, JavaScript enhancements and Node tests.
- Go HTTP load runner, its fixtures/contracts/results, and benchmark guest nginx.
- Rust CI/CD, security workflows and Cargo/Docker/Actions dependency updates.
  Rust CI now also validates the retained database Compose file and runs when its
  configuration or init script changes.
- Product, design, journey, route and engineering documents that apply to Rust.
- The Compose `db` service, `haven_rust` initialization script, network and
  `postgres_data` volume with their existing names, credentials and PostgreSQL
  version. Existing volumes and live databases are not modified or migrated by
  this source cleanup. Compose application services are no longer provided.

The root README now describes the Rust workspace and retained database setup.
The Rust environment example remains under `expts/rust/.env.example`.
Real local environment files and ignored build/dependency directories are not
part of this tracked-file removal; no containers or external deployments are
stopped, and no persistent data is deleted.

The old TypeScript schema is not a migration path into the Rust schema. Any
future transfer from an old database requires separate explicit data-migration
work. No old application deployment is verified or changed by this PR.

This is repository retirement, not a new feature or performance correction.
The v0.7 release exception and open freeze-phase debt remain unchanged.

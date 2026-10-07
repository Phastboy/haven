# CI/CD Plan: Haven Rust (`expts/rust`)

Status: **Finalized plan, pending implementation.**  
Location: `expts/rust/CICD_PLAN.md` (beside `ENGINEERING.md`).

---

## 1. Goals & Non-Goals

### Goals
1. **Zero-Warning CI Gate**: Strict formatting (`cargo fmt`), linting (`cargo clippy` with `-D warnings`), builds, and tests on every pull request and push.
2. **Proven Builds**: Every PR verifies compilation with locked dependencies (`--locked`) and reproducible toolchains.
3. **Smallest Practical Container**: Containerize `haven-web` using a multi-stage `cargo-chef` build, pushed to GitHub Container Registry (GHCR).
4. **Automated Continuous Deployment**: Automated deployment of `main` commits to Koyeb using Koyeb's hosted PostgreSQL database.
5. **Traceability & Rollback**: Every deployment is pinned to an immutable tag (`sha-<commit-sha>`) with simple one-click rollback.

### Non-Goals
- Running Docker locally (local development continues via native `cargo`).
- Running the full 1,000+ RPS capacity and benchmark campaign in CI (shared CI runners are too noisy; QEMU load tests remain manual/dedicated).

---

## 2. Monorepo Architecture & Paths

Haven is structured as a monorepo rooted at `/` with Rust services isolated under `expts/rust/`:

```
.github/workflows/          # Central workflow definitions
  haven-rust-ci.yml         # CI checks on PR / push (fmt, clippy, test, doc)
  haven-rust-security.yml   # Supply-chain & vulnerability auditing
  haven-rust-docker.yml     # Container build, Trivy scan, smoke test, GHCR push
  haven-rust-deploy.yml     # Koyeb deployment orchestration
expts/rust/
  CICD_PLAN.md              # This plan
  ENGINEERING.md            # Architecture & engineering constraints
  rust-toolchain.toml       # Pinned Rust toolchain (1.99.0, edition 2024)
  deny.toml                 # Cargo deny policy
  Dockerfile                # Multi-stage container recipe
  .dockerignore             # Excludes target/, .git/, etc.
```

All GitHub Actions jobs targeting Rust will configure:
```yaml
defaults:
  run:
    working-directory: expts/rust
```
and filter execution using:
```yaml
on:
  pull_request:
    paths:
      - 'expts/rust/**'
      - '.github/workflows/haven-rust-*'
  push:
    paths:
      - 'expts/rust/**'
      - '.github/workflows/haven-rust-*'
```

---

## 3. Pipeline Stages

```
PR / Push (expts/rust/**)
   │
   ├──> Stage 1: CI (Format -> Clippy -> Test -> Doctest -> Doc)
   │
   └──> Stage 2: Security (cargo-deny + Dependabot)
          │
          ▼ (main branch & v* tags only)
        Stage 3 & 4: Docker Build & Verification (cargo-chef -> Trivy -> k6 Smoke)
          │
          ▼
        Stage 5: Publish (ghcr.io/<owner>/haven-rust:sha-<SHA>)
          │
          ▼ (main branch only, environment: production)
        Stage 6: Deploy to Koyeb
```

---

### Stage 1: Continuous Integration (CI)

Triggered on every PR and push affecting `expts/rust/**`.

| Step | Command | Details |
|---|---|---|
| **Toolchain** | `dtolnay/rust-toolchain@master` | Pins version `1.99.0` with `clippy` and `rustfmt`. |
| **Cache** | `Swatinem/rust-cache@v2` | Keyed to `expts/rust/Cargo.lock`. |
| **Format** | `cargo fmt --all --check` | Enforces standard formatting. *(Pre-req: run `cargo fmt --all` once).* |
| **Lint** | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Denies all warnings. *(Note: Test modules must allow `unwrap_used`).* |
| **Build** | `cargo build --workspace --locked` | Ensures all workspace crates compile cleanly. |
| **Database** | Service container: `postgres:17-alpine` | Spun up for integration tests with `DATABASE_URL`. |
| **Tests** | `cargo test --workspace --locked` | Runs unit and integration test suites. |
| **Doctests** | `cargo test --doc --workspace --locked` | Validates documentation examples against strict lints. |
| **Docs** | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items` | Enforces zero broken links and complete doc syntax. |

---

### Stage 2: Supply-Chain Security

- **`cargo deny check`**:
  - Enforces `deny.toml` covering licenses (MIT, Apache-2.0, BSD-3-Clause, etc.), security advisories, bans, and duplicate crate versions (`cargo tree -d`).
  - Runs on pull requests and on a weekly schedule.
- **Dependabot**:
  - Configured in `.github/dependabot.yml` targeting `cargo` dependencies under `expts/rust`, plus GitHub Actions and Docker.

---

### Stage 3: Docker Image Construction

#### Base Image & Allocator Strategy
* **Decision**: `gcr.io/distroless/cc-debian12` (glibc runtime) with fallback to static `scratch` + `musl`.
* **Rationale**: 
  - The performance contract and baseline measurements are recorded on a Debian glibc environment.
  - Musl's default allocator exhibits high latency jitter under concurrent lock contention, requiring `mimalloc` as a global allocator override.
  - Distroless produces a minimal ~25 MB image, provides full glibc performance parity with zero unnecessary tools/shells, and natively works with `ring` and `rustls`.

#### Multi-Stage `Dockerfile` Shape
1. **Planner (`cargo-chef`)**:
   ```dockerfile
   FROM lukemathwalker/cargo-chef:latest-rust-1.99-bookworm AS planner
   WORKDIR /app
   COPY . .
   RUN cargo chef prepare --recipe-path recipe.json
   ```
2. **Builder**:
   ```dockerfile
   FROM lukemathwalker/cargo-chef:latest-rust-1.99-bookworm AS builder
   WORKDIR /app
   COPY --from=planner /app/recipe.json recipe.json
   RUN cargo chef cook --release --recipe-path recipe.json
   COPY . .
   ENV SQLX_OFFLINE=true
   RUN cargo build --release --locked -p haven-web
   ```
3. **Runtime**:
   ```dockerfile
   FROM gcr.io/distroless/cc-debian12:nonroot
   WORKDIR /app
   COPY --from=builder /app/target/release/haven-web /app/haven-web
   ENV HOST=0.0.0.0
   ENV PORT=8000
   EXPOSE 8000
   ENTRYPOINT ["/app/haven-web"]
   ```

#### Application Runtime Constraints
* **`HOST` Binding**: Topcoat defaults `HOST` to `127.0.0.1`. The container **must** set `ENV HOST=0.0.0.0` so container ingress traffic is accepted.
* **`SQLX_OFFLINE=true`**: Because `haven-db` relies on `sqlx::query!`, the builder stage must copy `.sqlx/` and build with `SQLX_OFFLINE=true`.
* **Release Profile**: Already tuned in `Cargo.toml` (`lto = "fat"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`).

---

### Stage 4: Image Verification

1. **Vulnerability Scan**:
   - Aquasecurity Trivy action scans the built image for High and Critical vulnerabilities.
2. **Container Smoke Test**:
   - In CI, spins up the built image alongside a Postgres service container.
   - Pings `GET http://127.0.0.1:8000/health`.
   - Runs `01-smoke.js` via k6 to verify authentication, session issuance, and basic CRUD flows before publishing.

---

### Stage 5: Publish to GHCR

- **Triggers**: Pushes to `main`, and semantic version tags (`v*`). Never from PRs.
- **Permissions**: `packages: write`, `contents: read`.
- **Tags**:
  - `ghcr.io/<owner>/haven-rust:sha-<SHA>` (immutable deployment target).
  - `ghcr.io/<owner>/haven-rust:latest` (convenience only; never deployed directly).
  - `ghcr.io/<owner>/haven-rust:vX.Y.Z` (on git releases).
- **Visibility**: Set package to Public (or configure a Koyeb registry secret with a Personal Access Token if kept Private).

---

### Stage 6: Deployment to Koyeb

- **Trigger**: Runs only after Stage 5 succeeds on `main`.
- **Environment**: GitHub Environment `production` (enables deployment protection and optional approval gates).
- **Deployment Mechanism**:
  - Koyeb CLI or GitHub Action deploying image `ghcr.io/<owner>/haven-rust:sha-<SHA>`.
  - Authenticated via `KOYEB_TOKEN` secret.
- **Health Checks & Routing**:
  - Koyeb HTTP health check targets `GET /health` on the configured `PORT`.
- **Database & Migrations**:
  - Connected via `DATABASE_URL` (stored as a Koyeb Secret).
  - `haven-web` automatically executes `sqlx::migrate!("./migrations")` on startup, ensuring zero-touch schema convergence on deploy.
- **Rollback**:
  - Re-deploying an earlier `sha-<SHA>` tag through Koyeb CLI or Git commit revert.

---

## 4. Resolution of Open Decisions

| # | Question | Decision | Rationale |
|---|---|---|---|
| **1** | Does `haven-db` use SQLx `query!`? | **Yes** | Extensive macro usage. Requires `.sqlx/` in Git and `SQLX_OFFLINE=true` in container builds. |
| **2** | GHCR package visibility | **Public** (recommended) | Avoids managing rotating personal access tokens inside Koyeb registry secrets. |
| **3** | Allocator & Libc | **glibc + distroless** | Guarantees identical allocator and performance characteristics to the Debian QEMU capacity baseline. |
| **4** | `panic = "abort"` | **Active** | Already set in `[profile.release]` and aligned with workspace-wide panic-free error propagation. |
| **5** | Migration strategy | **On startup** | Add `sqlx::migrate!()` in `haven-web/src/main.rs`. Idempotent and eliminates external migration orchestrators. |
| **6** | Manual approval | **Optional** | Configured via GitHub `production` environment rule without code changes. |

---

## 5. Step-by-Step Implementation Roadmap

1. [x] **Workspace Prerequisites**:
   - Create [`rust-toolchain.toml`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/rust-toolchain.toml) pinning `1.99.0` with `clippy` and `rustfmt`.
   - Run `cargo fmt --all` across the workspace to clear existing formatting differences.
   - Adjust `#[cfg(test)]` modules in domain/db to allow `unwrap_used` in test suites so `--all-targets` clippy passes.
2. [x] **Application Endpoints**:
   - Add `GET /health` endpoint in `haven-web`.
   - Add `sqlx::migrate!("../../migrations").run(&pool).await?;` in `haven-web/src/main.rs`.
3. [x] **CI Workflows**:
   - Create `.github/workflows/haven-rust-ci.yml` (formatting, clippy, test with postgres service, doctest, doc check).
   - Create `expts/rust/deny.toml` and `.github/workflows/haven-rust-security.yml`.
4. [x] **Dockerization**:
   - Create `expts/rust/Dockerfile` and `expts/rust/.dockerignore`.
   - Verify local container build with `SQLX_OFFLINE=true`.
5. [ ] **Publish & Deploy**:
   - Create `.github/workflows/haven-rust-cd.yml` (build, scan, smoke test, GHCR push, Koyeb deployment).

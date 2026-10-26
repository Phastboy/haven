# Haven QEMU Load-Testing Infrastructure

This directory contains the isolated, reproducible load-testing infrastructure for Haven. The database and app run inside a tightly constrained QEMU VM (1 vCPU, 2 GB RAM); the Go load generator runs on the host to avoid competing with the app for CPU.

## Architecture

The benchmark relies on strict CPU topology partitioning and automated orchestration to guarantee valid metrics:

1. **Host CPU 0 (Housekeeping & I/O)**: Handles the host OS, orchestrator scripts, QEMU's `vhost` net threads, QEMU I/O helpers, and device interrupts.
2. **Host CPU 1 (Guest vCPU)**: Exclusively pinned to the single QEMU vCPU thread running the Debian guest. Host scheduling and interrupts are pushed away to ensure zero context-switching for the guest.
3. **Host CPU 2 (Load Generator)**: Exclusively pinned to the `haven-load` Go process so the generator never steals cycles from the guest.

## Scripts & Tools

### 1. Provisioning & Setup
* `dry-run.sh`: Boots the QEMU image manually for testing and debugging.
* `guest-provision.sh` / `host-prepare.sh` / `host-root.sh`: Bootstrap the Debian 13 environment with PostgreSQL and haven-web securely configured for 1vCPU / 2GB RAM.

### 2. Isolation & Architecture
* `pin.sh`: The core isolation enforcer. Moves IRQs, offlines SMT siblings, and pins all threads exactly to their designated topology (`vcpu=1 helpers/vhost=0 host=0 load generator=2`). Disables swap.

### 3. Auditing & Metrics
* `telemetry.sh`: Orchestrates the host and guest samplers over an SSH multiplexed connection.
* `host-sampler.sh`: Low-overhead Python script measuring host KVM ticks and Core busy %.
* `guest-sampler.sh`: Reads cgroup CPU limits and PSI (Pressure Stall Information) inside the guest dynamically.

### 4. Orchestration & Load Generation
* `standard-run.sh`: The main benchmark orchestrator. Handles preflight checks, database snapshots/restores, warm-up criteria monitoring (99% cache hit ratio), running the Go scenarios, and post-run cleanup.
* `run-campaign.sh`: Wraps `standard-run.sh` to run the actual tests across different parameters.
* `analyze-run.py`: Validates that the run did not cross invalidation boundaries (for example, load-generator core saturation > 70%, host core > 90%, dropped journeys, or device interrupts on CPU 1).

## Baseline Results

Using this infrastructure, we established a mathematically sound baseline for Haven on **1vCPU and 2GB RAM**:

* **Maximum Iteration Rate**: 630 iterations/sec
* **HTTP Request Throughput**: ~750 requests/sec
* **Latency SLO**: Maintained p99 < 500ms and p95 < 200ms across all read and write endpoints.
* **Historical failure boundary**: The earlier Node.js run at 650 iterations/sec (~770 req/s) breached the 500ms p99 threshold on write endpoints. Treat this as historical context, not a Go-runner result.

## Running a Test

Install Go on the host, prepare the disposable database and the generated session fixture, and ensure pinning is configured. Then run the campaign or a single Go scenario:

```bash
sudo ./pin.sh apply
./standard-run.sh --scenario steady --pool 4 --rate 630 --target https://10.10.0.2 --label my-run
```

The `v05` and `v06` scenarios cover the pending roadmap gates. `v06` needs at least 22
seeded users per worker to keep each worker's collection state isolated. For example, a
four-worker run needs 88 or more users. The script uses the explicit self-signed TLS bypass
only for this isolated QEMU endpoint; do not use it for production targets.

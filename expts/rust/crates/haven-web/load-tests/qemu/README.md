# Haven QEMU Load-Testing Infrastructure

This directory contains the completely isolated, reproducible load testing infrastructure for Haven. By running the database and application inside a tightly constrained QEMU VM (1vCPU, 2GB RAM) and the load generator (`k6`) on the host, we guarantee mathematically sound measurements for capacity baselines without "noisy neighbor" interference.

## Architecture

The benchmark relies on strict CPU topology partitioning and automated orchestration to guarantee valid metrics:

1. **Host CPU 0 (Housekeeping & I/O)**: Handles the host OS, orchestrator scripts, QEMU's `vhost` net threads, QEMU I/O helpers, and device interrupts.
2. **Host CPU 1 (Guest vCPU)**: Exclusively pinned to the single QEMU vCPU thread running the Debian guest. Host scheduling and interrupts are pushed away to ensure zero context-switching for the guest.
3. **Host CPU 2 (Load Generator)**: Exclusively pinned to the `k6` process to ensure the load generator never starves for CPU and never steals cycles from the guest.

## Scripts & Tools

### 1. Provisioning & Setup
* `dry-run.sh`: Boots the QEMU image manually for testing and debugging.
* `guest-provision.sh` / `host-prepare.sh` / `host-root.sh`: Bootstrap the Debian 13 environment with PostgreSQL and haven-web securely configured for 1vCPU / 2GB RAM.

### 2. Isolation & Architecture
* `pin.sh`: The core isolation enforcer. Moves IRQs, offlines SMT siblings, and pins all threads exactly to their designated topology (`vcpu=1 helpers/vhost=0 host=0 k6=2`). Disables swap.

### 3. Auditing & Metrics
* `telemetry.sh`: Orchestrates the host and guest samplers over an SSH multiplexed connection.
* `host-sampler.sh`: Low-overhead Python script measuring host KVM ticks and Core busy %.
* `guest-sampler.sh`: Reads cgroup CPU limits and PSI (Pressure Stall Information) inside the guest dynamically.

### 4. Orchestration & Load Generation
* `standard-run.sh`: The main benchmark orchestrator. Handles preflight checks, database snapshots/restores, warm-up criteria monitoring (99% cache hit ratio), running `k6`, and post-run cleanup.
* `run-campaign.sh`: Wraps `standard-run.sh` to run the actual tests across different parameters.
* `analyze-run.py`: Analyzes the outputs of a run to ensure it didn't cross invalidation boundaries (e.g. k6 core saturation > 70%, host core > 90%, or non-flat device interrupts on CPU 1).

## Baseline Results

Using this infrastructure, we established a mathematically sound baseline for Haven on **1vCPU and 2GB RAM**:

* **Maximum Iteration Rate**: 630 iterations/sec
* **HTTP Request Throughput**: ~750 requests/sec
* **Latency SLO**: Maintained p99 < 500ms and p95 < 200ms across all read and write endpoints.
* **Failure Boundaries**: Pushing to 650 iterations/sec (~770 req/s) caused `POST` endpoints (edit/delete) to breach the 500ms p99 SLA limit (k6 `p(99)<=500` threshold failed).

## Running a Test

First, ensure pinning is permanently passwordless via `sudo visudo`. Then simply run the campaign or a single test:

```bash
sudo ./pin.sh apply
./standard-run.sh --scenario 02-steady-state --pool 4 --rate 630 --target https://10.10.0.2 --label my-run
```

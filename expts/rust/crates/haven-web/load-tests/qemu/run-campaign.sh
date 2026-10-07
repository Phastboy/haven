#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

echo "========================================="
echo " Starting Full Benchmark Campaign"
echo "========================================="

# 1. Calibration
echo "[1/10] Calibration Run 1..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 1000 --target https://10.10.0.2 --label exp1-calib-run1
echo "[2/10] Calibration Run 2..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 1000 --target https://10.10.0.2 --label exp1-calib-run2
echo "[3/10] Calibration Run 3..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 1000 --target https://10.10.0.2 --label exp1-calib-run3

# 2. Nginx vs Direct
echo "[4/10] Direct to App (Bypass Nginx)..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 1000 --target http://10.10.0.2:8080 --label exp2-direct-run1

# 3. Knee Discovery
echo "[5/10] Rate Sweep: 2000 RPS..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 2000 --target https://10.10.0.2 --label exp3-rate-2k
echo "[6/10] Rate Sweep: 4000 RPS..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 4000 --target https://10.10.0.2 --label exp3-rate-4k
echo "[7/10] Rate Sweep: 6000 RPS..."
./standard-run.sh --scenario 02-steady-state --pool 16 --rate 6000 --target https://10.10.0.2 --label exp3-rate-6k

# 4. Connection Pool Sweep
echo "[8/10] Pool Sweep: 4 Connections..."
./standard-run.sh --scenario 02-steady-state --pool 4 --rate 4000 --target https://10.10.0.2 --label exp4-pool-4
echo "[9/10] Pool Sweep: 8 Connections..."
./standard-run.sh --scenario 02-steady-state --pool 8 --rate 4000 --target https://10.10.0.2 --label exp4-pool-8
echo "[10/10] Pool Sweep: 32 Connections..."
./standard-run.sh --scenario 02-steady-state --pool 32 --rate 4000 --target https://10.10.0.2 --label exp4-pool-32

echo "========================================="
echo " Campaign Complete! All data saved to results/"
echo "========================================="

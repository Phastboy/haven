#!/usr/bin/env bash
set -euo pipefail

[ "$EUID" -eq 0 ] || { echo "Run this script with sudo."; exit 1; }
REAL_USER=${SUDO_USER:?run via sudo from your own user}

echo "=== 1. Tuning Host CPU ==="
for f in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
  echo performance > "$f" 2>/dev/null || echo "Warning: failed to set governor on $f"
done
echo "Current governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'N/A')"

echo "=== 2. Stopping IRQ Balance ==="
command -v systemctl >/dev/null && systemctl stop irqbalance || true
if pgrep -x irqbalance >/dev/null; then
  echo "FATAL: irqbalance is still running. Stop it with your init system."
  exit 1
fi

echo "=== 3. Loading vhost_net ==="
modprobe vhost_net || echo "Warning: Failed to modprobe vhost_net. Is it built-in?"
ls -l /dev/vhost-net || { echo "FATAL: /dev/vhost-net not available."; exit 1; }

echo "=== 4. Setting up TAP Network ==="
if ! ip link show tap0 &>/dev/null; then
  ip tuntap add dev tap0 mode tap user "$REAL_USER"
  ip addr add 10.10.0.1/24 dev tap0
  ip link set tap0 up
else
  echo "tap0 already exists."
fi

echo "Host root setup complete! You can now run ./host-prepare.sh as your normal user."

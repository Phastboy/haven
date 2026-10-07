#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

BASE="https://cdimage.debian.org/debian-cd/current/amd64/iso-cd"
DISK="haven-bench.img"

[ -w /dev/kvm ] || { echo "FATAL: No write access to /dev/kvm. (Are you in the kvm group?)"; exit 1; }

echo "=== 1. Downloading Debian Stable ISO ==="
curl -fsSL "$BASE/SHA256SUMS" -o SHA256SUMS
ISO=$(awk '$2 ~ /^debian-[0-9.]+-amd64-netinst\.iso$/ {print $2; exit}' SHA256SUMS)
[ -n "$ISO" ] || { echo "FATAL: netinst not found in SHA256SUMS."; exit 1; }

if [ ! -f "$ISO" ]; then
  echo "Downloading $ISO..."
  curl -fL -o "$ISO.part" "$BASE/$ISO"
  mv "$ISO.part" "$ISO"
else
  echo "$ISO already exists locally."
fi

echo -n "Verifying SHA256... "
grep " $ISO\$" SHA256SUMS | sha256sum -c -

echo "=== 2. Creating VM Disk ==="
if [ ! -f "$DISK" ]; then
  echo "Preallocating 20G disk (this may take a moment)..."
  qemu-img create -f raw -o preallocation=full "$DISK" 20G
else
  echo "$DISK already exists."
fi

echo "=== 3. Launching QEMU Installer ==="
echo "Install with no swap partition, SSH server & standard system utilities only (no desktop environment)."
echo "When it reboots and finishes, shut it down from inside the guest."

exec qemu-system-x86_64 -enable-kvm -machine q35,accel=kvm -cpu host -smp 1 -m 2G \
  -drive file="$DISK",format=raw,if=virtio,cache=none,aio=native \
  -cdrom "$ISO" \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0,mac=52:54:00:12:34:56 \
  -audiodev none,id=snd0 -device intel-hda -device hda-output,audiodev=snd0 \
  -display gtk,zoom-to-fit=on

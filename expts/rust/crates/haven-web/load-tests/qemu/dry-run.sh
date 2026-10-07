#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

DISK=haven-bench.img
PIDFILE=vm.pid
SERIAL=serial.sock
MON=monitor.sock
GUEST=10.10.0.2
PIN_CORES=${PIN_CORES:-}        # e.g. PIN_CORES=2,6 (X,Y); empty = unpinned
SSH=(ssh -o BatchMode=yes -o ConnectTimeout=3 -o StrictHostKeyChecking=accept-new root@$GUEST)

[ -f "$DISK" ] || { echo "Disk image not found"; exit 1; }
ip link show tap0 >/dev/null 2>&1 || { echo "tap0 missing: run sudo ./host-root.sh"; exit 1; }
[ -w /dev/kvm ] && [ -w /dev/vhost-net ] || { echo "need write access to /dev/kvm and /dev/vhost-net"; exit 1; }
if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
  echo "VM already running (PID $(cat "$PIDFILE"))"; exit 1
fi
rm -f "$SERIAL" "$MON" "$PIDFILE"

PREFIX=()
[ -n "$PIN_CORES" ] && PREFIX=(taskset -c "$PIN_CORES")

"${PREFIX[@]}" qemu-system-x86_64 -enable-kvm -machine q35,accel=kvm -cpu host -smp 1 -m 2G \
  -drive file="$DISK",format=raw,if=virtio,cache=none,aio=native \
  -netdev tap,id=n0,ifname=tap0,script=no,downscript=no,vhost=on \
  -device virtio-net-pci,netdev=n0,mac=52:54:00:12:34:56 \
  -chardev socket,id=ser0,path="$SERIAL",server=on,wait=off,logfile=serial.log \
  -serial chardev:ser0 \
  -monitor unix:"$MON",server,nowait \
  -display none -daemonize -pidfile "$PIDFILE"

echo "VM launched (PID $(cat "$PIDFILE")). Waiting for SSH (up to 180s)..."
for i in $(seq 1 60); do
  if "${SSH[@]}" true 2>/dev/null; then echo "SSH up after ~$((i*3))s"; break; fi
  kill -0 "$(cat "$PIDFILE")" 2>/dev/null || { echo "QEMU exited early; see serial.log"; exit 1; }
  if [ "$i" -eq 60 ]; then
    echo "SSH not reachable. Check serial.log, or attach: socat -,rawer UNIX-CONNECT:$SERIAL"
    exit 1
  fi
  sleep 3
done

echo "Next: pin threads (X/Y), then run the checks below."
echo "Stop with: ssh root@$GUEST poweroff   (fallback: echo system_powerdown | socat - UNIX-CONNECT:$MON)"

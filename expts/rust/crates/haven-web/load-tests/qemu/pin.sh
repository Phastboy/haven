#!/usr/bin/env bash
# pin.sh {apply|reset|check}: apply/reset require sudo; check is read-only (no sudo needed).
set -euo pipefail
X=${X:-1}; OFF=${OFF:-3}; Y=${Y:-0}; HOST=${HOST_CPUS:-0}; ALL=${ALL_CPUS:-0-3}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# check is read-only (/proc is world-readable); apply and reset require root
case ${1:-} in apply|reset) [ "$EUID" -eq 0 ] || { echo "run with sudo"; exit 1; } ;; esac
SYS=/sys/devices/system/cpu
tasks() { ls /proc | grep -E '^[0-9]+$'; }

_qpid() {
  local pid=""
  [ -f "$SCRIPT_DIR/vm.pid" ] && pid=$(cat "$SCRIPT_DIR/vm.pid")
  if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
    pid=$(pgrep -f 'qemu-system-x86_64.*haven-bench\.img' | head -n1 || true)
  fi
  [ -n "$pid" ] || { echo "QEMU not running (check vm.pid)" >&2; exit 1; }
  echo "$pid"
}

check_pin() {
  fail=0
  QPID=$(_qpid)

  for t in /proc/$QPID/task/*; do
    [ -d "$t" ] || continue
    comm=$(<"$t/comm"); al=$(awk '/Cpus_allowed_list/{print $2}' "$t/status")
    want=$Y; case $comm in CPU*KVM) want=$X;; esac
    [ "$al" = "$want" ] || { echo "FAIL tid ${t##*/} ($comm): allowed=$al want=$want"; fail=1; }
  done
  for t in $(pgrep "vhost-$QPID" || true); do
    al=$(awk '/Cpus_allowed_list/{print $2}' "/proc/$t/status")
    [ "$al" = "$Y" ] || { echo "FAIL vhost $t: allowed=$al want=$Y"; fail=1; }
  done

  if [ "$fail" -eq 0 ]; then
    echo "pinning OK: vcpu=$X helpers/vhost=$Y host=$HOST k6=${K6:-2}"
  fi
  return $fail
}

case ${1:?apply|reset|check} in
apply)
  # 0. Disable swap (required for reproducible benchmarks)
  if swapon --show | grep -q .; then
    echo "Disabling swap..."
    swapoff -a
    echo "swap disabled"
  else
    echo "swap already off"
  fi

  # 1. Take X's SMT sibling offline (verify it really is the sibling first)
  if [ "$(<$SYS/cpu$OFF/online)" = 1 ]; then
    sib=$(<$SYS/cpu$X/topology/thread_siblings_list)
    case ",$sib," in *",$OFF,"*) ;; *) echo "cpu$OFF is not a sibling of cpu$X ($sib)"; exit 1;; esac
    echo 0 > $SYS/cpu$OFF/online
  fi
  grep -q performance $SYS/cpu$X/cpufreq/scaling_governor \
    || { echo "governor is not performance on cpu$X"; exit 1; }

  # 2. IRQs to the host CPU (managed queues will refuse; that's expected)
  moved=0 refused=0
  for f in /proc/irq/*/smp_affinity_list; do
    if echo "$HOST" > "$f" 2>/dev/null; then moved=$((moved+1)); else refused=$((refused+1)); fi
  done
  echo "IRQs moved to cpu$HOST: $moved (refused: $refused)"

  # 3. Push every non-QEMU task to the host CPU
  QPID=$(_qpid)
  for p in $(tasks); do
    [ "$p" = "$QPID" ] && continue
    taskset -apc "$HOST" "$p" >/dev/null 2>&1 || true
  done

  # 4. QEMU: vCPU thread -> X, everything else -> Y, vhost -> Y
  ps -L -o tid=,comm= -p "$QPID" | while read -r tid comm; do
    case $comm in "CPU "*"/KVM") taskset -pc "$X" "$tid" >/dev/null;;
                  *)              taskset -pc "$Y" "$tid" >/dev/null;; esac
  done
  for t in $(pgrep "vhost-$QPID" || true); do taskset -pc "$Y" "$t" >/dev/null; done

  # 5. Verify from kernel records (Cpus_allowed_list in /proc/*/status)
  check_pin
  exit $?
  ;;
reset)
  echo 1 > $SYS/cpu$OFF/online 2>/dev/null || true
  for f in /proc/irq/*/smp_affinity_list; do echo "$ALL" > "$f" 2>/dev/null || true; done
  for p in $(tasks); do taskset -apc "$ALL" "$p" >/dev/null 2>&1 || true; done
  echo "reset to $ALL"
  ;;
check)
  check_pin
  exit $?
  ;;
esac

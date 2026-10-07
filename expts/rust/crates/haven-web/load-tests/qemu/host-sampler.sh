#!/usr/bin/env bash
# HOST: host-sampler.sh <outfile> <qemu_pid> <X> <Y> <K6_CORE>; ticks are USER_HZ (usually 100)
set -u
OUT=$1 QPID=$2 X=$3 Y=$4 K6_CORE=$5
echo "ts,qemu_vcpu_ticks,qemu_other_ticks,vhost_ticks,coreX_busy,coreX_total,coreY_busy,coreY_total,coreK6_busy,coreK6_total,k6_process_ticks,cpu1_scaling_cur_freq" > "$OUT"
trap 'exit 0' TERM INT
while :; do
  ts=$EPOCHREALTIME; V=0 O=0 H=0
  for t in /proc/$QPID/task/*; do
    s=$(<"$t/stat") || continue
    nm=${s#*(}; nm=${nm%)*}; set -- ${s##*) }; k=$(( ${12} + ${13} ))
    case $nm in "CPU "*"/KVM") V=$((V+k));; *) O=$((O+k));; esac
  done
  for p in $(pgrep "vhost-$QPID" || true); do s=$(<"/proc/$p/stat") || continue; set -- ${s##*) }; H=$((H+${12}+${13})); done
  
  XB=0; XT=0; YB=0; YT=0; K6B=0; K6T=0
  while read -r c u n sy i w irq sirq st _; do
    tot=$((u+n+sy+i+w+irq+sirq+st)); busy=$((tot-i-w))
    case $c in 
      cpu$X) XB=$busy XT=$tot;; 
      cpu$Y) YB=$busy YT=$tot;;
      cpu$K6_CORE) K6B=$busy K6T=$tot;;
    esac
  done < /proc/stat
  
  K6_TICKS=0
  K6_PID=$(pgrep -x k6 | head -n1 || true)
  if [ -n "$K6_PID" ] && [ -r "/proc/$K6_PID/stat" ]; then
    s=$(<"/proc/$K6_PID/stat") || true
    if [ -n "$s" ]; then
      set -- ${s##*) }
      K6_TICKS=$(( ${12} + ${13} ))
    fi
  fi
  
  # Get scaling_cur_freq for CPU 1 (which is X)
  FREQ=0
  if [ -r "/sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq" ]; then
    FREQ=$(<"/sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq")
  fi

  echo "$ts,$V,$O,$H,$XB,$XT,$YB,$YT,$K6B,$K6T,$K6_TICKS,$FREQ" >> "$OUT"
  now=${EPOCHREALTIME/./}; next=$(( now/1000000*1000000 + 1000000 ))
  printf -v d '0.%06d' $(( next - now )); sleep "$d"
done

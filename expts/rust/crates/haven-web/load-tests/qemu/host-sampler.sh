#!/usr/bin/env bash
# HOST: host-sampler.sh <outfile> <qemu_pid> <X> <Y> <LOAD_CORE>; ticks are USER_HZ (usually 100)
set -u
OUT=$1 QPID=$2 X=$3 Y=$4 LOAD_CORE=$5
echo "ts,qemu_vcpu_ticks,qemu_other_ticks,vhost_ticks,coreX_busy,coreX_total,coreY_busy,coreY_total,coreLoad_busy,coreLoad_total,load_process_ticks,cpu1_scaling_cur_freq" > "$OUT"
trap 'exit 0' TERM INT
while :; do
  ts=$EPOCHREALTIME; V=0 O=0 H=0
  for t in /proc/$QPID/task/*; do
    s=$(<"$t/stat") || continue
    nm=${s#*(}; nm=${nm%)*}; set -- ${s##*) }; k=$(( ${12} + ${13} ))
    case $nm in "CPU "*"/KVM") V=$((V+k));; *) O=$((O+k));; esac
  done
  for p in $(pgrep "vhost-$QPID" || true); do s=$(<"/proc/$p/stat") || continue; set -- ${s##*) }; H=$((H+${12}+${13})); done
  
  XB=0; XT=0; YB=0; YT=0; LB=0; LT=0
  while read -r c u n sy i w irq sirq st _; do
    tot=$((u+n+sy+i+w+irq+sirq+st)); busy=$((tot-i-w))
    case $c in 
      cpu$X) XB=$busy XT=$tot;; 
      cpu$Y) YB=$busy YT=$tot;;
      cpu$LOAD_CORE) LB=$busy LT=$tot;;
    esac
  done < /proc/stat
  
  LOAD_TICKS=0
  LOAD_PID=$(pgrep -x haven-load | head -n1 || true)
  if [ -n "$LOAD_PID" ] && [ -r "/proc/$LOAD_PID/stat" ]; then
    s=$(<"/proc/$LOAD_PID/stat") || true
    if [ -n "$s" ]; then
      set -- ${s##*) }
      LOAD_TICKS=$(( ${12} + ${13} ))
    fi
  fi
  
  # Get scaling_cur_freq for CPU 1 (which is X)
  FREQ=0
  if [ -r "/sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq" ]; then
    FREQ=$(<"/sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq")
  fi

  echo "$ts,$V,$O,$H,$XB,$XT,$YB,$YT,$LB,$LT,$LOAD_TICKS,$FREQ" >> "$OUT"
  now=${EPOCHREALTIME/./}; next=$(( now/1000000*1000000 + 1000000 ))
  printf -v d '0.%06d' $(( next - now )); sleep "$d"
done

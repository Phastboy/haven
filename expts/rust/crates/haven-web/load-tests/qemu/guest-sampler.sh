#!/usr/bin/env bash
# guest-sampler.sh
# Runs INSIDE the guest. Raw cumulative counters, 1 Hz, no per-sample forks except sleep.
# Verified: bash 5.2, vda disk, PSI supported.
set -u
OUT=${1:?outfile}
CG=/sys/fs/cgroup/system.slice
APP=$CG/haven-web.service
PG=$CG/system-postgresql.slice/postgresql@18-main.service
NGX=$CG/nginx.service
# haven-web cgroup only exists while the service is running; warn but continue.
[ -r "$PG/cpu.stat" ] || { echo "bad cgroup path: $PG" >&2; exit 1; }
[ -r "$NGX/cpu.stat" ] || { echo "bad cgroup path: $NGX" >&2; exit 1; }
[ -r "$APP/cpu.stat" ] || echo "WARN: haven-web cgroup not found yet (will read zeroes until app starts)" >&2

cg() {  # sets APP_CPU APP_USR APP_SYS APP_MEM (or PG_* or NGX_*) for cgroup dir $1; zeros if absent
  local k v _cpu=0 _usr=0 _sys=0 _mem=0
  if [ -r "$1/cpu.stat" ]; then
    while read -r k v; do
      case $k in usage_usec) _cpu=$v;; user_usec) _usr=$v;; system_usec) _sys=$v;; esac
    done < "$1/cpu.stat"
    read -r _mem < "$1/memory.current" 2>/dev/null || _mem=0
  fi
  # Return via positional name prefix passed as $2
  eval "${2}_CPU=$_cpu ${2}_USR=$_usr ${2}_SYS=$_sys ${2}_MEM=$_mem"
}
psi() { local l; read -r l < "$1" 2>/dev/null || { PSI=NA; return; }; l=${l##* }; PSI=${l#total=}; }

echo "ts,app_cpu_us,app_usr_us,app_sys_us,app_mem,pg_cpu_us,pg_usr_us,pg_sys_us,pg_mem,ngx_cpu_us,ngx_usr_us,ngx_sys_us,ngx_mem,cpu_user,cpu_nice,cpu_sys,cpu_idle,cpu_iowait,cpu_irq,cpu_softirq,cpu_steal,mem_avail_kb,dirty_kb,vda_rd_sect,vda_rd_ms,vda_wr_sect,vda_wr_ms,psi_cpu_us,psi_mem_us,psi_io_us,oom_kill" > "$OUT"
trap 'exit 0' TERM INT

while :; do
  TS=$EPOCHREALTIME
  cg "$APP" A
  cg "$PG"  P
  cg "$NGX" N
  read -r _ u ni s i w irq sirq st _ < /proc/stat
  MA=0 DT=0
  while read -r k v _; do case $k in MemAvailable:) MA=$v;; Dirty:) DT=$v;; esac; done < /proc/meminfo
  RS=0 RM=0 WS=0 WM=0
  while read -r _ _ name _ _ rs rm _ _ ws wm _; do
    [ "$name" = vda ] && { RS=$rs; RM=$rm; WS=$ws; WM=$wm; }
  done < /proc/diskstats
  psi /proc/pressure/cpu;    PC=$PSI
  psi /proc/pressure/memory; PM=$PSI
  psi /proc/pressure/io;     PI=$PSI
  OOM=0
  while read -r k v; do [ "$k" = oom_kill ] && OOM=$v; done < "$PG/memory.events"
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$TS" \
    "$A_CPU" "$A_USR" "$A_SYS" "$A_MEM" \
    "$P_CPU" "$P_USR" "$P_SYS" "$P_MEM" \
    "$N_CPU" "$N_USR" "$N_SYS" "$N_MEM" \
    "$u" "$ni" "$s" "$i" "$w" "$irq" "$sirq" "$st" \
    "$MA" "$DT" \
    "$RS" "$RM" "$WS" "$WM" \
    "$PC" "$PM" "$PI" "$OOM" >> "$OUT"
  now=${EPOCHREALTIME/./}; next=$(( now/1000000*1000000 + 1000000 ))
  printf -v d '0.%06d' $(( next - now )); sleep "$d"
done

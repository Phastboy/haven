#!/usr/bin/env bash
set -euo pipefail
CMD=${1:?start|stop}; DIR=${2:?results dir}
SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=5 -o ControlMaster=auto -o ControlPath=/tmp/haven-%C -o ControlPersist=15m)
g() { ssh "${SSH_OPTS[@]}" root@10.10.0.2 "$@"; }

case $CMD in
start)
  QPID=$3 X=$4 Y=$5 K6_CORES=${6:?cores for k6}
  # Pin the host sampler to the host CPU (0) so it doesn't compete with K6 on CPU 2.
  HOST_SAMPLER_CORE=0

  g 'rm -rf /root/telemetry; mkdir -p /root/telemetry'
  # a failed sampler start must abort the run, not give you empty data
  g -n 'nohup /root/guest-sampler.sh /root/telemetry/guest.csv >/root/telemetry/sampler.log 2>&1 </dev/null & echo $! > /root/telemetry/sampler.pid'
  g -n 'nohup vmstat -t -n 1 >/root/telemetry/vmstat.log 2>&1 </dev/null & echo $! > /root/telemetry/vmstat.pid'
  sleep 2; g 'kill -0 $(cat /root/telemetry/sampler.pid)' || { echo "guest sampler died"; g cat /root/telemetry/sampler.log; exit 1; }
  
  taskset -c "$HOST_SAMPLER_CORE" ./host-sampler.sh "$DIR/host.csv" "$QPID" "$X" "$Y" "$K6_CORES" & echo $! > "$DIR/host-sampler.pid"
  date +%s.%N > "$DIR/telemetry_start"
  ;;
stop)
  date +%s.%N > "$DIR/telemetry_end"
  kill "$(cat "$DIR/host-sampler.pid" 2>/dev/null)" 2>/dev/null || true
  g 'kill $(cat /root/telemetry/sampler.pid /root/telemetry/vmstat.pid 2>/dev/null) 2>/dev/null || true'
  sleep 1
  scp -q "${SSH_OPTS[@]}" -r root@10.10.0.2:/root/telemetry "$DIR/guest-telemetry"
  ;;
esac

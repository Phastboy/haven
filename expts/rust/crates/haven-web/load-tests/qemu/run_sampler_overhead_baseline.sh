#!/usr/bin/env bash
# run_sampler_overhead_baseline.sh
set -euo pipefail
SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=5 -o ControlMaster=auto -o ControlPath=/tmp/haven-%C -o ControlPersist=15m)
g() { ssh "${SSH_OPTS[@]}" root@10.10.0.2 "$@"; }

echo "Uploading guest-sampler.sh..."
scp "${SSH_OPTS[@]}" guest-sampler.sh root@10.10.0.2:/root/guest-sampler.sh

echo "Starting guest sampler..."
g 'rm -rf /root/telemetry; mkdir -p /root/telemetry'
g -n 'nohup /root/guest-sampler.sh /root/telemetry/guest.csv >/root/telemetry/sampler.log 2>&1 </dev/null & echo $! > /root/telemetry/sampler.pid'
sleep 2
g 'kill -0 $(cat /root/telemetry/sampler.pid)' || { echo "guest sampler died"; exit 1; }

echo "Capturing start /proc/stat..."
g 'cat /proc/stat' > stat_start.txt

echo "Waiting 5 minutes for overhead baseline... (do not touch the machine!)"
sleep 300

echo "Capturing end /proc/stat and sampler ticks..."
g 'cat /proc/stat' > stat_end.txt
g 'cat /proc/$(cat /root/telemetry/sampler.pid)/stat' > sampler_stat.txt

echo "Stopping guest sampler..."
g 'kill $(cat /root/telemetry/sampler.pid) 2>/dev/null || true'

echo "=== Guest Sampler Overhead ==="
python3 << 'EOF'
import sys

def get_cpu_ticks(file):
    with open(file) as f:
        for line in f:
            if line.startswith('cpu '):
                parts = line.split()
                # user, nice, system, idle, iowait, irq, softirq, steal
                u, n, s, i, w, irq, sirq, st = map(int, parts[1:9])
                tot = u + n + s + i + w + irq + sirq + st
                busy = tot - i - w
                return busy, tot
    return 0, 0

busy_start, tot_start = get_cpu_ticks('stat_start.txt')
busy_end, tot_end = get_cpu_ticks('stat_end.txt')

delta_busy = busy_end - busy_start
delta_tot = tot_end - tot_start

print(f"Guest total CPU ticks: {delta_tot}")
print(f"Guest busy CPU ticks: {delta_busy}")

sampler_utime = 0
sampler_stime = 0
with open('sampler_stat.txt') as f:
    line = f.read().strip()
    if line:
        parts = line.split(') ')
        if len(parts) > 1:
            stats = parts[1].split()
            # utime is 12th after ), so index 11 (0-based)
            # stime is 13th after ), so index 12
            sampler_utime = int(stats[11])
            sampler_stime = int(stats[12])

sampler_busy = sampler_utime + sampler_stime
print(f"Sampler process ticks: {sampler_busy}")

if delta_tot > 0:
    print(f"Sampler CPU usage: {(sampler_busy / delta_tot) * 100:.2f}% of total guest time")
    print(f"Overall guest busy: {(delta_busy / delta_tot) * 100:.2f}% of total guest time")
EOF

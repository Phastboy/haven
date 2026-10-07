#!/usr/bin/env bash
# run_idle_baseline.sh
set -euo pipefail

echo "Capturing start interrupts..."
cat /proc/interrupts > interrupts_start.txt
cat /proc/softirqs > softirqs_start.txt

echo "Waiting 5 minutes for baseline... (do not touch the machine!)"
sleep 300

echo "Capturing end interrupts..."
cat /proc/interrupts > interrupts_end.txt
cat /proc/softirqs > softirqs_end.txt

echo ""
echo "=== CPU 1 Interrupt Deltas (over 5 mins) ==="
paste interrupts_start.txt interrupts_end.txt | awk '{
  # Simple parser assuming standard /proc/interrupts format
  # CPU0 is $2, CPU1 is $3 in interrupts_start, and $2, $3 in interrupts_end.
  # But paste joins them, so if start has N fields, end fields are shifted.
  # Let us just rely on a simple python script to parse it accurately.
}' > /dev/null

python3 << 'EOF'
import sys
def parse(file):
    counts = {}
    with open(file) as f:
        lines = f.readlines()
        if not lines: return counts
        # find CPU1 column index
        headers = lines[0].split()
        if 'CPU1' not in headers: return counts
        idx = headers.index('CPU1')
        for line in lines[1:]:
            parts = line.split(':')
            if len(parts) < 2: continue
            name = parts[0].strip()
            vals = parts[1].split()
            if len(vals) > idx:
                try:
                    counts[name] = int(vals[idx])
                except:
                    pass
    return counts

start_int = parse('interrupts_start.txt')
end_int = parse('interrupts_end.txt')
start_soft = parse('softirqs_start.txt')
end_soft = parse('softirqs_end.txt')

print("Interrupts (Delta / sec):")
for k, v in end_int.items():
    s = start_int.get(k, 0)
    delta = v - s
    if delta > 0:
        print(f"  {k}: {delta} total, {delta/300.0:.2f}/sec")

print("\nSoftIRQs (Delta / sec):")
for k, v in end_soft.items():
    s = start_soft.get(k, 0)
    delta = v - s
    if delta > 0:
        print(f"  {k}: {delta} total, {delta/300.0:.2f}/sec")
EOF

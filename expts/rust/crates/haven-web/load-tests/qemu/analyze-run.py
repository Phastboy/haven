#!/usr/bin/env python3
import sys
import os
import json
import csv

def parse_interrupts(file_path):
    counts = {}
    if not os.path.exists(file_path):
        return counts
    with open(file_path, 'r') as f:
        lines = f.readlines()
        if not lines: return counts
        headers = lines[0].split()
        if 'CPU1' not in headers: return counts
        idx = headers.index('CPU1')
        for line in lines[1:]:
            parts = line.split(':')
            if len(parts) < 2: continue
            name = parts[0].strip()
            vals = parts[1].split()
            if len(vals) > idx:
                try: counts[name] = int(vals[idx])
                except: pass
    return counts

def analyze_run(results_dir):
    status_file = os.path.join(results_dir, "status")
    with open(status_file, "r") as f:
        status = f.read().strip()
    
    if status != "PASS":
        print(f"Run status is already {status}. Skipping validity checks.")
        return

    invalid_reasons = []

    # 1. K6 Dropped Iterations
    k6_summary_file = os.path.join(results_dir, "k6_summary.json")
    if os.path.exists(k6_summary_file):
        with open(k6_summary_file, "r") as f:
            k6 = json.load(f)
            dropped = k6.get("metrics", {}).get("dropped_iterations", {}).get("values", {}).get("count", 0)
            if dropped > 0:
                invalid_reasons.append(f"Dropped iterations: {dropped}")
    else:
        invalid_reasons.append("Missing k6_summary.json")

    # 2. Host Core & K6 Core Saturation
    host_csv = os.path.join(results_dir, "host.csv")
    if os.path.exists(host_csv):
        with open(host_csv, "r") as f:
            reader = list(csv.DictReader(f))
            if len(reader) > 1:
                start = reader[0]
                end = reader[-1]
                try:
                    k6_busy_delta = int(end["coreK6_busy"]) - int(start["coreK6_busy"])
                    k6_tot_delta = int(end["coreK6_total"]) - int(start["coreK6_total"])
                    if k6_tot_delta > 0:
                        k6_util = k6_busy_delta / k6_tot_delta
                        if k6_util > 0.70:
                            invalid_reasons.append(f"K6 core utilization too high: {k6_util:.1%}")

                    host_busy_delta = int(end["coreY_busy"]) - int(start["coreY_busy"])
                    host_tot_delta = int(end["coreY_total"]) - int(start["coreY_total"])
                    if host_tot_delta > 0:
                        host_util = host_busy_delta / host_tot_delta
                        if host_util > 0.90:
                            invalid_reasons.append(f"Host core utilization too high: {host_util:.1%}")
                except Exception as e:
                    invalid_reasons.append(f"Error parsing host.csv metrics: {e}")
            else:
                invalid_reasons.append("Insufficient data in host.csv")
    else:
        invalid_reasons.append("Missing host.csv")

    # 3. CPU 1 Interrupts Flatness
    int_start = parse_interrupts(os.path.join(results_dir, "interrupts_start.txt"))
    int_end = parse_interrupts(os.path.join(results_dir, "interrupts_end.txt"))
    
    # Allowed timer/IPI interrupts on CPU 1 (informational)
    allowed_irqs = {"LOC", "RES", "CAL", "TLB", "IWI", "MCP", "NMI", "PMI", "TRM", "THR"}
    
    for irq, count_end in int_end.items():
        if irq in allowed_irqs:
            continue
        count_start = int_start.get(irq, 0)
        delta = count_end - count_start
        if delta > 5:
            invalid_reasons.append(f"CPU 1 device interrupt '{irq}' not flat (delta={delta})")

    # Write outcome
    with open(os.path.join(results_dir, "run_analysis.txt"), "w") as f:
        if invalid_reasons:
            f.write("INVALID RUN\n")
            for r in invalid_reasons:
                f.write(f"- {r}\n")
        else:
            f.write("VALID RUN\n")
            f.write("- K6 core utilization < 70%\n")
            f.write("- Host core utilization < 90%\n")
            f.write("- CPU 1 device interrupts flat (< 5 per run)\n")
            f.write("- No dropped iterations\n")

    if invalid_reasons:
        print("Run marked INVALID:")
        for r in invalid_reasons:
            print(f"  - {r}")
        with open(status_file, "w") as f:
            f.write("INVALID\n")
    else:
        print("Run analysis: VALID")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: analyze-run.py <results_dir>")
        sys.exit(1)
    analyze_run(sys.argv[1])

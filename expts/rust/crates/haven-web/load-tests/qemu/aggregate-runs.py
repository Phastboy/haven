#!/usr/bin/env python3
import sys
import os
import glob

def get_status(run_dir):
    p = os.path.join(run_dir, "status")
    if os.path.exists(p):
        with open(p) as f:
            return f.read().strip()
    return "UNKNOWN"

def get_meta(run_dir):
    meta = {}
    p = os.path.join(run_dir, "run_meta.txt")
    if os.path.exists(p):
        with open(p) as f:
            for line in f:
                if '=' in line:
                    k, v = line.strip().split('=', 1)
                    meta[k] = v
    return meta

def main():
    if len(sys.argv) < 2:
        print("Usage: aggregate-runs.py <results_base_dir> <label>")
        sys.exit(1)
        
    base_dir = sys.argv[1]
    label = sys.argv[2]
    
    # Find all runs matching the label
    search_pattern = os.path.join(base_dir, f"{label}_pool*")
    run_dirs = [d for d in glob.glob(search_pattern) if os.path.isdir(d)]
    
    passes = []
    for d in run_dirs:
        if get_status(d) == "PASS":
            passes.append(d)
            
    print(f"Found {len(run_dirs)} runs for label '{label}', {len(passes)} are PASS.")
    
    if len(passes) < 3:
        print("Need at least 3 PASS runs to aggregate.")
        sys.exit(1)
        
    # Group by pool size and parameters
    groups = {}
    for d in passes:
        meta = get_meta(d)
        pool = meta.get("pool", "unknown")
        rate = meta.get("rate", "unknown")
        key = (pool, rate)
        if key not in groups:
            groups[key] = []
        groups[key].append(d)
        
    for (pool, rate), runs in groups.items():
        if len(runs) >= 3:
            print(f"\n[OK] Found {len(runs)} identical passes for pool={pool} rate={rate}.")
            print("Runs:")
            for r in runs:
                print(f"  - {os.path.basename(r)}")
            print("Ready for final aggregation.")
            # Here we would average out throughput, latency, etc. 
            # For now, just validating the campaign succeeded.

if __name__ == "__main__":
    main()

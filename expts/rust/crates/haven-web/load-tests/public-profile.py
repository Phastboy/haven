#!/usr/bin/env python3
"""Bounded Public diagnostic: baseline versus JIT off, never a release gate."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import threading
import importlib.util


def sql(query):
    return subprocess.check_output(
        ['psql', os.environ['DATABASE_URL'], '-XAt', '-v', 'ON_ERROR_STOP=1', '-c', query],
        text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', required=True)
    parser.add_argument('--fixture', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    sql('CREATE EXTENSION IF NOT EXISTS pg_stat_statements')
    (output / 'postgres-settings.json').write_text(sql("""
        SELECT json_object_agg(name, setting) FROM pg_settings WHERE name IN
        ('server_version','jit','jit_above_cost','jit_inline_above_cost',
         'jit_optimize_above_cost','shared_buffers','work_mem',
         'max_parallel_workers_per_gather','pg_stat_statements.track_planning')
    """) + '\n')
    plans_spec = importlib.util.spec_from_file_location('plans', Path(__file__).with_name('public-plans.py'))
    plans = importlib.util.module_from_spec(plans_spec)
    plans_spec.loader.exec_module(plans)
    plans.capture(args.fixture, output)
    (output / 'offer-indexes.json').write_text(sql("""
        SELECT json_agg(i) FROM (SELECT indexname, indexdef FROM pg_indexes
        WHERE tablename = 'offer') i
    """) + '\n')
    # Two fixed 60-second verification stages; no capacity search or automatic reruns.
    spec = importlib.util.spec_from_file_location('gate', Path(__file__).with_name('v07-gate.py'))
    gate = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(gate)
    results = []
    for jit, rate in (('on', 100), ('on', 1010)):
        label = f'public-jit-{jit}-{rate}'
        sql(f"ALTER SYSTEM SET jit = '{jit}'")
        sql('SELECT pg_reload_conf()')
        time.sleep(2)
        sql('SELECT pg_stat_statements_reset()')
        pool_log = output / 'pool-acquire-seconds.txt'
        pool_start = pool_log.stat().st_size if pool_log.exists() else 0
        stop = threading.Event()
        sampler = threading.Thread(target=gate.monitor, args=(output / f'{label}-host.jsonl', stop))
        sampler.start()
        container_log = (output / f'{label}-database-stats.jsonl').open('w')
        container = subprocess.Popen(['docker', 'stats', '--format', '{{json .}}',
                                      os.environ['POSTGRES_CONTAINER']], stdout=container_log)
        summary = output / f'{label}.json'
        command = [args.runner, '--base-url', 'http://127.0.0.1:3337',
                   '--scenario', 'v07', '--narrowing-fixture', args.fixture,
                   '--query-class', 'public', '--history', 'all',
                   '--rate', str(rate), '--duration', '60s', '--workers', '200',
                   '--semantic-percent', '100', '--summary', str(summary)]
        try:
            with (output / f'{label}.log').open('w') as log:
                run = subprocess.run(command, stdout=log, stderr=log, check=False, timeout=90)
        finally:
            stop.set()
            sampler.join()
            container.terminate()
            try:
                container.wait(timeout=5)
            except subprocess.TimeoutExpired:
                container.kill()
                container.wait(timeout=5)
            container_log.close()
        # Preserve statement costs even when arrivals or correctness fail.
        (output / f'{label}-statements.json').write_text(sql("""
            SELECT COALESCE(json_agg(s), '[]'::json) FROM (
              SELECT * FROM pg_stat_statements
              WHERE dbid = (SELECT oid FROM pg_database WHERE datname=current_database())
                AND query NOT LIKE '%pg_stat_statements%'
              ORDER BY total_exec_time + total_plan_time DESC LIMIT 40
            ) s
        """) + '\n')
        data = json.loads(summary.read_text())
        samples = [float(v) for v in pool_log.read_bytes()[pool_start:].decode().splitlines()]
        data['pool_acquire'] = gate.summarize_pool_acquire(samples)
        summary.write_text(json.dumps(data, indent=2) + '\n')
        result = dict(jit=jit, rate=rate, completed=data['completed'], dropped=data['dropped'],
                      failed=data['failed'], checks=data['checks'],
                      checks_passed=data['checks_passed'], exit_code=run.returncode,
                      p95_ms=data['overall']['p95_ns'] / 1e6)
        results.append(result)
        print(json.dumps(result), flush=True)
        time.sleep(2)
    sql('ALTER SYSTEM RESET jit')
    sql('SELECT pg_reload_conf()')
    (output / 'public-profile-result.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Bounded Saved capacity diagnostic with PostgreSQL statement costs."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time


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
    results = []
    for rate in (25, 50, 75, 100):
        label = f'saved-dense-{rate}'
        sql('SELECT pg_stat_statements_reset()')
        summary = output / f'{label}.json'
        command = [args.runner, '--base-url', 'http://127.0.0.1:3337',
                   '--scenario', 'v07', '--narrowing-fixture', args.fixture,
                   '--query-class', 'saved', '--history', 'dense',
                   '--rate', str(rate), '--duration', '60s', '--workers', '200',
                   '--semantic-percent', '100', '--summary', str(summary)]
        with (output / f'{label}.log').open('w') as log:
            run = subprocess.run(command, stdout=log, stderr=log, check=False)
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
        result = dict(rate=rate, completed=data['completed'], dropped=data['dropped'],
                      failed=data['failed'], checks=data['checks'],
                      checks_passed=data['checks_passed'], exit_code=run.returncode,
                      p95_ms=data['overall']['p95_ns'] / 1e6)
        results.append(result)
        print(json.dumps(result), flush=True)
        time.sleep(2)
    (output / 'saved-profile-result.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()

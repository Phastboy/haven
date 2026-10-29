#!/usr/bin/env python3
"""Run release narrowing gates; private fixtures are never copied to results."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time


def monitor(output, stop):
    with output.open('w', buffering=1) as stream:
        while not stop.is_set():
            processes = []
            for directory in Path('/proc').glob('[0-9]*'):
                try:
                    name = (directory / 'comm').read_text().strip()
                    if name not in ('haven-web', 'haven-load', 'postgres'):
                        continue
                    stat = (directory / 'stat').read_text().split()
                    status = (directory / 'status').read_text().splitlines()
                    rss = next(int(line.split()[1]) for line in status if line.startswith('VmRSS:'))
                    processes.append(dict(pid=int(directory.name), name=name, rss_kb=rss,
                                          cpu_ticks=int(stat[13]) + int(stat[14]),
                                          descriptors=len(list((directory / 'fd').iterdir()))))
                except (OSError, ValueError, StopIteration):
                    continue
            stream.write(json.dumps(dict(at=time.time(), processes=processes,
                                         cpu=Path('/proc/stat').read_text().splitlines()[0],
                                         memory=Path('/proc/meminfo').read_text(),
                                         load=Path('/proc/loadavg').read_text())) + '\n')
            stop.wait(5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', required=True)
    parser.add_argument('--fixture', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--base-url', default='http://127.0.0.1:3337')
    args = parser.parse_args()
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    stop = threading.Event()
    sampler = threading.Thread(target=monitor, args=(output / 'telemetry.jsonl', stop))
    sampler.start()
    try:
        # Every history has its own gate: adding zero-history people must not
        # dilute the dense-history workload into an easier aggregate.
        strata = [('public', 'all')] + [(scope, history)
                  for scope in ('public-authenticated', 'saved', 'inquired')
                  for history in ('dense', 'sparse', 'zero')]
        phases = [(f'correctness-{scope}-{history}', scope, 100, '60s', 100, 'v07', history)
                  for scope, history in strata]
        phases += [(f'contract-{scope}-{history}', scope, 1010, '60s', 10, 'v07', history)
                   for scope, history in strata]
        phases += [('mixed', 'mixed', 1010, '60s', 10, 'v07', 'all'),
                   ('spike', 'mixed', 100, '180s', 10, 'spike', 'all'),
                   ('soak', 'mixed', 1010, '30m', 10, 'v07', 'all'),
                   ('capacity', 'mixed', 100, '120s', 10, 'capacity', 'all')]
        for label, scope, rate, duration, percent, scenario, history in phases:
            pool_log = output / 'pool-acquire-seconds.txt'
            pool_log_start = pool_log.stat().st_size if pool_log.exists() else 0
            summary = output / (label + '.json')
            command = [args.runner, '--base-url', args.base_url, '--scenario', scenario,
                       '--profile-target', 'v07', '--narrowing-fixture', args.fixture,
                       '--query-class', scope, '--history', history, '--rate', str(rate), '--duration', duration,
                       '--workers', '200', '--semantic-percent', str(percent),
                       '--sample-seed', '42', '--summary', str(summary)]
            print(f'Starting {label}', flush=True)
            with (output / (label + '.log')).open('w') as log:
                result = subprocess.run(command, stdout=log, stderr=log, check=False)
            time.sleep(0.3)  # flush telemetry after the measured phase has ended
            if not summary.exists():
                raise RuntimeError(f'{label}: runner exited {result.returncode} without summary')
            data = json.loads(summary.read_text())
            pool_samples = [float(value) for value in pool_log.read_bytes()[pool_log_start:].decode().splitlines()]
            data['pool_acquire'] = summarize_pool_acquire(pool_samples)
            summary.write_text(json.dumps(data, indent=2) + '\n')
            if not pool_samples:
                raise RuntimeError(f'{label}: SQLx pool acquisition telemetry was empty')
            latency = data['overall']
            print(json.dumps(dict(phase=label, rps=data['requests_per_second'],
                                  p95_ms=latency['p95_ns'] / 1e6,
                                  p99_ms=latency['p99_ns'] / 1e6,
                                  pool_acquire_p95_ms=data['pool_acquire']['p95_seconds'] * 1000,
                                  failures=data['failed'], dropped=data['dropped'])), flush=True)
            # Capacity intentionally discovers failure; preserve its stage results.
            if scenario == 'capacity':
                if not data.get('profile_failed'):
                    raise RuntimeError('Capacity ladder exhausted: extend beyond final stage')
                continue
            if result.returncode or data['failed'] or data['dropped'] or data['checks'] != data['checks_passed']:
                raise RuntimeError(f'{label}: request/correctness/arrival failure')
            semantic = data.get('semantic_cases', {})
            if not semantic or any(count['checked'] != count['passed'] for count in semantic.values()):
                raise RuntimeError(f'{label}: semantic checks failed or absent')
            if scenario == 'spike':
                if data.get('profile_failed'):
                    raise RuntimeError('Spike/recovery budget failed')
            elif not label.startswith('correctness'):
                if (data['completed'] / (data['duration_ns'] / 1e9) < 1000
                        or any(route['p95_ns'] > 200e6 or route['p99_ns'] > 500e6
                               for route in data['routes'])):
                    raise RuntimeError(f'{label}: throughput/latency budget failed')
        print('HTTP gates complete; review telemetry for bounded memory and CPU headroom.', flush=True)
    finally:
        stop.set()
        sampler.join()


def summarize_pool_acquire(values):
    values.sort()

    def percentile(fraction):
        if not values:
            return None
        index = max(0, int(len(values) * fraction + 0.999999) - 1)
        return values[index]

    return {'scope': 'SQLx connection acquisition (wait plus connection creation)',
            'count': len(values), 'p50_seconds': percentile(0.50),
            'p95_seconds': percentile(0.95), 'p99_seconds': percentile(0.99)}


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Low-load real-application journey checks; credentials stay in a private directory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
from urllib.parse import unquote, urlparse
from urllib.request import urlopen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True)
    parser.add_argument('--private', required=True)
    parser.add_argument('--scenario', choices=('messaging', 'narrowing'), default='messaging')
    args = parser.parse_args()
    narrowing = args.scenario == 'narrowing'
    port = '3342' if narrowing else '3341'
    database_name = 'haven_narrowing_ci' if narrowing else 'haven_messaging_ci'
    profile = 'release' if narrowing else 'debug'
    output, private = Path(args.output).resolve(), Path(args.private).resolve()
    if output == private or output in private.parents or private in output.parents:
        raise ValueError('public evidence and private credentials must be separate directories')
    output.mkdir(parents=True, exist_ok=True)
    private.mkdir(parents=True, exist_ok=True, mode=0o700)
    private.chmod(0o700)
    root = Path(__file__).resolve().parents[3]
    result = {'status': 'incomplete', 'scenario': args.scenario, 'profile': profile,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              'scope': 'low-load correctness only'}
    report = output / 'result.json'
    report.write_text(json.dumps(result, indent=2))
    server = None
    try:
        env = os.environ.copy()
        database = urlparse(env['DATABASE_URL'])
        if database.hostname not in ('localhost', '127.0.0.1', '::1'):
            raise ValueError('journey correctness requires a local disposable database')
        postgres_env = dict(env, PGHOST=database.hostname, PGPORT=str(database.port or 5432),
                            PGUSER=unquote(database.username or ''),
                            PGPASSWORD=unquote(database.password or ''),
                            PGDATABASE=unquote(database.path.lstrip('/')))
        subprocess.run(['psql', '-X', '-v', 'ON_ERROR_STOP=1', '-c',
                        'CREATE DATABASE ' + database_name], env=postgres_env, check=True)
        env.update(DATABASE_URL=database._replace(path='/' + database_name).geturl(),
                   PORT=port, HOST='127.0.0.1', PUBLIC_BASE_URL='http://127.0.0.1:' + port,
                   EMAIL_DELIVERY_ENABLED='false')
        runner = private / 'haven-load'
        subprocess.run(['go', 'build', '-o', str(runner), './cmd/haven-load'],
                       cwd=root / 'crates/haven-web/load-tests/go', check=True)
        with (private / 'server.log').open('w') as log:
            server = subprocess.Popen([str(root / 'target' / profile / 'haven-web')],
                                      cwd=root, env=env, stdout=log, stderr=log)
            for attempt in range(60):
                if server.poll() is not None:
                    raise RuntimeError('journey test server exited before readiness')
                try:
                    with urlopen('http://127.0.0.1:' + port + '/health', timeout=1) as response:
                        if response.status == 200:
                            break
                except OSError:
                    if attempt == 59:
                        raise RuntimeError('journey test server was not ready') from None
                    time.sleep(0.5)
            fixture = private / 'sessions.json'
            cases_path = private / 'cases.json'
            seed_command = [str(root / 'target' / profile / 'load-test-support'),
                            '--users', '10000' if narrowing else '6', '--output', str(fixture)]
            if narrowing:
                seed_command += ['--narrowing', '--cases', str(cases_path)]
            with (private / 'fixture.log').open('w') as seed_log:
                subprocess.run(seed_command, cwd=root, env=env,
                               stdout=seed_log, stderr=seed_log, check=True)
            sessions = json.loads(fixture.read_text())
            truth = cases_path if narrowing else fixture
            result.update(fixture_sha256=hashlib.sha256(truth.read_bytes()).hexdigest(),
                          people=len(sessions))
            command = [str(runner), '--base-url', 'http://127.0.0.1:' + port,
                       '--summary', str(output / 'summary.json')]
            if narrowing:
                cases = json.loads(cases_path.read_text())['cases']
                expected_semantics = {case['id'] for case in cases}
                expected_requests = sum(1 + bool(case.get('session_token')) for case in cases)
                strata = {}
                for case in cases:
                    key = case['class'] + '/' + case['history']
                    strata[key] = strata.get(key, 0) + 1
                result.update(cases=len(cases), strata=strata, expected_requests=expected_requests)
                command += ['--scenario', 'v07', '--narrowing-fixture', str(cases_path),
                            '--query-class', 'mixed', '--fixture-sweep', '--semantic-percent', '100']
            else:
                expected_requests = 68
                command += ['--scenario', 'messaging', '--sessions', str(fixture),
                            '--workers', '2', '--rate', '2', '--duration', '5s', '--iterations', '4']
                result.update(workers=2, iterations=4)
            with (output / 'runner.log').open('w') as runner_log:
                subprocess.run(command, cwd=root, stdout=runner_log, stderr=runner_log, check=True)
            summary = json.loads((output / 'summary.json').read_text())
            if (summary['failed'] or summary['dropped'] or summary['requests'] != expected_requests
                    or summary['checks'] != expected_requests
                    or summary['checks_passed'] != expected_requests):
                raise RuntimeError('journey correctness HTTP counts failed')
            semantic = summary.get('semantic_cases', {})
            if narrowing:
                if (set(semantic) != expected_semantics or any(
                        count['checked'] != 1 or count['passed'] != 1 for count in semantic.values())
                        or summary['completed'] != len(cases)):
                    raise RuntimeError('narrowing sweep omitted or failed a fixture case')
            elif semantic.get('messaging/round-trip') != {'checked': 4, 'passed': 4}:
                raise RuntimeError('messaging semantic counts failed')
            result['status'] = 'passed'
    except Exception:
        result['status'] = 'failed'
        raise
    finally:
        if server is not None:
            server.terminate()
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait()
        report.write_text(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()

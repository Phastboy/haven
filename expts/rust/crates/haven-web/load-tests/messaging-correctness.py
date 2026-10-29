#!/usr/bin/env python3
"""Low-load real-application messaging check; credentials stay in a private directory."""
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
    args = parser.parse_args()
    output, private = Path(args.output).resolve(), Path(args.private).resolve()
    if output == private or output in private.parents or private in output.parents:
        raise ValueError('public evidence and private credentials must be separate directories')
    output.mkdir(parents=True, exist_ok=True)
    private.mkdir(parents=True, exist_ok=True, mode=0o700)
    private.chmod(0o700)
    root = Path(__file__).resolve().parents[3]
    result = {'status': 'incomplete', 'scenario': 'messaging', 'workers': 2,
              'iterations': 4, 'scope': 'low-load correctness only'}
    report = output / 'result.json'
    report.write_text(json.dumps(result, indent=2))
    server = None
    try:
        env = os.environ.copy()
        database = urlparse(env['DATABASE_URL'])
        if database.hostname not in ('localhost', '127.0.0.1', '::1'):
            raise ValueError('messaging correctness requires a local disposable database')
        postgres_env = dict(env, PGHOST=database.hostname, PGPORT=str(database.port or 5432),
                            PGUSER=unquote(database.username or ''),
                            PGPASSWORD=unquote(database.password or ''),
                            PGDATABASE=unquote(database.path.lstrip('/')))
        subprocess.run(['psql', '-X', '-v', 'ON_ERROR_STOP=1', '-c',
                        'CREATE DATABASE haven_messaging_ci'], env=postgres_env, check=True)
        env.update(DATABASE_URL=database._replace(path='/haven_messaging_ci').geturl(),
                   PORT='3341', HOST='127.0.0.1', PUBLIC_BASE_URL='http://127.0.0.1:3341',
                   EMAIL_DELIVERY_ENABLED='false')
        runner = private / 'haven-load'
        subprocess.run(['go', 'build', '-o', str(runner), './cmd/haven-load'],
                       cwd=root / 'crates/haven-web/load-tests/go', check=True)
        with (private / 'server.log').open('w') as log:
            server = subprocess.Popen([str(root / 'target/debug/haven-web')],
                                      cwd=root, env=env, stdout=log, stderr=log)
            for attempt in range(60):
                if server.poll() is not None:
                    raise RuntimeError('messaging test server exited before readiness')
                try:
                    with urlopen('http://127.0.0.1:3341/health', timeout=1) as response:
                        if response.status == 200:
                            break
                except OSError:
                    if attempt == 59:
                        raise RuntimeError('messaging test server was not ready') from None
                    time.sleep(0.5)
            fixture = private / 'sessions.json'
            with (private / 'fixture.log').open('w') as seed_log:
                subprocess.run([str(root / 'target/debug/load-test-support'), '--users', '6',
                                '--output', str(fixture)], cwd=root, env=env,
                               stdout=seed_log, stderr=seed_log, check=True)
            sessions = json.loads(fixture.read_text())
            result.update(fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),
                          people=len(sessions),
                          revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root,
                                                           text=True).strip())
            with (output / 'runner.log').open('w') as runner_log:
                subprocess.run([str(runner), '--base-url', 'http://127.0.0.1:3341',
                                '--scenario', 'messaging', '--sessions', str(fixture),
                                '--workers', '2', '--rate', '2', '--duration', '5s',
                                '--iterations', '4', '--summary', str(output / 'summary.json')],
                               cwd=root, stdout=runner_log, stderr=runner_log, check=True)
            summary = json.loads((output / 'summary.json').read_text())
            semantic = summary.get('semantic_cases', {}).get('messaging/round-trip', {})
            if (summary['failed'] or summary['dropped'] or summary['requests'] != 68
                    or summary['checks'] != 68 or summary['checks_passed'] != 68
                    or semantic.get('checked') != 4 or semantic.get('passed') != 4):
                raise RuntimeError('messaging correctness counts or semantics failed')
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

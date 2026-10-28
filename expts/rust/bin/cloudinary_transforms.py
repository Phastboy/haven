#!/usr/bin/env python3
"""Check/provision Haven named transforms without logging credentials or API bodies."""
import argparse
import base64
import json
import os
from pathlib import Path
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, build_opener, HTTPRedirectHandler

MANIFEST = Path(__file__).with_name('cloudinary_transformations.json')


class ProvisionError(Exception):
    """A redacted, operator-safe provisioning failure."""


class NoRedirects(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ProvisionError('Admin API redirect refused')


class AdminAPI:
    def __init__(self, cloud, key, secret):
        self.base = f'https://api.cloudinary.com/v1_1/{cloud}/transformations/'
        self.authorization = 'Basic ' + base64.b64encode(f'{key}:{secret}'.encode()).decode()
        self.opener = build_opener(NoRedirects())

    def request(self, method, name, fields=None):
        data = None if fields is None else urlencode(fields).encode()
        request = Request(self.base + name, data=data, method=method, headers={
            'Authorization': self.authorization,
            'Content-Type': 'application/x-www-form-urlencoded',
        })
        try:
            with self.opener.open(request, timeout=30) as response:
                return json.load(response)
        except HTTPError as error:
            if method == 'GET' and error.code == 404:
                return None
            raise ProvisionError(f'{method} {name}: HTTP {error.code}') from None
        except (URLError, TimeoutError, ValueError, OSError):
            raise ProvisionError(f'{method} {name}: transport or response failure') from None


def normalize(definition):
    """Ignore parameter order within a stage, preserving chained stage order."""
    aliases = {'c': 'crop', 'w': 'width', 'h': 'height', 'f': 'format', 'g': 'gravity'}
    if isinstance(definition, str):
        stages = []
        for stage in definition.split('/'):
            fields = {}
            for parameter in stage.split(','):
                key, value = parameter.split('_', 1)
                fields[aliases.get(key, key)] = value
            stages.append(fields)
    elif isinstance(definition, list) and all(isinstance(stage, dict) for stage in definition):
        stages = definition
    else:
        raise ProvisionError('Unrecognized transformation definition')
    return tuple(tuple(sorted((key, str(value)) for key, value in stage.items())) for stage in stages)


def matches(actual, expected):
    return normalize(actual.get('info')) == normalize(expected['definition'])


def reconcile(api, manifest, provision=False):
    """Preflight every definition before writing; never overwrite a definition."""
    entries = manifest['transformations']
    states = {name: api.request('GET', name) for name in entries}
    for name, actual in states.items():
        if actual is not None and not matches(actual, entries[name]):
            raise ProvisionError(f'{name}: conflicting definition; no changes applied')
    problems = []
    for name, expected in entries.items():
        actual = states[name]
        if actual is None:
            if not provision:
                problems.append(f'{name}: missing')
                continue
            api.request('POST', name, {'transformation': expected['definition'],
                                      'allowed_for_strict': 'true'})
        elif actual.get('allowed_for_strict') is not True:
            if not provision:
                problems.append(f'{name}: strict allowance missing')
                continue
            api.request('PUT', name, {'allowed_for_strict': 'true'})
        # Re-read after every provision operation and on existing definitions.
        verified = api.request('GET', name) if provision else actual
        if verified is None or not matches(verified, expected) or verified.get('allowed_for_strict') is not True:
            problems.append(f'{name}: verification failed')
    if problems:
        raise ProvisionError('; '.join(problems))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cloud', required=True, help='Explicit target cloud')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--check', action='store_true', help='Read-only (default)')
    mode.add_argument('--provision', action='store_true', help='Create missing names and allow strict delivery')
    args = parser.parse_args()
    try:
        if not re.fullmatch(r'[A-Za-z0-9_-]+', args.cloud):
            raise ProvisionError('Invalid cloud name')
        configured = os.environ.get('CLOUDINARY_CLOUD_NAME')
        if configured and configured != args.cloud:
            raise ProvisionError('Explicit cloud differs from CLOUDINARY_CLOUD_NAME')
        key = os.environ.get('CLOUDINARY_API_KEY')
        secret = os.environ.get('CLOUDINARY_API_SECRET')
        if not key or not secret:
            raise ProvisionError('Export CLOUDINARY_API_KEY and CLOUDINARY_API_SECRET')
        manifest = json.loads(MANIFEST.read_text())
        print(f"cloud={args.cloud} manifest={manifest['revision']} mode={'provision' if args.provision else 'check'}")
        reconcile(AdminAPI(args.cloud, key, secret), manifest, args.provision)
        print('All named definitions and strict allowances verified')
        return 0
    except (ProvisionError, ValueError, OSError) as error:
        print(f'ERROR: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())

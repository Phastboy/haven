#!/usr/bin/env python3
"""Capture exact guest fixture query plans without exporting private fixture truth."""
import json
import os
from pathlib import Path
import subprocess
from urllib.parse import parse_qs, urlsplit
import uuid


def capture(fixture, output):
    cases = json.loads(Path(fixture).read_text())['cases']
    for case in cases:
        if case['class'] != 'public' or case.get('session_token'):
            continue
        if '/free/' not in case['label'] and '/dominant/' not in case['label']:
            continue
        query = parse_qs(urlsplit(case['path']).query)
        variables = ['-v', 'category=' + query['category'][0]]
        where = 'o.image_count >= 1 AND o.category = $1'
        types = ['text']
        values = [":'category'"]
        if 'max_price' in query:
            variables += ['-v', 'currency=' + query['currency'][0], '-v', 'price=' + query['max_price'][0]]
            where += ' AND o.currency = $2 AND o.price <= $3'
            types += ['text', 'integer']
            values += [":'currency'", ":'price'"]
        if 'after' in query:
            micros, ident = bytes.fromhex(query['after'][0]).decode().split('_')
            variables += ['-v', 'micros=' + str(int(micros)), '-v', 'id=' + str(uuid.UUID(ident))]
            n = len(types)
            where += f' AND (o.created_at, o.id) < (${n+1}, ${n+2})'
            types += ['timestamptz', 'uuid']
            values += ["to_timestamp(:'micros'::numeric / 1000000)", ":'id'"]
        types += ['bigint']
        values += ['21']
        sql = f'''SET statement_timeout = '30s';
PREPARE feed ({', '.join(types)}) AS
SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency,
       o.category, o.image_count, o.created_at, o.updated_at, u.display_name, u.handle
FROM offer o JOIN "user" u ON o.user_id = u.id
WHERE {where}
ORDER BY o.created_at DESC, o.id DESC LIMIT ${len(types)};
EXPLAIN (ANALYZE, BUFFERS, SETTINGS, FORMAT JSON) EXECUTE feed ({', '.join(values)});
'''
        result = subprocess.check_output(['psql', os.environ['DATABASE_URL'], '-XAtq',
                                         '-v', 'ON_ERROR_STOP=1', *variables], input=sql, text=True)
        plan = json.loads(result)
        name = case['label'].replace('/', '-')
        (output / f'plan-{name}.json').write_text(json.dumps(plan, indent=2) + '\n')

        if 'max_price' in query:
            equality_sql = sql.replace('o.price <= $3', 'o.price = $3')
            result = subprocess.check_output(['psql', os.environ['DATABASE_URL'], '-XAtq',
                                             '-v', 'ON_ERROR_STOP=1', *variables],
                                            input=equality_sql, text=True)
            (output / f'plan-equality-{name}.json').write_text(
                json.dumps(json.loads(result), indent=2) + '\n')

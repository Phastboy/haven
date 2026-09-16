#!/usr/bin/env -S node
import type { Contract as End } from '../../snapshots/50f2f27de5e65d96ac657ffb4d63a66d4b5162d22d53dc7c8102a9728a7c4de9/contract';
import endContract from '../../snapshots/50f2f27de5e65d96ac657ffb4d63a66d4b5162d22d53dc7c8102a9728a7c4de9/contract.json' with { type: 'json' };
import type { Contract as Start } from '../../snapshots/7a780ece539bf5a3d4fd91740c41ead8b834b183d0e5ae235f01ef78bb73106a/contract';
import startContract from '../../snapshots/7a780ece539bf5a3d4fd91740c41ead8b834b183d0e5ae235f01ef78bb73106a/contract.json' with { type: 'json' };
import { Migration, MigrationCLI, col, fn, primaryKey } from '@prisma/orm-postgres/migration';

export default class M extends Migration<Start, End> {
  override readonly startContractJson = startContract;
  override readonly endContractJson = endContract;

  override get operations() {
    return [
      this.createTable({
        schema: 'public',
        table: 'MagicLink',
        columns: [
          col('createdAt', 'timestamptz', {
            notNull: true,
            default: fn('now()'),
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
          col('email', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('expiresAt', 'timestamp', {
            notNull: true,
            codecRef: { codecId: 'pg/timestamp-string@1' },
          }),
          col('id', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('token', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('usedAt', 'timestamp', { codecRef: { codecId: 'pg/timestamp-string@1' } }),
        ],
        constraints: [primaryKey(['id'])],
      }),
      this.addUnique({
        schema: 'public',
        table: 'MagicLink',
        constraint: 'MagicLink_token_key',
        columns: ['token'],
      }),
    ];
  }
}

MigrationCLI.run(import.meta.url, M);

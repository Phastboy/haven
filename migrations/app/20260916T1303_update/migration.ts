#!/usr/bin/env -S node
import type { Contract as Start } from '../../snapshots/bd6cd4febdccf911cbc0230a0ee66d1eeb16a0f11a9a95931dbe32fc35a3feeb/contract';
import startContract from '../../snapshots/bd6cd4febdccf911cbc0230a0ee66d1eeb16a0f11a9a95931dbe32fc35a3feeb/contract.json' with { type: 'json' };
import type { Contract as End } from '../../snapshots/f9729875259238fb4bdc864070c1c8a0d941002f02edcc70e739c05e256c6558/contract';
import endContract from '../../snapshots/f9729875259238fb4bdc864070c1c8a0d941002f02edcc70e739c05e256c6558/contract.json' with { type: 'json' };
import { Migration, MigrationCLI, col, fn, lit, primaryKey } from '@prisma/orm-postgres/migration';

export default class M extends Migration<Start, End> {
  override readonly startContractJson = startContract;
  override readonly endContractJson = endContract;

  override get operations() {
    return [
      this.createTable({
        schema: 'public',
        table: 'Account',
        columns: [
          col('createdAt', 'timestamptz', {
            notNull: true,
            default: fn('now()'),
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
          col('email', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('emailVerified', 'bool', {
            notNull: true,
            default: lit(false),
            codecRef: { codecId: 'pg/bool@1' },
          }),
          col('id', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('updatedAt', 'timestamptz', {
            notNull: true,
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
        ],
        constraints: [primaryKey(['id'])],
      }),
      this.createTable({
        schema: 'public',
        table: 'AccountPlatformLink',
        columns: [
          col('accountId', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('createdAt', 'timestamptz', {
            notNull: true,
            default: fn('now()'),
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
          col('id', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('platform', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('platformUserId', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
        ],
        constraints: [primaryKey(['id'])],
      }),
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
      this.createTable({
        schema: 'public',
        table: 'OAuthCredential',
        columns: [
          col('accessToken', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('accountId', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('createdAt', 'timestamptz', {
            notNull: true,
            default: fn('now()'),
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
          col('id', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('provider', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('providerUserId', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('refreshToken', 'text', { codecRef: { codecId: 'pg/text@1' } }),
          col('tokenExpiresAt', 'timestamp', { codecRef: { codecId: 'pg/timestamp-string@1' } }),
          col('updatedAt', 'timestamptz', {
            notNull: true,
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
        ],
        constraints: [primaryKey(['id'])],
      }),
      this.createTable({
        schema: 'public',
        table: 'Session',
        columns: [
          col('accountId', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('createdAt', 'timestamptz', {
            notNull: true,
            default: fn('now()'),
            codecRef: { codecId: 'pg/timestamptz-string@1' },
          }),
          col('expiresAt', 'timestamp', {
            notNull: true,
            codecRef: { codecId: 'pg/timestamp-string@1' },
          }),
          col('id', 'character(36)', {
            notNull: true,
            codecRef: { codecId: 'sql/char@1', typeParams: { length: 36 } },
          }),
          col('ipAddress', 'text', { codecRef: { codecId: 'pg/text@1' } }),
          col('token', 'text', { notNull: true, codecRef: { codecId: 'pg/text@1' } }),
          col('userAgent', 'text', { codecRef: { codecId: 'pg/text@1' } }),
        ],
        constraints: [primaryKey(['id'])],
      }),
      this.addUnique({
        schema: 'public',
        table: 'Account',
        constraint: 'Account_email_key',
        columns: ['email'],
      }),
      this.addUnique({
        schema: 'public',
        table: 'MagicLink',
        constraint: 'MagicLink_token_key',
        columns: ['token'],
      }),
      this.addUnique({
        schema: 'public',
        table: 'Session',
        constraint: 'Session_token_key',
        columns: ['token'],
      }),
      this.createIndex({
        schema: 'public',
        table: 'AccountPlatformLink',
        index: 'account_platform_link_unique_9a628f2f',
        columns: ['accountId', 'platform'],
        extras: { unique: true },
      }),
    ];
  }
}

MigrationCLI.run(import.meta.url, M);

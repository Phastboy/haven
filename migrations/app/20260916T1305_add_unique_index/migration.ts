#!/usr/bin/env -S node
import type { Contract as Start } from '../../snapshots/50f2f27de5e65d96ac657ffb4d63a66d4b5162d22d53dc7c8102a9728a7c4de9/contract';
import startContract from '../../snapshots/50f2f27de5e65d96ac657ffb4d63a66d4b5162d22d53dc7c8102a9728a7c4de9/contract.json' with { type: 'json' };
import type { Contract as End } from '../../snapshots/f9729875259238fb4bdc864070c1c8a0d941002f02edcc70e739c05e256c6558/contract';
import endContract from '../../snapshots/f9729875259238fb4bdc864070c1c8a0d941002f02edcc70e739c05e256c6558/contract.json' with { type: 'json' };
import { Migration, MigrationCLI } from '@prisma/orm-postgres/migration';

export default class M extends Migration<Start, End> {
  override readonly startContractJson = startContract;
  override readonly endContractJson = endContract;

  override get operations() {
    return [
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

import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../prisma/db';
import { CreateAccountPlatformLinkDTO, IAccountPlatformLinkRepository } from '../../domain/ports/IAccountPlatformLinkRepository';
import { AccountPlatformLink } from '../../domain/account-platform-link.schema';

export class AccountPlatformLinkRepository implements IAccountPlatformLinkRepository {
  async create(data: CreateAccountPlatformLinkDTO): Promise<AccountPlatformLink> {
    return await db.orm.public.AccountPlatformLink.create({ ...data, accountId: asId(data.accountId) });
  }

  async findByAccountAndPlatform(accountId: string, platform: string): Promise<AccountPlatformLink | null> {
    return await db.orm.public.AccountPlatformLink.where({ accountId: asId(accountId), platform }).first();
  }
}

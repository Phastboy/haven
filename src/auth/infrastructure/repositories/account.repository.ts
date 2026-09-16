import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../prisma/db';
import { IAccountRepository } from '../../domain/ports/IAccountRepository';
import { Account, CreateAccountDTO } from '../../domain/account.schema';

export class AccountRepository implements IAccountRepository {
  async create(data: CreateAccountDTO): Promise<Account> {
    return await db.orm.public.Account.create({
      id: asId(crypto.randomUUID()),
      email: data.email,
      emailVerified: data.emailVerified,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });
  }

  async findById(id: string): Promise<Account | null> {
    return await db.orm.public.Account.first({ id: asId(id) });
  }

  async findByEmail(email: string): Promise<Account | null> {
    return await db.orm.public.Account.where({ email }).first();
  }

  async markEmailVerified(id: string): Promise<void> {
    await db.orm.public.Account.where({ id: asId(id) }).update({
      emailVerified: true,
      updatedAt: new Date().toISOString(),
    });
  }
}

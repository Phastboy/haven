import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../prisma/db';
import { CreateMagicLinkDTO, IMagicLinkRepository } from '../../domain/ports/IMagicLinkRepository';
import { MagicLink } from '../../domain/magic-link.schema';

export class MagicLinkRepository implements IMagicLinkRepository {
  async create(data: CreateMagicLinkDTO): Promise<MagicLink> {
    const row = await db.orm.public.MagicLink.create(data);
    const { usedAt, ...rest } = row;
    return { ...rest, ...(usedAt ? { usedAt } : {}) };
  }

  async findByToken(token: string): Promise<MagicLink | null> {
    const row = await db.orm.public.MagicLink.where({ token }).first();
    if (!row) return null;
    const { usedAt, ...rest } = row;
    return { ...rest, ...(usedAt ? { usedAt } : {}) };
  }

  async markUsed(id: string, usedAt: string): Promise<void> {
    await db.orm.public.MagicLink.where({ id: asId(id) }).update({ usedAt });
  }
}

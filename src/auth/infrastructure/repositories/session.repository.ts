import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../prisma/db';
import { CreateSessionDTO, ISessionRepository } from '../../domain/ports/ISessionRepository';
import { Session, SessionWithAccount } from '../../domain/session.schema';

export class SessionRepository implements ISessionRepository {
  async create(data: CreateSessionDTO): Promise<Session> {
    const row = await db.orm.public.Session.create({
      ...data,
      accountId: asId(data.accountId),
      ...(data.userAgent ? { userAgent: data.userAgent } : {}),
      ...(data.ipAddress ? { ipAddress: data.ipAddress } : {}),
      // Prisma handles uuid and createdAt
    });
    const { userAgent, ipAddress, ...rest } = row;
    return {
      ...rest,
      ...(userAgent ? { userAgent } : {}),
      ...(ipAddress ? { ipAddress } : {}),
    };
  }

  async findByToken(token: string): Promise<SessionWithAccount | null> {
    const row = await db.orm.public.Session.where({ token }).include('account').first();
    if (!row) return null;
    const { userAgent, ipAddress, ...rest } = row;
    return {
      ...rest,
      ...(userAgent ? { userAgent } : {}),
      ...(ipAddress ? { ipAddress } : {}),
      account: {
        ...rest.account,
      },
    };
  }

  async deleteByToken(token: string): Promise<void> {
    await db.orm.public.Session.where({ token }).delete();
  }

  async deleteExpired(): Promise<void> {
    const now = new Date().toISOString();
    await db.orm.public.Session.where((s) => s.expiresAt.lt(now)).delete();
  }
}

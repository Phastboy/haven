import { db } from '../../../database/db';
import { sessions } from '../../../database/schema';
import { eq, lt } from 'drizzle-orm';
import { toIso } from '../../../database/map';
import { CreateSessionDTO, ISessionRepository } from '../../domain/ports/ISessionRepository';
import { Session, SessionWithAccount } from '../../domain/session.schema';

export class SessionRepository implements ISessionRepository {
  async create(data: CreateSessionDTO): Promise<Session> {
    const id = crypto.randomUUID();
    const rows = await db.insert(sessions).values({
      id,
      accountId: data.accountId,
      token: data.token,
      expiresAt: new Date(data.expiresAt),
      userAgent: data.userAgent ?? null,
      ipAddress: data.ipAddress ?? null,
    }).returning();
    return this.#toSession(rows[0]!);
  }

  async findByToken(token: string): Promise<SessionWithAccount | null> {
    const row = await db.query.sessions.findFirst({
      where: eq(sessions.token, token),
      with: {
        account: true,
      },
    });

    if (!row) return null;

    return {
      ...this.#toSession(row),
      account: {
        id: row.account.id,
        email: row.account.email,
        emailVerified: row.account.emailVerified,
        createdAt: toIso(row.account.createdAt),
        updatedAt: toIso(row.account.updatedAt),
      }
    };
  }

  async deleteByToken(token: string): Promise<void> {
    await db.delete(sessions).where(eq(sessions.token, token));
  }

  async deleteExpired(): Promise<void> {
    await db.delete(sessions).where(lt(sessions.expiresAt, new Date()));
  }

  #toSession(row: typeof sessions.$inferSelect): Session {
    return {
      id: row.id,
      accountId: row.accountId,
      token: row.token,
      expiresAt: toIso(row.expiresAt),
      createdAt: toIso(row.createdAt),
      ...(row.userAgent ? { userAgent: row.userAgent } : {}),
      ...(row.ipAddress ? { ipAddress: row.ipAddress } : {}),
    };
  }
}

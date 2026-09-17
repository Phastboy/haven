import { db } from '../../../database/db';
import { CreateSessionDTO, ISessionRepository } from '../../domain/ports/ISessionRepository';
import { Session, SessionWithAccount } from '../../domain/session.schema';

export class SessionRepository implements ISessionRepository {
  async create(data: CreateSessionDTO): Promise<Session> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "Session" (id, "accountId", token, "expiresAt", "userAgent", "ipAddress")
      VALUES (${id}, ${data.accountId}, ${data.token}, ${data.expiresAt}, ${data.userAgent || null}, ${data.ipAddress || null})
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async findByToken(token: string): Promise<SessionWithAccount | null> {
    const rows = await db`
      SELECT s.*, 
             a.id as a_id, a.email as a_email, a."emailVerified" as "a_emailVerified", 
             a."createdAt" as "a_createdAt", a."updatedAt" as "a_updatedAt"
      FROM "Session" s
      JOIN "Account" a ON s."accountId" = a.id
      WHERE s.token = ${token}
    `;
    if (rows.length === 0) return null;
    
    const row = rows[0];
    const session = this.toEntity(row);
    return {
      ...session,
      account: {
        id: row.a_id,
        email: row.a_email,
        emailVerified: row.a_emailVerified,
        createdAt: row.a_createdAt instanceof Date ? row.a_createdAt.toISOString() : row.a_createdAt,
        updatedAt: row.a_updatedAt instanceof Date ? row.a_updatedAt.toISOString() : row.a_updatedAt,
      }
    };
  }

  async deleteByToken(token: string): Promise<void> {
    await db`DELETE FROM "Session" WHERE token = ${token}`;
  }

  async deleteExpired(): Promise<void> {
    const now = new Date().toISOString();
    await db`DELETE FROM "Session" WHERE "expiresAt" < ${now}`;
  }

  private toEntity(row: any): Session {
    const res: any = {
      id: row.id,
      accountId: row.accountId,
      token: row.token,
      expiresAt: row.expiresAt instanceof Date ? row.expiresAt.toISOString() : row.expiresAt,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
    };
    if (row.userAgent) res.userAgent = row.userAgent;
    if (row.ipAddress) res.ipAddress = row.ipAddress;
    return res;
  }
}

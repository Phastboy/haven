import type { DB } from "../../../database/db";
import { sessions, accounts, users } from "../../../database/schema";
import { eq, lt } from "drizzle-orm";
import { toIso } from "../../../database/map";
import type { CreateSessionDTO, ISessionRepository } from "../../domain/ports/ISessionRepository";
import type { Session, SessionContext } from "../../domain/session.schema";

export class SessionRepository implements ISessionRepository {
  readonly #db: DB;

  constructor(db: DB) {
    this.#db = db;
  }

  async create(data: CreateSessionDTO): Promise<Session> {
    const id = crypto.randomUUID();
    const rows = await this.#db
      .insert(sessions)
      .values({
        id,
        accountId: data.accountId,
        token: data.token,
        expiresAt: new Date(data.expiresAt),
        userAgent: data.userAgent ?? null,
        ipAddress: data.ipAddress ?? null,
      })
      .returning();
    return this.#toSession(rows[0]!);
  }

  async findByToken(token: string): Promise<SessionContext | null> {
    const rows = await this.#db
      .select({
        session: sessions,
        account: accounts,
        user: users,
      })
      .from(sessions)
      .innerJoin(accounts, eq(sessions.accountId, accounts.id))
      .leftJoin(users, eq(users.accountId, accounts.id))
      .where(eq(sessions.token, token))
      .limit(1);

    const row = rows[0];
    if (!row) return null;

    return {
      ...this.#toSession(row.session),
      account: {
        id: row.account.id,
        email: row.account.email,
        emailVerified: row.account.emailVerified,
        createdAt: toIso(row.account.createdAt),
        updatedAt: toIso(row.account.updatedAt),
      },
      user: row.user
        ? {
            id: row.user.id,
            accountId: row.user.accountId,
            username: row.user.username ?? `user_${row.user.id.replace(/-/g, "").substring(0, 10)}`,
            name: row.user.name,
            bio: row.user.bio,
            profilePictureUrl: row.user.profilePictureUrl,
            profileComplete: row.user.name !== null,
            createdAt: toIso(row.user.createdAt),
            updatedAt: toIso(row.user.updatedAt),
          }
        : null,
    };
  }

  async deleteByToken(token: string): Promise<void> {
    await this.#db.delete(sessions).where(eq(sessions.token, token));
  }

  async deleteExpired(): Promise<void> {
    await this.#db.delete(sessions).where(lt(sessions.expiresAt, new Date()));
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

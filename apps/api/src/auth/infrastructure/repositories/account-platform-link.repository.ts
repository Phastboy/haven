import { db } from '../../../database/db';
import { CreateAccountPlatformLinkDTO, IAccountPlatformLinkRepository } from '../../domain/ports/IAccountPlatformLinkRepository';
import { AccountPlatformLink } from '../../domain/account-platform-link.schema';

export class AccountPlatformLinkRepository implements IAccountPlatformLinkRepository {
  async create(data: CreateAccountPlatformLinkDTO): Promise<AccountPlatformLink> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "AccountPlatformLink" (id, "accountId", "platformUserId", platform)
      VALUES (${id}, ${data.accountId}, ${data.platformUserId}, ${data.platform})
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async findByAccountAndPlatform(accountId: string, platform: string): Promise<AccountPlatformLink | null> {
    const rows = await db`
      SELECT * FROM "AccountPlatformLink"
      WHERE "accountId" = ${accountId} AND platform = ${platform}
    `;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  private toEntity(row: any): AccountPlatformLink {
    return {
      id: row.id,
      accountId: row.accountId,
      platformUserId: row.platformUserId,
      platform: row.platform,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
    };
  }
}

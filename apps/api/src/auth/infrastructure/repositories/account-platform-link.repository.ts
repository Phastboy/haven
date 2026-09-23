import { db } from "../../../database/db";
import { accountPlatformLinks } from "../../../database/schema";
import { eq, and } from "drizzle-orm";
import { toIso } from "../../../database/map";
import type {
  CreateAccountPlatformLinkDTO,
  IAccountPlatformLinkRepository,
} from "../../domain/ports/IAccountPlatformLinkRepository";
import type { AccountPlatformLink } from "../../domain/account-platform-link.schema";

export class AccountPlatformLinkRepository implements IAccountPlatformLinkRepository {
  async create(data: CreateAccountPlatformLinkDTO): Promise<AccountPlatformLink> {
    const id = crypto.randomUUID();
    const rows = await db
      .insert(accountPlatformLinks)
      .values({
        id,
        accountId: data.accountId,
        platformUserId: data.platformUserId,
        platform: data.platform,
      })
      .returning();
    return this.#toAccountPlatformLink(rows[0]!);
  }

  async findByAccountAndPlatform(
    accountId: string,
    platform: string,
  ): Promise<AccountPlatformLink | null> {
    const row = await db.query.accountPlatformLinks.findFirst({
      where: and(
        eq(accountPlatformLinks.accountId, accountId),
        eq(accountPlatformLinks.platform, platform),
      ),
    });
    return row ? this.#toAccountPlatformLink(row) : null;
  }

  #toAccountPlatformLink(row: typeof accountPlatformLinks.$inferSelect): AccountPlatformLink {
    return {
      id: row.id,
      accountId: row.accountId,
      platformUserId: row.platformUserId,
      platform: row.platform,
      createdAt: toIso(row.createdAt),
    };
  }
}

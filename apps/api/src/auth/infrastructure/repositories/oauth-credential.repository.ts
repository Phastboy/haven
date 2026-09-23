import { db } from "../../../database/db";
import { oauthCredentials } from "../../../database/schema";
import { eq, and } from "drizzle-orm";
import { toIso } from "../../../database/map";
import {
  CreateOAuthCredentialDTO,
  IOAuthCredentialRepository,
} from "../../domain/ports/IOAuthCredentialRepository";
import { OAuthCredential } from "../../domain/oauth-credential.schema";

export class OAuthCredentialRepository implements IOAuthCredentialRepository {
  async create(data: CreateOAuthCredentialDTO): Promise<OAuthCredential> {
    const id = crypto.randomUUID();
    const rows = await db
      .insert(oauthCredentials)
      .values({
        id,
        accountId: data.accountId,
        provider: data.provider,
        providerUserId: data.providerUserId,
        accessToken: data.accessToken,
        refreshToken: data.refreshToken ?? null,
        tokenExpiresAt: data.tokenExpiresAt ? new Date(data.tokenExpiresAt) : null,
      })
      .returning();
    return this.#toOAuthCredential(rows[0]!);
  }

  async findByProvider(provider: string, providerUserId: string): Promise<OAuthCredential | null> {
    const row = await db.query.oauthCredentials.findFirst({
      where: and(
        eq(oauthCredentials.provider, provider),
        eq(oauthCredentials.providerUserId, providerUserId),
      ),
    });
    return row ? this.#toOAuthCredential(row) : null;
  }

  async updateTokens(
    id: string,
    accessToken: string,
    refreshToken?: string,
    tokenExpiresAt?: string,
  ): Promise<void> {
    const updates: Partial<typeof oauthCredentials.$inferInsert> = {
      accessToken,
      updatedAt: new Date(),
    };

    if (refreshToken !== undefined) {
      updates.refreshToken = refreshToken;
    }
    if (tokenExpiresAt !== undefined) {
      updates.tokenExpiresAt = tokenExpiresAt ? new Date(tokenExpiresAt) : null;
    }

    await db.update(oauthCredentials).set(updates).where(eq(oauthCredentials.id, id));
  }

  #toOAuthCredential(row: typeof oauthCredentials.$inferSelect): OAuthCredential {
    return {
      id: row.id,
      accountId: row.accountId,
      provider: row.provider as "GOOGLE",
      providerUserId: row.providerUserId,
      accessToken: row.accessToken,
      createdAt: toIso(row.createdAt),
      updatedAt: toIso(row.updatedAt),
      ...(row.refreshToken ? { refreshToken: row.refreshToken } : {}),
      ...(row.tokenExpiresAt ? { tokenExpiresAt: toIso(row.tokenExpiresAt) } : {}),
    };
  }
}

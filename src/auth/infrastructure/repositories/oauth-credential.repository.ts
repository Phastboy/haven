import { db } from '../../../database/db';
import { CreateOAuthCredentialDTO, IOAuthCredentialRepository } from '../../domain/ports/IOAuthCredentialRepository';
import { OAuthCredential } from '../../domain/oauth-credential.schema';

export class OAuthCredentialRepository implements IOAuthCredentialRepository {
  async create(data: CreateOAuthCredentialDTO): Promise<OAuthCredential> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "OAuthCredential" (id, "accountId", provider, "providerUserId", "accessToken", "refreshToken", "tokenExpiresAt")
      VALUES (${id}, ${data.accountId}, ${data.provider}, ${data.providerUserId}, ${data.accessToken}, ${data.refreshToken || null}, ${data.tokenExpiresAt || null})
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async findByProvider(provider: string, providerUserId: string): Promise<OAuthCredential | null> {
    const rows = await db`
      SELECT * FROM "OAuthCredential"
      WHERE provider = ${provider} AND "providerUserId" = ${providerUserId}
    `;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async updateTokens(id: string, accessToken: string, refreshToken?: string, tokenExpiresAt?: string): Promise<void> {
    const values: any[] = [accessToken, id];
    let query = 'UPDATE "OAuthCredential" SET "accessToken" = $1, "updatedAt" = CURRENT_TIMESTAMP';
    
    let i = 3;
    if (refreshToken !== undefined) {
      query += `, "refreshToken" = $${i++}`;
      values.push(refreshToken);
    }
    if (tokenExpiresAt !== undefined) {
      query += `, "tokenExpiresAt" = $${i++}`;
      values.push(tokenExpiresAt);
    }
    
    query += ' WHERE id = $2';
    
    await db.unsafe(query, values);
  }

  private toEntity(row: any): OAuthCredential {
    const res: any = {
      id: row.id,
      accountId: row.accountId,
      provider: row.provider as "GOOGLE",
      providerUserId: row.providerUserId,
      accessToken: row.accessToken,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
      updatedAt: row.updatedAt instanceof Date ? row.updatedAt.toISOString() : row.updatedAt,
    };
    if (row.refreshToken) res.refreshToken = row.refreshToken;
    if (row.tokenExpiresAt) res.tokenExpiresAt = row.tokenExpiresAt instanceof Date ? row.tokenExpiresAt.toISOString() : row.tokenExpiresAt;
    return res;
  }
}

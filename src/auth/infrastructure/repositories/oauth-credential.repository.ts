import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../prisma/db';
import { CreateOAuthCredentialDTO, IOAuthCredentialRepository } from '../../domain/ports/IOAuthCredentialRepository';
import { OAuthCredential } from '../../domain/oauth-credential.schema';

export class OAuthCredentialRepository implements IOAuthCredentialRepository {
  async create(data: CreateOAuthCredentialDTO): Promise<OAuthCredential> {
    const row = await db.orm.public.OAuthCredential.create({
      ...data,
      accountId: asId(data.accountId),
      provider: data.provider as "GOOGLE",
      updatedAt: new Date().toISOString(),
    });
    const { refreshToken, tokenExpiresAt, ...rest } = row;
    return {
      ...rest,
      provider: rest.provider as "GOOGLE",
      ...(refreshToken ? { refreshToken } : {}),
      ...(tokenExpiresAt ? { tokenExpiresAt } : {}),
    };
  }

  async findByProvider(provider: string, providerUserId: string): Promise<OAuthCredential | null> {
    const row = await db.orm.public.OAuthCredential.where({ provider: provider as "GOOGLE", providerUserId }).first();
    if (!row) return null;
    const { refreshToken, tokenExpiresAt, ...rest } = row;
    return {
      ...rest,
      provider: rest.provider as "GOOGLE",
      ...(refreshToken ? { refreshToken } : {}),
      ...(tokenExpiresAt ? { tokenExpiresAt } : {}),
    };
  }

  async updateTokens(id: string, accessToken: string, refreshToken?: string, tokenExpiresAt?: string): Promise<void> {
    await db.orm.public.OAuthCredential.where({ id: asId(id) }).update({
      accessToken,
      ...(refreshToken ? { refreshToken } : {}),
      ...(tokenExpiresAt ? { tokenExpiresAt } : {}),
      updatedAt: new Date().toISOString(),
    });
  }
}

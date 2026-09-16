import { expect, test, describe } from 'bun:test';
import { OAuthCredentialRepository } from '../oauth-credential.repository';
import { AccountRepository } from '../account.repository';
import type { Char } from '@prisma/orm-postgres/target/codec-types';
import { db } from '../../../../prisma/db';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

describe('OAuthCredentialRepository Integration', () => {
  const oauthRepo = new OAuthCredentialRepository();
  const accountRepo = new AccountRepository();
  
  test('should create an oauth credential', async () => {
    const account = await accountRepo.create({
      email: `test-oauth-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const providerUserId = `google-${crypto.randomUUID()}`;
    const credential = await oauthRepo.create({
      accountId: account.id,
      provider: 'GOOGLE',
      providerUserId,
      accessToken: 'initial-access-token',
    });

    expect(credential.id).toBeDefined();
    expect(credential.accountId).toBe(account.id);
    expect(credential.providerUserId).toBe(providerUserId);

    await db.orm.public.OAuthCredential.where({ id: asId(credential.id) }).delete();
    await db.orm.public.Account.where({ id: asId(account.id) }).delete();
  });

  test('should find oauth credential by provider and providerUserId', async () => {
    const account = await accountRepo.create({
      email: `test-oauth-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const providerUserId = `google-${crypto.randomUUID()}`;
    const credential = await oauthRepo.create({
      accountId: account.id,
      provider: 'GOOGLE',
      providerUserId,
      accessToken: 'initial-access-token',
    });

    const found = await oauthRepo.findByProvider('GOOGLE', providerUserId);
    expect(found).not.toBeNull();
    expect(found!.id).toBe(credential.id);

    await db.orm.public.OAuthCredential.where({ id: asId(credential.id) }).delete();
    await db.orm.public.Account.where({ id: asId(account.id) }).delete();
  });

  test('should return null for non-existent credential', async () => {
    const credential = await oauthRepo.findByProvider('GOOGLE', 'fake-id');
    expect(credential).toBeNull();
  });

  test('should update tokens', async () => {
    const account = await accountRepo.create({
      email: `test-oauth-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const providerUserId = `google-${crypto.randomUUID()}`;
    const credential = await oauthRepo.create({
      accountId: account.id,
      provider: 'GOOGLE',
      providerUserId,
      accessToken: 'initial-access-token',
    });

    await oauthRepo.updateTokens(credential.id, 'new-access-token', 'new-refresh-token');
    
    const updated = await oauthRepo.findByProvider('GOOGLE', providerUserId);
    expect(updated!.accessToken).toBe('new-access-token');
    expect(updated!.refreshToken).toBe('new-refresh-token');

    await db.orm.public.OAuthCredential.where({ id: asId(credential.id) }).delete();
    await db.orm.public.Account.where({ id: asId(account.id) }).delete();
  });
});

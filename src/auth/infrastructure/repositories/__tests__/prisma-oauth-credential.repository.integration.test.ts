import { expect, test, describe, afterAll, beforeAll } from 'bun:test';
import { OAuthCredentialRepository } from '../oauth-credential.repository';
import { AccountRepository } from '../account.repository';
import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../../prisma/db';

describe('OAuthCredentialRepository Integration', () => {
  const oauthRepo = new OAuthCredentialRepository();
  const accountRepo = new AccountRepository();
  
  const testEmail = `test-oauth-${crypto.randomUUID()}@example.com`;
  const providerUserId = `google-${crypto.randomUUID()}`;
  let accountId: string;
  let credentialId: string;

  beforeAll(async () => {
    const account = await accountRepo.create({
      email: testEmail,
      emailVerified: true,
    });
    accountId = account.id;
  });

  afterAll(async () => {
    if (accountId) {
      await db.orm.public.OAuthCredential.where({ accountId: asId(accountId) }).delete();
      await db.orm.public.Account.where({ id: asId(accountId) }).delete();
    }
  });

  test('should create an oauth credential', async () => {
    const credential = await oauthRepo.create({
      accountId,
      provider: 'GOOGLE',
      providerUserId,
      accessToken: 'initial-access-token',
    });

    expect(credential.id).toBeDefined();
    expect(credential.accountId).toBe(accountId);
    expect(credential.providerUserId).toBe(providerUserId);

    credentialId = credential.id;
  });

  test('should find oauth credential by provider and providerUserId', async () => {
    const credential = await oauthRepo.findByProvider('GOOGLE', providerUserId);
    expect(credential).not.toBeNull();
    expect(credential!.id).toBe(credentialId);
  });

  test('should return null for non-existent credential', async () => {
    const credential = await oauthRepo.findByProvider('GOOGLE', 'fake-id');
    expect(credential).toBeNull();
  });

  test('should update tokens', async () => {
    await oauthRepo.updateTokens(credentialId, 'new-access-token', 'new-refresh-token');
    
    const credential = await oauthRepo.findByProvider('GOOGLE', providerUserId);
    expect(credential!.accessToken).toBe('new-access-token');
    expect(credential!.refreshToken).toBe('new-refresh-token');
  });
});

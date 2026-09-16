import { expect, test, describe, afterAll, beforeAll } from 'bun:test';
import { AccountPlatformLinkRepository } from '../account-platform-link.repository';
import { AccountRepository } from '../account.repository';
import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../../prisma/db';

describe('AccountPlatformLinkRepository Integration', () => {
  const linkRepo = new AccountPlatformLinkRepository();
  const accountRepo = new AccountRepository();
  
  const testEmail = `test-link-${crypto.randomUUID()}@example.com`;
  const platformUserId = `platform-user-${crypto.randomUUID()}`;
  let accountId: string;

  beforeAll(async () => {
    const account = await accountRepo.create({
      email: testEmail,
      emailVerified: true,
    });
    accountId = account.id;
  });

  afterAll(async () => {
    if (accountId) {
      await db.orm.public.AccountPlatformLink.where({ accountId: asId(accountId) }).delete();
      await db.orm.public.Account.where({ id: asId(accountId) }).delete();
    }
  });

  test('should create an account platform link', async () => {
    const link = await linkRepo.create({
      accountId,
      platformUserId,
      platform: 'haven',
    });

    expect(link.id).toBeDefined();
    expect(link.accountId).toBe(accountId);
    expect(link.platformUserId).toBe(platformUserId);
    expect(link.platform).toBe('haven');
  });

  test('should find link by account and platform', async () => {
    const link = await linkRepo.findByAccountAndPlatform(accountId, 'haven');
    expect(link).not.toBeNull();
    expect(link!.platformUserId).toBe(platformUserId);
  });

  test('should enforce unique constraint on (accountId, platform)', async () => {
    // Attempting to create a second link for the same platform should throw
    try {
      await linkRepo.create({
        accountId,
        platformUserId: 'another-user',
        platform: 'haven',
      });
      expect(true).toBe(false); // Should not reach here
    } catch (error: any) {
      // Just assert that an error was thrown (Prisma Unique Constraint Violation is typically P2002)
      expect(error.code === 'P2002' || error.message.includes('Unique constraint') || error.message.includes('duplicate')).toBeTruthy();
    }
  });

  test('should allow a different platform for the same account', async () => {
    const link = await linkRepo.create({
      accountId,
      platformUserId: 'another-user-external',
      platform: 'external-app',
    });
    
    expect(link.platform).toBe('external-app');
  });
});

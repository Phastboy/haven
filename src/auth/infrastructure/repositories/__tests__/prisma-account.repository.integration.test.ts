import { expect, test, describe, afterAll } from 'bun:test';
import { AccountRepository } from '../account.repository';
import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../../prisma/db';

describe('AccountRepository Integration', () => {
  const repo = new AccountRepository();
  const testEmail = `test-${crypto.randomUUID()}@example.com`;
  let createdAccountId: string;

  afterAll(async () => {
    if (createdAccountId) {
      await db.orm.public.Account.where({ id: asId(createdAccountId) }).delete();
    }
  });

  test('should create a new account', async () => {
    const account = await repo.create({
      email: testEmail,
      emailVerified: false,
    });

    expect(account.id).toBeDefined();
    expect(account.email).toBe(testEmail);
    expect(account.emailVerified).toBe(false);

    createdAccountId = account.id;
  });

  test('should find account by email', async () => {
    const account = await repo.findByEmail(testEmail);
    expect(account).not.toBeNull();
    expect(account!.id).toBe(createdAccountId);
  });

  test('should return null for non-existent email', async () => {
    const account = await repo.findByEmail('does-not-exist@example.com');
    expect(account).toBeNull();
  });

  test('should find account by id', async () => {
    const account = await repo.findById(createdAccountId);
    expect(account).not.toBeNull();
    expect(account!.email).toBe(testEmail);
  });

  test('should mark email as verified', async () => {
    await repo.markEmailVerified(createdAccountId);
    const account = await repo.findById(createdAccountId);
    expect(account!.emailVerified).toBe(true);
  });
});

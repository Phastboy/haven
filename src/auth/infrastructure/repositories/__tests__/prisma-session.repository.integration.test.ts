import { expect, test, describe, afterAll, beforeAll } from 'bun:test';
import { SessionRepository } from '../session.repository';
import { AccountRepository } from '../account.repository';
import type { Char } from '@prisma/orm-postgres/target/codec-types';

function asId(id: string): Char<36> {
  return id as unknown as Char<36>;
}

import { db } from '../../../../prisma/db';

describe('SessionRepository Integration', () => {
  const sessionRepo = new SessionRepository();
  const accountRepo = new AccountRepository();
  
  const testEmail = `test-session-${crypto.randomUUID()}@example.com`;
  let accountId: string;
  let testToken = crypto.randomUUID();

  beforeAll(async () => {
    const account = await accountRepo.create({
      email: testEmail,
      emailVerified: true,
    });
    accountId = account.id;
  });

  afterAll(async () => {
    if (accountId) {
      await db.orm.public.Session.where({ accountId: asId(accountId) }).delete();
      await db.orm.public.Account.where({ id: asId(accountId) }).delete();
    }
  });

  test('should create a session', async () => {
    const expiresAt = new Date(Date.now() + 100000).toISOString();
    const session = await sessionRepo.create({
      accountId,
      token: testToken,
      expiresAt,
      userAgent: 'integration-test',
      ipAddress: '127.0.0.1'
    });

    expect(session.id).toBeDefined();
    expect(session.accountId).toBe(accountId);
    expect(session.token).toBe(testToken);
  });

  test('should find session by token with account included', async () => {
    const session = await sessionRepo.findByToken(testToken);
    expect(session).not.toBeNull();
    expect(session!.token).toBe(testToken);
    expect(session!.account).toBeDefined();
    expect(session!.account.email).toBe(testEmail);
  });

  test('should delete session by token', async () => {
    await sessionRepo.deleteByToken(testToken);
    const session = await sessionRepo.findByToken(testToken);
    expect(session).toBeNull();
  });

  test('should delete expired sessions', async () => {
    // Create an expired session
    const expiredToken = crypto.randomUUID();
    const pastDate = new Date(Date.now() - 100000).toISOString();
    
    await sessionRepo.create({
      accountId,
      token: expiredToken,
      expiresAt: pastDate
    });

    // Create a valid session
    const validToken = crypto.randomUUID();
    const futureDate = new Date(Date.now() + 100000).toISOString();
    
    await sessionRepo.create({
      accountId,
      token: validToken,
      expiresAt: futureDate
    });

    await sessionRepo.deleteExpired();

    const expiredSession = await sessionRepo.findByToken(expiredToken);
    const validSession = await sessionRepo.findByToken(validToken);

    expect(expiredSession).toBeNull();
    expect(validSession).not.toBeNull();

    await sessionRepo.deleteByToken(validToken);
  });
});

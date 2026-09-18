// oxlint-disable typescript/no-explicit-any
import { sql } from 'drizzle-orm';
import { describe, expect, it, beforeAll, afterAll } from 'bun:test';
import { SessionRepository } from '../session.repository';
import { db } from '../../../../database/db';
import * as crypto from 'crypto';

describe('SessionRepository Integration', () => {
  const repo = new SessionRepository();
  let testAccountId: string;

  beforeAll(async () => {
    const accountId = crypto.randomUUID();
    testAccountId = accountId;
    await db.execute(sql`
      INSERT INTO "Account" (id, email, "emailVerified")
      VALUES (${accountId}, ${`test-${Date.now()}@test.com`}, true)
    `);
  });

  afterAll(async () => {
    await db.execute(sql`DELETE FROM "Account" WHERE id = ${testAccountId}`);
  });

  it('should create and retrieve a session by token', async () => {
    const token = crypto.randomUUID(); // Mock hashed token
    const expiresAt = new Date(Date.now() + 100000).toISOString();

    const created = await repo.create({
      accountId: testAccountId,
      token,
      expiresAt,
      userAgent: 'bun-test',
    });

    expect(created.id).toBeDefined();
    expect(created.accountId).toBe(testAccountId);
    expect(created.token).toBe(token);
    expect(created.userAgent).toBe('bun-test');

    const retrieved = await repo.findByToken(token);
    expect(retrieved).not.toBeNull();
    expect(retrieved?.id).toBe(created.id);
    expect(retrieved?.account).toBeDefined();
    expect(retrieved?.account.id).toBe(testAccountId);
    
    // Clean up
    await repo.deleteByToken(token);
    const deleted = await repo.findByToken(token);
    expect(deleted).toBeNull();
  });
});

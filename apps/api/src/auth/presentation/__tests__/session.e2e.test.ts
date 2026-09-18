// oxlint-disable typescript/no-explicit-any
import { sql } from 'drizzle-orm';
import { describe, expect, it, beforeAll, afterAll } from 'bun:test';
import { treaty } from '@elysia/eden';
import { authController } from '../auth.controller';
import { db } from '../../../database/db';
import { tokenService } from '../../infrastructure/services/token.service';
import { SessionRepository } from '../../infrastructure/repositories/session.repository';
import * as crypto from 'crypto';

type App = typeof authController;

describe('Session Middleware E2E', () => {
  let testAccountId: string;
  let validRawToken: string;
  let expiredRawToken: string;

  const client = treaty<App>(authController);

  beforeAll(async () => {
    const accountId = crypto.randomUUID();
    testAccountId = accountId;
    await db.execute(sql`
      INSERT INTO "Account" (id, email, "emailVerified")
      VALUES (${accountId}, ${`e2e-${Date.now()}@test.com`}, true)
    `);

    const repo = new SessionRepository();
    
    validRawToken = tokenService.generate(64);
    await repo.create({
      accountId,
      token: tokenService.hash(validRawToken),
      expiresAt: new Date(Date.now() + 100000).toISOString(),
    });

    expiredRawToken = tokenService.generate(64);
    await repo.create({
      accountId,
      token: tokenService.hash(expiredRawToken),
      expiresAt: new Date(Date.now() - 100000).toISOString(),
    });
  });

  afterAll(async () => {
    await db.execute(sql`DELETE FROM "Account" WHERE id = ${testAccountId}`);
  });

  it('should successfully get the user account using a valid token', async () => {
    const { data, error, status } = await client.auth.me.get({
      headers: {
        Authorization: `Bearer ${validRawToken}`
      }
    });

    expect(status).toBe(200);
    expect(error).toBeNull();
    expect(data).not.toBeNull();
    expect(data?.account).toBeDefined();
    expect(data?.account?.id).toBe(testAccountId);
  });

  it('should return 401 Unauthorized if no token is provided', async () => {
    const { data, error, status } = await client.auth.me.get();

    expect(status).toBe(401);
    expect(data).toBeNull();
    expect(error?.value).toMatchObject({ message: 'Unauthorized access' });
  });

  it('should return 401 Unauthorized if the token is invalid', async () => {
    const { data, error, status } = await client.auth.me.get({
      headers: {
        Authorization: 'Bearer invalid_garbage_token'
      }
    });

    expect(status).toBe(401);
    expect(data).toBeNull();
    expect(error?.value).toMatchObject({ message: 'Unauthorized access' });
  });

  it('should return 401 Unauthorized if the token is expired', async () => {
    const { data, error, status } = await client.auth.me.get({
      headers: {
        Authorization: `Bearer ${expiredRawToken}`
      }
    });

    expect(status).toBe(401);
    expect(data).toBeNull();
    expect(error?.value).toMatchObject({ message: 'Unauthorized access' });
  });
});

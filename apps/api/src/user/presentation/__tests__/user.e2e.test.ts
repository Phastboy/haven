// oxlint-disable typescript/no-explicit-any
import { expect, test, describe, beforeAll, afterAll } from 'bun:test';
import { treaty } from '@elysia/eden';
import { Elysia } from 'elysia';
import { createUserPlugin } from '../user.plugin';
import { authController } from '../../../auth/presentation/auth.controller';
import { db } from '../../../database/db';
import { sql } from 'drizzle-orm';
import { AccountRepository } from '../../../auth/infrastructure/repositories/account.repository';
import { SessionRepository } from '../../../auth/infrastructure/repositories/session.repository';
import { TokenService } from '../../../auth/infrastructure/services/token.service';
import { SqlUserRepository } from '../../infrastructure/sql-user.repository';

const app = new Elysia({ prefix: '/api' })
  .use(createUserPlugin())
  .use(authController);

describe('User Plugin E2E', () => {
  const accountRepo = new AccountRepository();
  const sessionRepo = new SessionRepository();
  const userRepo = new SqlUserRepository();
  const tokenService = new TokenService();

  let testAccountId: string;
  let testSessionRawToken: string;
  let testUserId: string;
  let otherUserId: string;

  beforeAll(async () => {


    // Setup Test User 1 (Authenticated)
    const testEmail = `e2e-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: true });
    testAccountId = account.id;

    const user = await userRepo.create({ accountId: account.id, username: 'e2e-user', name: 'E2E User' });
    testUserId = user.id;

    testSessionRawToken = tokenService.generate(64);
    const sessionHashedToken = tokenService.hash(testSessionRawToken);

    await sessionRepo.create({
      accountId: account.id,
      token: sessionHashedToken,
      expiresAt: new Date(Date.now() + 100000).toISOString(),
    });

    // Setup Test User 2 (Other User)
    const otherAccount = await accountRepo.create({ email: `other-${crypto.randomUUID()}@example.com`, emailVerified: true });
    const otherUser = await userRepo.create({ accountId: otherAccount.id, username: 'other-user' });
    otherUserId = otherUser.id;
  });

  afterAll(async () => {
    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${testAccountId}`);
    await db.execute(sql`DELETE FROM "Account" WHERE "id" IN (SELECT "accountId" FROM "User" WHERE "id" = ${otherUserId})`);
  });

  test('GET /api/users should list profiles', async () => {
    const res = await app.handle(new Request('http://localhost/api/users'));
    expect(res.status).toBe(200);
    const data = await res.json() as any[];
    expect(Array.isArray(data)).toBe(true);
    expect(data.length).toBeGreaterThanOrEqual(2);
  });

  test('GET /api/users/:id should fetch a specific profile', async () => {
    const res = await app.handle(new Request(`http://localhost/api/users/${testUserId}`));
    expect(res.status).toBe(200);
    const data = await res.json() as any;
    expect(data.id).toBe(testUserId);
    expect(data.username).toBe('e2e-user');
  });

  test('GET /api/users/:id should return 404 for non-existent profile', async () => {
    const res = await app.handle(new Request('http://localhost/api/users/non-existent'));
    expect(res.status).toBe(404);
  });

  test('PATCH /api/users/me should fail without auth', async () => {
    const res = await app.handle(new Request('http://localhost/api/users/me', {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Hacked' })
    }));
    expect(res.status).toBe(401);
  });

  test('PATCH /api/users/me should update the authenticated user profile', async () => {
    const res = await app.handle(new Request('http://localhost/api/users/me', {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${testSessionRawToken}`
      },
      body: JSON.stringify({ name: 'Updated E2E Name', bio: 'New Bio' })
    }));

    expect(res.status).toBe(200);
    const data = await res.json() as any;
    expect(data.id).toBe(testUserId);
    expect(data.name).toBe('Updated E2E Name');
    expect(data.bio).toBe('New Bio');

    // Verify in db
    const fetchRes = await app.handle(new Request(`http://localhost/api/users/${testUserId}`));
    const fetchProfile = await fetchRes.json() as any;
    expect(fetchProfile.name).toBe('Updated E2E Name');
  });
});

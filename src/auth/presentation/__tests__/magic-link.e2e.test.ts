import { expect, test, describe, afterAll } from 'bun:test';
import { Elysia } from 'elysia';
import { authController } from '../auth.controller';
import { db } from '../../../prisma/db';
import { tokenService } from '../../infrastructure/services/token.service';

describe('Magic Link E2E', () => {
  const app = new Elysia().use(authController);
  const testEmail = `e2e-magic-${crypto.randomUUID()}@example.com`;
  
  afterAll(async () => {
    // Cleanup
    const account = await db.orm.public.Account.where({ email: testEmail }).first();
    if (account) {
      await db.orm.public.Session.where({ accountId: account.id }).delete();
      await db.orm.public.AccountPlatformLink.where({ accountId: account.id }).delete();
      await db.orm.public.Account.where({ id: account.id }).delete();
    }
    await db.orm.public.MagicLink.where({ email: testEmail }).delete();
  });

  test('should request a magic link', async () => {
    const response = await app.handle(
      new Request('http://localhost/auth/magic-link/request', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email: testEmail }),
      })
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.message).toContain('If the email exists, a magic link was sent');

    const magicLink = await db.orm.public.MagicLink.where({ email: testEmail }).first();
    expect(magicLink).not.toBeNull();
    expect(magicLink!.usedAt).toBeNull();
  });

  test('should verify a magic link and create a session', async () => {
    const rawToken = crypto.randomUUID();
    const hashedToken = tokenService.hash(rawToken);

    await db.orm.public.MagicLink.create({
      email: testEmail,
      token: hashedToken,
      expiresAt: new Date(Date.now() + 1000 * 60 * 15).toISOString(),
    });

    const verifyResponse = await app.handle(
      new Request(`http://localhost/auth/magic-link/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ token: rawToken }),
      })
    );

    const body = await verifyResponse.json();
    if (verifyResponse.status !== 200) {
      console.log('Verify Error:', body);
    }
    expect(verifyResponse.status).toBe(200);
    expect(body.id).toBeDefined(); // The returned object is the session
    expect(body.token).toBeDefined();
    
    // Verify account and link were created
    const account = await db.orm.public.Account.where({ email: testEmail }).first();
    expect(account).not.toBeNull();
    expect(account!.emailVerified).toBe(true);

    const platformLink = await db.orm.public.AccountPlatformLink.where({ accountId: account!.id, platform: 'haven_platform' }).first();
    expect(platformLink).not.toBeNull();
  });
});

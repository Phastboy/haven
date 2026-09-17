import { expect, test, describe, afterAll } from 'bun:test';
import { Elysia } from 'elysia';
import { authController } from '../auth.controller';
import { db } from '../../../database/db';
import { tokenService } from '../../infrastructure/services/token.service';

describe('Magic Link E2E', () => {
  const app = new Elysia().use(authController);
  const testEmail = `e2e-magic-${crypto.randomUUID()}@example.com`;
  
  afterAll(async () => {
    // Cleanup
    const account = await db`SELECT * FROM "Account" WHERE "email" = ${testEmail}`.then(res => res[0] || null);
    if (account) {
      await db`DELETE FROM "Session" WHERE "accountId" = ${account.id}`;
      await db`DELETE FROM "AccountPlatformLink" WHERE "accountId" = ${account.id}`;
      await db`DELETE FROM "Account" WHERE "id" = ${account.id}`;
    }
    await db`DELETE FROM "MagicLink" WHERE "email" = ${testEmail}`;
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

    const magicLink = await db`SELECT * FROM "MagicLink" WHERE "email" = ${testEmail}`.then(res => res[0] || null);
    expect(magicLink).not.toBeNull();
    expect(magicLink!.usedAt).toBeNull();
  });

  test('should verify a magic link and create a session', async () => {
    const rawToken = crypto.randomUUID();
    const hashedToken = tokenService.hash(rawToken);

    await (async () => { const id = crypto.randomUUID(); await db`INSERT INTO "MagicLink" (id, email, token, "expiresAt") VALUES (${id}, ${testEmail}, ${hashedToken}, ${new Date(Date.now() + 1000000).toISOString()})`; return { id }; })();

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
    const account = await db`SELECT * FROM "Account" WHERE "email" = ${testEmail}`.then(res => res[0] || null);
    expect(account).not.toBeNull();
    expect(account!.emailVerified).toBe(true);

    const platformLink = await db`SELECT * FROM "AccountPlatformLink" WHERE "accountId" = ${account!.id} AND "platform" = 'haven_platform'`.then(res => res[0] || null);
    expect(platformLink).not.toBeNull();
  });
});

import { expect, test, describe, beforeAll, afterAll, mock } from 'bun:test';
import { Elysia } from 'elysia';
import { authController } from '../auth.controller';
import { db } from '../../../database/db';
import { OAuth2Client } from 'google-auth-library';

describe('Google OAuth E2E', () => {
  const app = new Elysia().use(authController);
  const testEmail = `e2e-google-${crypto.randomUUID()}@example.com`;
  const googleId = `google-id-${crypto.randomUUID()}`;
  
  let originalVerifyIdToken: any;

  beforeAll(() => {
    originalVerifyIdToken = OAuth2Client.prototype.verifyIdToken;
    
    // Mock the verifyIdToken method
    OAuth2Client.prototype.verifyIdToken = mock(async () => {
      return {
        getPayload: () => ({
          sub: googleId,
          email: testEmail,
          email_verified: true,
          name: 'Test Google User',
          picture: 'https://example.com/photo.jpg',
        })
      };
    }) as any;
  });

  afterAll(async () => {
    OAuth2Client.prototype.verifyIdToken = originalVerifyIdToken;

    // Cleanup
    const account = await db`SELECT * FROM "Account" WHERE "email" = ${testEmail}`.then(res => res[0] || null);
    if (account) {
      await db`DELETE FROM "Session" WHERE "accountId" = ${account.id}`;
      await db`DELETE FROM "OAuthCredential" WHERE "accountId" = ${account.id}`;
      await db`DELETE FROM "AccountPlatformLink" WHERE "accountId" = ${account.id}`;
      await db`DELETE FROM "Account" WHERE "id" = ${account.id}`;
    }
  });

  test('should login with google idToken and create session', async () => {
    const response = await app.handle(
      new Request('http://localhost/auth/google/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ idToken: 'mocked-id-token' }),
      })
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.id).toBeDefined();
    expect(body.token).toBeDefined();
    
    // Verify DB records
    const account = await db`SELECT * FROM "Account" WHERE "email" = ${testEmail}`.then(res => res[0] || null);
    expect(account).not.toBeNull();
    expect(account!.emailVerified).toBe(true);

    const oauth = await db`SELECT * FROM "OAuthCredential" WHERE "accountId" = ${account!.id} AND "provider" = 'GOOGLE'`.then(res => res[0] || null);
    expect(oauth).not.toBeNull();
    expect(oauth!.providerUserId).toBe(googleId);

    const platformLink = await db`SELECT * FROM "AccountPlatformLink" WHERE "accountId" = ${account!.id} AND "platform" = 'haven_platform'`.then(res => res[0] || null);
    expect(platformLink).not.toBeNull();
  });
});

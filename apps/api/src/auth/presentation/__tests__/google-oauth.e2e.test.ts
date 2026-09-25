import { config } from "../../../config";
import { sql } from "drizzle-orm";
import { expect, test, describe, beforeAll, afterAll, mock } from "bun:test";
import { Elysia } from "elysia";
import { createAuthPlugin } from "../auth.controller";
import { createDb } from "../../../database/db";
const db = createDb(config);

import { OAuth2Client } from "google-auth-library";

const mockProfileCreator = { createProfileForAccount: async () => {} };

describe("Google OAuth E2E", () => {
  const app = new Elysia().use(createAuthPlugin(mockProfileCreator, config, db));
  const testEmail = `e2e-google-${crypto.randomUUID()}@example.com`;
  const googleId = `google-id-${crypto.randomUUID()}`;

  let originalVerifyIdToken: typeof OAuth2Client.prototype.verifyIdToken;

  beforeAll(() => {
    originalVerifyIdToken = OAuth2Client.prototype.verifyIdToken;

    // Mock the verifyIdToken method
    OAuth2Client.prototype.verifyIdToken = mock(async () => {
      return {
        getPayload: () => ({
          sub: googleId,
          email: testEmail,
          email_verified: true,
          name: "Test Google User",
          picture: "https://example.com/photo.jpg",
        }),
      };
    }) as unknown as typeof OAuth2Client.prototype.verifyIdToken;
  });

  afterAll(async () => {
    OAuth2Client.prototype.verifyIdToken = originalVerifyIdToken;

    // Cleanup
    const account = (
      await db.execute(sql`SELECT * FROM "Account" WHERE "email" = ${testEmail}`)
    )[0] as unknown as { id: string; emailVerified: boolean; providerUserId: string };
    if (!account) return;
    await db.execute(sql`DELETE FROM "Session" WHERE "accountId" = ${account.id}`);
    await db.execute(sql`DELETE FROM "OAuthCredential" WHERE "accountId" = ${account.id}`);
    await db.execute(sql`DELETE FROM "AccountPlatformLink" WHERE "accountId" = ${account.id}`);
    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should login with google idToken and create session", async () => {
    const response = await app.handle(
      new Request("http://localhost/auth/google/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ idToken: "mocked-id-token" }),
      }),
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.data.id).toBeDefined();
    expect(body.data.token).toBeDefined();

    // Verify DB records
    const account = (
      await db.execute(sql`SELECT * FROM "Account" WHERE "email" = ${testEmail}`)
    )[0] as unknown as { id: string; emailVerified: boolean; providerUserId: string };
    expect(account).toBeDefined();
    expect((account as unknown as { emailVerified: boolean }).emailVerified).toBe(true);

    const oauth = (
      await db.execute(
        sql`SELECT * FROM "OAuthCredential" WHERE "accountId" = ${(account as unknown as { id: string }).id} AND "provider" = 'GOOGLE'`,
      )
    )[0] as unknown as { id: string; emailVerified: boolean; providerUserId: string };
    expect(oauth).toBeDefined();
    expect((oauth as unknown as { providerUserId: string }).providerUserId).toBe(googleId);
  });
});

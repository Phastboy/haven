// oxlint-disable typescript/no-explicit-any
import { sql } from "drizzle-orm";
import { expect, test, describe, afterAll } from "bun:test";
import { Elysia } from "elysia";
import { authController } from "../auth.controller";
import { db } from "../../../database/db";
import { tokenService } from "../../infrastructure/services/token.service";

describe("Magic Link E2E", () => {
  const app = new Elysia().use(authController);
  const testEmail = `e2e-magic-${crypto.randomUUID()}@example.com`;

  afterAll(async () => {
    // Cleanup
    const account = (
      await db.execute(sql`SELECT * FROM "Account" WHERE "email" = ${testEmail}`)
    )[0] as unknown as { id: string; emailVerified: boolean; token: string; usedAt: Date | null };
    if (account) {
      await db.execute(sql`DELETE FROM "Session" WHERE "accountId" = ${account.id}`);
      await db.execute(sql`DELETE FROM "AccountPlatformLink" WHERE "accountId" = ${account.id}`);
      await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
    }
    await db.execute(sql`DELETE FROM "MagicLink" WHERE "email" = ${testEmail}`);
  });

  test("should request a magic link", async () => {
    const response = await app.handle(
      new Request("http://localhost/auth/magic-link/request", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email: testEmail }),
      }),
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.message).toContain("If the email exists, a magic link was sent");

    const magicLink = (
      await db.execute(sql`SELECT * FROM "MagicLink" WHERE "email" = ${testEmail}`)
    )[0] as unknown as { id: string; emailVerified: boolean; token: string; usedAt: Date | null };
    expect(magicLink).toBeDefined();
    expect(magicLink!.token).toBeDefined();
    expect((magicLink as unknown as { usedAt: Date | null }).usedAt).toBeNull();
  });

  test("should verify a magic link and create a session", async () => {
    const rawToken = crypto.randomUUID();
    const hashedToken = tokenService.hash(rawToken);

    await (async () => {
      const id = crypto.randomUUID();
      await db.execute(
        sql`INSERT INTO "MagicLink" (id, email, token, "expiresAt") VALUES (${id}, ${testEmail}, ${hashedToken}, ${new Date(Date.now() + 1000000).toISOString()})`,
      );
      return { id };
    })();

    const verifyResponse = await app.handle(
      new Request(`http://localhost/auth/magic-link/verify`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ token: rawToken }),
      }),
    );

    const body = await verifyResponse.json();
    if (verifyResponse.status !== 200) {
      console.log("Verify Error:", body);
    }
    expect(verifyResponse.status).toBe(200);
    expect(body.id).toBeDefined(); // The returned object is the session
    expect(body.token).toBeDefined();

    // Verify account and link were created
    const account = (
      await db.execute(sql`SELECT * FROM "Account" WHERE "email" = ${testEmail}`)
    )[0] as unknown as { id: string; emailVerified: boolean; token: string; usedAt: Date | null };
    expect(account).toBeDefined();
    expect((account as unknown as { emailVerified: boolean }).emailVerified).toBe(true);
  });
});

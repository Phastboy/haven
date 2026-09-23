import { sql } from "drizzle-orm";
import { expect, test, describe } from "bun:test";
import { SessionRepository } from "../session.repository";
import { AccountRepository } from "../account.repository";

import { db } from "../../../../database/db";

describe("SessionRepository Integration", () => {
  const sessionRepo = new SessionRepository();
  const accountRepo = new AccountRepository();

  test("should create a session", async () => {
    const account = await accountRepo.create({
      email: `test-session-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const testToken = crypto.randomUUID();
    const expiresAt = new Date(Date.now() + 100000).toISOString();
    const session = await sessionRepo.create({
      accountId: account.id,
      token: testToken,
      expiresAt,
      userAgent: "integration-test",
      ipAddress: "127.0.0.1",
    });

    expect(session.id).toBeDefined();
    expect(session.accountId).toBe(account.id);
    expect(session.token).toBe(testToken);

    await db.execute(sql`DELETE FROM "Session" WHERE "id" = ${session.id}`);
    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should find session by token with account included", async () => {
    const account = await accountRepo.create({
      email: `test-session-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const testToken = crypto.randomUUID();
    const expiresAt = new Date(Date.now() + 100000).toISOString();
    const session = await sessionRepo.create({
      accountId: account.id,
      token: testToken,
      expiresAt,
    });

    const found = await sessionRepo.findByToken(testToken);
    expect(found).not.toBeNull();
    expect(found!.token).toBe(testToken);
    expect(found!.account).toBeDefined();
    expect(found!.account.email).toBe(account.email);

    await db.execute(sql`DELETE FROM "Session" WHERE "id" = ${session.id}`);
    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should delete session by token", async () => {
    const account = await accountRepo.create({
      email: `test-session-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    const testToken = crypto.randomUUID();
    const expiresAt = new Date(Date.now() + 100000).toISOString();
    await sessionRepo.create({
      accountId: account.id,
      token: testToken,
      expiresAt,
    });

    await sessionRepo.deleteByToken(testToken);
    const found = await sessionRepo.findByToken(testToken);
    expect(found).toBeNull();

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should delete expired sessions", async () => {
    const account = await accountRepo.create({
      email: `test-session-${crypto.randomUUID()}@example.com`,
      emailVerified: true,
    });

    // Create an expired session
    const expiredToken = crypto.randomUUID();
    const pastDate = new Date(Date.now() - 100000).toISOString();

    await sessionRepo.create({
      accountId: account.id,
      token: expiredToken,
      expiresAt: pastDate,
    });

    // Create a valid session
    const validToken = crypto.randomUUID();
    const futureDate = new Date(Date.now() + 100000).toISOString();

    const validSession = await sessionRepo.create({
      accountId: account.id,
      token: validToken,
      expiresAt: futureDate,
    });

    await sessionRepo.deleteExpired();

    const checkExpired = await sessionRepo.findByToken(expiredToken);
    const checkValid = await sessionRepo.findByToken(validToken);

    expect(checkExpired).toBeNull();
    expect(checkValid).not.toBeNull();

    await db.execute(sql`DELETE FROM "Session" WHERE "id" = ${validSession.id}`);
    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });
});

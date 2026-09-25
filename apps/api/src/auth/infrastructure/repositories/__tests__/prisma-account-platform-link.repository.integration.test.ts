import { config } from "../../../../config";
import { sql } from "drizzle-orm";
import { expect, test, describe, afterAll, beforeAll } from "bun:test";
import { AccountPlatformLinkRepository } from "../account-platform-link.repository";
import { AccountRepository } from "../account.repository";

import { createDb } from "../../../../database/db";
const db = createDb(config);


describe("AccountPlatformLinkRepository Integration", () => {
  const linkRepo = new AccountPlatformLinkRepository(db);
  const accountRepo = new AccountRepository(db);

  const testEmail = `test-link-${crypto.randomUUID()}@example.com`;
  const platformUserId = `platform-user-${crypto.randomUUID()}`;
  let accountId: string;

  beforeAll(async () => {
    const account = await accountRepo.create({
      email: testEmail,
      emailVerified: true,
    });
    accountId = account.id;
  });

  afterAll(async () => {
    if (accountId) {
      await db.execute(sql`DELETE FROM "AccountPlatformLink" WHERE "accountId" = ${accountId}`);
      await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${accountId}`);
    }
  });

  test("should create an account platform link", async () => {
    const link = await linkRepo.create({
      accountId,
      platformUserId,
      platform: "haven",
    });

    expect(link.id).toBeDefined();
    expect(link.accountId).toBe(accountId);
    expect(link.platformUserId).toBe(platformUserId);
    expect(link.platform).toBe("haven");
  });

  test("should find link by account and platform", async () => {
    const link = await linkRepo.findByAccountAndPlatform(accountId, "haven");
    expect(link).not.toBeNull();
    expect(link!.platformUserId).toBe(platformUserId);
  });

  test("should enforce unique constraint on (accountId, platform)", async () => {
    // Attempting to create a second link for the same platform should throw
    const promise = linkRepo.create({
      accountId,
      platformUserId: "another-user",
      platform: "haven",
    });

    await expect(promise).rejects.toThrow();
  });

  test("should allow a different platform for the same account", async () => {
    const link = await linkRepo.create({
      accountId,
      platformUserId: "another-user-external",
      platform: "external-app",
    });

    expect(link.platform).toBe("external-app");
  });
});

// oxlint-disable typescript/no-explicit-any
import { sql } from "drizzle-orm";
import { expect, test, describe } from "bun:test";
import { AccountRepository } from "../account.repository";

import { db } from "../../../../database/db";

describe("AccountRepository Integration", () => {
  const repo = new AccountRepository();

  test("should create a new account", async () => {
    const testEmail = `test-${crypto.randomUUID()}@example.com`;
    const account = await repo.create({
      email: testEmail,
      emailVerified: false,
    });

    expect(account.id).toBeDefined();
    expect(account.email).toBe(testEmail);
    expect(account.emailVerified).toBe(false);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should find account by email", async () => {
    const testEmail = `test-${crypto.randomUUID()}@example.com`;
    const account = await repo.create({
      email: testEmail,
      emailVerified: false,
    });

    const found = await repo.findByEmail(testEmail);
    expect(found).not.toBeNull();
    expect(found!.id).toBe(account.id);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should return null for non-existent email", async () => {
    const account = await repo.findByEmail("does-not-exist@example.com");
    expect(account).toBeNull();
  });

  test("should find account by id", async () => {
    const testEmail = `test-${crypto.randomUUID()}@example.com`;
    const account = await repo.create({
      email: testEmail,
      emailVerified: false,
    });

    const found = await repo.findById(account.id);
    expect(found).not.toBeNull();
    expect(found!.email).toBe(testEmail);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test("should mark email as verified", async () => {
    const testEmail = `test-${crypto.randomUUID()}@example.com`;
    const account = await repo.create({
      email: testEmail,
      emailVerified: false,
    });

    await repo.markEmailVerified(account.id);
    const updated = await repo.findById(account.id);
    expect(updated!.emailVerified).toBe(true);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });
});

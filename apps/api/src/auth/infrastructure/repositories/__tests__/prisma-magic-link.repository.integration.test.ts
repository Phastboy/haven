import { sql } from "drizzle-orm";
import { expect, test, describe, afterAll } from "bun:test";
import { MagicLinkRepository } from "../magic-link.repository";

import { db } from "../../../../database/db";

describe("MagicLinkRepository Integration", () => {
  const repo = new MagicLinkRepository();
  const testEmail = `test-ml-${crypto.randomUUID()}@example.com`;
  const testToken = crypto.randomUUID();
  let magicLinkId: string;

  afterAll(async () => {
    if (magicLinkId) {
      await db.execute(sql`DELETE FROM "MagicLink" WHERE "id" = ${magicLinkId}`);
    }
  });

  test("should create a magic link", async () => {
    const expiresAt = new Date(Date.now() + 900000).toISOString(); // 15 mins
    const magicLink = await repo.create({
      email: testEmail,
      token: testToken,
      expiresAt,
    });

    expect(magicLink.id).toBeDefined();
    expect(magicLink.email).toBe(testEmail);
    expect(magicLink.token).toBe(testToken);

    magicLinkId = magicLink.id;
  });

  test("should find magic link by token", async () => {
    const magicLink = await repo.findByToken(testToken);
    expect(magicLink).not.toBeNull();
    expect(magicLink!.id).toBe(magicLinkId);
  });

  test("should return null for non-existent token", async () => {
    const magicLink = await repo.findByToken("does-not-exist");
    expect(magicLink).toBeNull();
  });

  test("should mark magic link as used", async () => {
    const usedAt = new Date().toISOString();
    await repo.markUsed(magicLinkId, usedAt);

    const magicLink = await repo.findByToken(testToken);
    expect(magicLink!.usedAt).toBeDefined();
    expect(magicLink!.usedAt).not.toBeNull();
  });
});

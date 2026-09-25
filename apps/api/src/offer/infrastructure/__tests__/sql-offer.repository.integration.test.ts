import { config } from "../../../config";
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { createDb } from "../../../database/db";
const db = createDb(config);

import { users, accounts } from "../../../database/schema";
import { SqlOfferRepository } from "../sql-offer.repository";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";

describe("SqlOfferRepository Integration", () => {
  let repository: SqlOfferRepository;
  let testAccountId: string;
  let testUserId: string;

  beforeAll(async () => {
    repository = new SqlOfferRepository(db);
    testAccountId = randomUUID();
    testUserId = randomUUID();

    await db.insert(accounts).values({
      id: testAccountId,
      email: `test-offer-${Date.now()}@example.com`,
    });

    await db.insert(users).values({
      id: testUserId,
      accountId: testAccountId,
      username: `testuser-${Date.now()}`,
    });
  });

  afterAll(async () => {
    await db.delete(accounts).where(eq(accounts.id, testAccountId));
  });

  it("should create a new offer", async () => {
    const offer = await repository.create(testUserId, {
      title: "Guitar Lessons",
      description: "Learn to play guitar",
      price: 5000,
      offerType: "SERVICE",
    });

    expect(offer.id).toBeDefined();
    expect(offer.userId).toBe(testUserId);
    expect(offer.title).toBe("Guitar Lessons");
    expect(offer.price).toBe(5000);
    expect(offer.offerType).toBe("SERVICE");
    expect(offer.status).toBe("ACTIVE");
  });

  it("should find offer by id", async () => {
    const created = await repository.create(testUserId, {
      title: "Test Offer 2",
    });

    const found = await repository.findById(created.id);
    expect(found).not.toBeNull();
    expect(found?.title).toBe("Test Offer 2");
  });

  it("should list offers by user id", async () => {
    const offers = await repository.findByUserId(testUserId);
    expect(offers.length).toBeGreaterThanOrEqual(2);
  });

  it("should update an offer", async () => {
    const created = await repository.create(testUserId, {
      title: "Update Me",
    });

    const updated = await repository.update(created.id, { title: "Updated Title" });
    expect(updated?.title).toBe("Updated Title");
  });

  it("should soft delete an offer", async () => {
    const created = await repository.create(testUserId, {
      title: "Delete Me",
    });

    await repository.delete(created.id);
    const found = await repository.findById(created.id);
    expect(found?.status).toBe("ARCHIVED");
  });
});

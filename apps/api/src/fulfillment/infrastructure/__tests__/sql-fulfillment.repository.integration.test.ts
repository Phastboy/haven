import { config } from "../../../config";
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { createDb } from "../../../database/db";
const db = createDb(config);

import { accounts, offers, orders, users } from "../../../database/schema";
import { SqlFulfillmentRepository } from "../sql-fulfillment.repository";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";

describe("SqlFulfillmentRepository Integration", () => {
  let repository: SqlFulfillmentRepository;

  let testAccountId: string;
  let testUserId: string;
  let testOfferId: string;
  let testOrderId: string;

  beforeAll(async () => {
    repository = new SqlFulfillmentRepository(db);

    testAccountId = randomUUID();
    testUserId = randomUUID();
    await db
      .insert(accounts)
      .values({ id: testAccountId, email: `fulfillment-owner-${Date.now()}@example.com` });
    await db
      .insert(users)
      .values({
        id: testUserId,
        accountId: testAccountId,
        username: `fulfillmentowner-${Date.now()}`,
      });

    testOfferId = randomUUID();
    await db.insert(offers).values({
      id: testOfferId,
      userId: testUserId,
      title: "Fulfillment Offer",
      price: 1000,
    });

    testOrderId = randomUUID();
    await db.insert(orders).values({
      id: testOrderId,
      offerId: testOfferId,
      requesterId: testUserId, // for test simplicity, same user
      price: 1000,
      quantity: 1,
    });
  });

  afterAll(async () => {
    await db.delete(accounts).where(eq(accounts.id, testAccountId));
  });

  it("should create a fulfillment and retrieve it by orderId", async () => {
    const fulfillmentId = randomUUID();
    const created = await repository.createFulfillment({
      id: fulfillmentId,
      orderId: testOrderId,
    });

    expect(created.id).toBe(fulfillmentId);
    expect(created.status).toBe("PENDING");

    const fetched = await repository.getFulfillmentByOrderId(testOrderId);
    expect(fetched).toBeDefined();
    expect(fetched?.id).toBe(fulfillmentId);
  });

  it("should update fulfillment status and options", async () => {
    const fulfillmentId = randomUUID();
    const orderId2 = randomUUID();
    await db.insert(orders).values({
      id: orderId2,
      offerId: testOfferId,
      requesterId: testUserId,
      price: 1000,
      quantity: 1,
    });

    await repository.createFulfillment({ id: fulfillmentId, orderId: orderId2 });

    const deadline = new Date(Date.now() + 10000);
    const updated = await repository.updateFulfillmentStatus(fulfillmentId, "DELIVERED", {
      deliveryMessage: "Here is your order",
      reviewDeadline: deadline,
    });

    expect(updated.status).toBe("DELIVERED");
    expect(updated.deliveryMessage).toBe("Here is your order");
    expect(new Date(updated.reviewDeadline!).getTime()).toBe(deadline.getTime());
  });

  it("should query expired fulfillments", async () => {
    const fulfillmentId = randomUUID();
    const orderId3 = randomUUID();
    await db.insert(orders).values({
      id: orderId3,
      offerId: testOfferId,
      requesterId: testUserId,
      price: 1000,
      quantity: 1,
    });

    await repository.createFulfillment({ id: fulfillmentId, orderId: orderId3 });

    const pastDeadline = new Date(Date.now() - 100000); // in the past
    await repository.updateFulfillmentStatus(fulfillmentId, "DELIVERED", {
      reviewDeadline: pastDeadline,
    });

    const expired = await repository.getExpiredFulfillments(new Date());
    expect(expired.length).toBeGreaterThanOrEqual(1);

    const found = expired.find((f) => f.id === fulfillmentId);
    expect(found).toBeDefined();
    expect(found?.status).toBe("DELIVERED");
  });
});

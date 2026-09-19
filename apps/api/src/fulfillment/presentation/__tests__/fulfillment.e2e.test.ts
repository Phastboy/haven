import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { randomUUID } from "crypto";
import { db } from "../../../database/db";
import { users, accounts, sessions, offers, orders } from "../../../database/schema";
import { app } from "../../../index";
import { eq, inArray } from "drizzle-orm";
import { tokenService } from "../../../auth/infrastructure/services/token.service";

describe("Fulfillment API E2E", () => {
  let ownerAccountId: string;
  let ownerUserId: string;
  let ownerSessionToken: string;

  let requesterAccountId: string;
  let requesterUserId: string;
  let requesterSessionToken: string;

  let testOfferId: string;
  let testOrderId: string;

  beforeAll(async () => {
    // 1. Setup Owner
    ownerAccountId = randomUUID();
    ownerUserId = randomUUID();
    await db
      .insert(accounts)
      .values({ id: ownerAccountId, email: `f-owner-${Date.now()}@example.com` });
    await db
      .insert(users)
      .values({ id: ownerUserId, accountId: ownerAccountId, username: `fowner-${Date.now()}` });

    ownerSessionToken = randomUUID();
    const hashedOwnerToken = await tokenService.hash(ownerSessionToken);
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: ownerAccountId,
      token: hashedOwnerToken,
      expiresAt: new Date(Date.now() + 1000000),
    });

    // 2. Setup Requester
    requesterAccountId = randomUUID();
    requesterUserId = randomUUID();
    await db
      .insert(accounts)
      .values({ id: requesterAccountId, email: `f-req-${Date.now()}@example.com` });
    await db
      .insert(users)
      .values({
        id: requesterUserId,
        accountId: requesterAccountId,
        username: `freq-${Date.now()}`,
      });

    requesterSessionToken = randomUUID();
    const hashedReqToken = await tokenService.hash(requesterSessionToken);
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: requesterAccountId,
      token: hashedReqToken,
      expiresAt: new Date(Date.now() + 1000000),
    });

    // 3. Setup Offer (SERVICE)
    testOfferId = randomUUID();
    await db.insert(offers).values({
      id: testOfferId,
      userId: ownerUserId,
      title: "Fulfillment E2E Offer",
      price: 1500,
      offerType: "SERVICE",
      status: "ACTIVE",
    });

    // 4. Setup Order (ACCEPTED)
    testOrderId = randomUUID();
    await db.insert(orders).values({
      id: testOrderId,
      offerId: testOfferId,
      requesterId: requesterUserId,
      price: 1500,
      quantity: 1,
      status: "ACCEPTED",
    });
  });

  afterAll(async () => {
    const ids = [];
    if (ownerAccountId) ids.push(ownerAccountId);
    if (requesterAccountId) ids.push(requesterAccountId);
    if (ids.length > 0) {
      await db.delete(accounts).where(inArray(accounts.id, ids));
    }
  });

  it("should not allow requester to deliver", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/fulfillment/deliver`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${requesterSessionToken}`,
      },
      body: JSON.stringify({ message: "Delivered!" }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(403);
  });

  it("should allow owner to deliver", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/fulfillment/deliver`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${ownerSessionToken}`,
      },
      body: JSON.stringify({ message: "Here is your work" }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body: any = await res.json();
    expect(body.status).toBe("DELIVERED");
    expect(body.deliveryMessage).toBe("Here is your work");
  });

  it("should allow requester to view fulfillment details", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/fulfillment`, {
      headers: { Authorization: `Bearer ${requesterSessionToken}` },
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body: any = await res.json();
    expect(body.status).toBe("DELIVERED");
  });

  it("should not allow owner to request a revision", async () => {
    const req = new Request(
      `http://localhost/api/orders/${testOrderId}/fulfillment/request-revision`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${ownerSessionToken}`,
        },
        body: JSON.stringify({ reason: "I am the owner" }),
      },
    );
    const res = await app.handle(req);
    expect(res.status).toBe(403);
  });

  it("should allow requester to request a revision", async () => {
    const req = new Request(
      `http://localhost/api/orders/${testOrderId}/fulfillment/request-revision`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${requesterSessionToken}`,
        },
        body: JSON.stringify({ reason: "Needs more polish" }),
      },
    );
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body: any = await res.json();
    expect(body.status).toBe("REVISION_REQUESTED");
  });

  it("should allow owner to deliver again after revision", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/fulfillment/deliver`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${ownerSessionToken}`,
      },
      body: JSON.stringify({ message: "Here is the fixed work" }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body: any = await res.json();
    expect(body.status).toBe("DELIVERED");
  });

  it("should allow requester to accept fulfillment", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/fulfillment/accept`, {
      method: "POST",
      headers: { Authorization: `Bearer ${requesterSessionToken}` },
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body: any = await res.json();
    expect(body.status).toBe("COMPLETED");
  });

  it("should have cascaded COMPLETED status back to the order", async () => {
    const [order] = await db.select().from(orders).where(eq(orders.id, testOrderId));
    expect(order!.status).toBe("COMPLETED");
  });
});

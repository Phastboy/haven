import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { randomUUID } from "crypto";
import { db } from "../../../database/db";
import { users, accounts, sessions, offers } from "../../../database/schema";
import { app } from "../../../index";
import { inArray } from "drizzle-orm";
import { tokenService } from "../../../auth/infrastructure/services/token.service";

describe("Order API E2E", () => {
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
      .values({ id: ownerAccountId, email: `owner-${Date.now()}@example.com` });
    await db
      .insert(users)
      .values({ id: ownerUserId, accountId: ownerAccountId, username: `owner-${Date.now()}` });

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
      .values({ id: requesterAccountId, email: `req-${Date.now()}@example.com` });
    await db.insert(users).values({
      id: requesterUserId,
      accountId: requesterAccountId,
      username: `req-${Date.now()}`,
    });

    requesterSessionToken = randomUUID();
    const hashedReqToken = await tokenService.hash(requesterSessionToken);
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: requesterAccountId,
      token: hashedReqToken,
      expiresAt: new Date(Date.now() + 1000000),
    });

    // 3. Setup Offer
    testOfferId = randomUUID();
    await db.insert(offers).values({
      id: testOfferId,
      userId: ownerUserId,
      title: "E2E Offer",
      price: 1500,
      status: "ACTIVE",
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

  it("should successfully create a new order as requester", async () => {
    const req = new Request("http://localhost/api/orders", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${requesterSessionToken}`,
      },
      body: JSON.stringify({
        offerId: testOfferId,
        quantity: 2,
        message: "I want this",
      }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(201);

    const body = (await res.json()) as {
      id?: string;
      status?: string;
      price?: number;
      quantity?: number;
      [key: string]: unknown;
    };
    expect(body.id).toBeDefined();
    expect(body.price).toBe(1500);
    expect(body.quantity).toBe(2);
    expect(body.status).toBe("PENDING");
    testOrderId = body.id!;
  });

  it("should fail to create order if user is offer owner", async () => {
    const req = new Request("http://localhost/api/orders", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${ownerSessionToken}`,
      },
      body: JSON.stringify({ offerId: testOfferId }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(400);
  });

  it("should allow requester to view their outgoing orders", async () => {
    const req = new Request("http://localhost/api/orders/me", {
      headers: { Authorization: `Bearer ${requesterSessionToken}` },
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { data: { id: string; status: string }[] };
    expect(Array.isArray(body.data)).toBe(true);
    expect(body.data.find((o: { id: string }) => o.id === testOrderId)).toBeDefined();
  });

  it("should allow owner to view received orders", async () => {
    const req = new Request("http://localhost/api/orders/received", {
      headers: { Authorization: `Bearer ${ownerSessionToken}` },
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { data: { id: string; status: string }[] };
    expect(Array.isArray(body.data)).toBe(true);
    expect(body.data.find((o: { id: string }) => o.id === testOrderId)).toBeDefined();
  });

  it("should allow owner to ACCEPT the order", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/status`, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${ownerSessionToken}`,
      },
      body: JSON.stringify({ status: "ACCEPTED" }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as {
      id?: string;
      status?: string;
      price?: number;
      quantity?: number;
      [key: string]: unknown;
    };
    expect(body.status).toBe("ACCEPTED");
  });

  it("should not allow requester to CANCEL an ACCEPTED order", async () => {
    const req = new Request(`http://localhost/api/orders/${testOrderId}/status`, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${requesterSessionToken}`,
      },
      body: JSON.stringify({ status: "CANCELLED" }),
    });
    const res = await app.handle(req);
    expect(res.status).toBe(422);
  });
});

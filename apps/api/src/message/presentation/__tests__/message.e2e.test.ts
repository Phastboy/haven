import { config } from "../../../config";
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { app } from "../../../index";
import { createDb } from "../../../database/db";
const db = createDb(config);

import { users, accounts, sessions } from "../../../database/schema";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";
import { offers } from "../../../database/schema";
import { TokenService } from "../../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);

describe("Message Plugin E2E", () => {
  let user1Token: string;
  let user2Token: string;
  let user1Id: string;
  let user2Id: string;
  let account1Id: string;
  let account2Id: string;

  beforeAll(async () => {
    account1Id = randomUUID();
    user1Id = randomUUID();
    user1Token = randomUUID();
    await db.insert(accounts).values({ id: account1Id, email: `e2e1_${randomUUID()}@example.com` });
    await db
      .insert(users)
      .values({
        id: user1Id,
        accountId: account1Id,
        username: `e2e1_${randomUUID()}`,
        name: "User 1",
      });
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: account1Id,
      token: tokenService.hash(user1Token),
      expiresAt: new Date(Date.now() + 1000000),
    });

    account2Id = randomUUID();
    user2Id = randomUUID();
    user2Token = randomUUID();
    await db.insert(accounts).values({ id: account2Id, email: `e2e2_${randomUUID()}@example.com` });
    await db
      .insert(users)
      .values({
        id: user2Id,
        accountId: account2Id,
        username: `e2e2_${randomUUID()}`,
        name: "User 2",
      });
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: account2Id,
      token: tokenService.hash(user2Token),
      expiresAt: new Date(Date.now() + 1000000),
    });

    for (let i = 0; i < 6; i++) {
      await db.insert(offers).values({
        id: randomUUID(),
        userId: user1Id,
        title: `Offer ${i}`,
        price: 1000,
        currency: "NGN",
      });
    }
  });

  afterAll(async () => {
    await db.delete(accounts).where(eq(accounts.id, account1Id));
    await db.delete(accounts).where(eq(accounts.id, account2Id));
  });

  it("should create a thread", async () => {
    const response = await app.handle(
      new Request("http://localhost/api/messages/threads", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ participantId: user2Id }),
      }),
    );
    expect(response.status).toBe(201);
    const body = (await response.json()) as { data?: { id?: string } };
    expect(body.data?.id).toBeDefined();
  });

  it("should list threads", async () => {
    const response = await app.handle(
      new Request("http://localhost/api/messages/threads", {
        method: "GET",
        headers: {
          Authorization: `Bearer ${user1Token}`,
        },
      }),
    );
    expect(response.status).toBe(200);
    const body = (await response.json()) as { data: unknown[] };
    expect(Array.isArray(body.data)).toBe(true);
  });

  it("should send a message and retrieve it", async () => {
    // 1. Create thread
    const createRes = await app.handle(
      new Request("http://localhost/api/messages/threads", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ participantId: user2Id }),
      }),
    );
    const createBody = (await createRes.json()) as { data?: { id?: string } };
    const threadId = createBody.data?.id;

    // 2. Send message
    const sendRes = await app.handle(
      new Request(`http://localhost/api/messages/threads/${threadId}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ content: "Hello E2E" }),
      }),
    );
    expect(sendRes.status).toBe(201);
    const sendBody = (await sendRes.json()) as { data?: { content?: string } };
    expect(sendBody.data?.content).toBe("Hello E2E");

    // 3. Get messages
    const getRes = await app.handle(
      new Request(`http://localhost/api/messages/threads/${threadId}`, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${user2Token}`,
        },
      }),
    );
    expect(getRes.status).toBe(200);
    const getBody = (await getRes.json()) as { data: { content?: string }[] };
    expect(Array.isArray(getBody.data)).toBe(true);
    expect(getBody.data[0]?.content).toBe("Hello E2E");
  });

  it("should send a message with multiple context offers", async () => {
    // 1. Create thread
    const createRes = await app.handle(
      new Request("http://localhost/api/messages/threads", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ participantId: user2Id }),
      }),
    );
    const createBody = (await createRes.json()) as { data?: { id?: string } };
    const threadId = createBody.data?.id;

    // Get 2 offers from user1
    const userOffers = await db.select().from(offers).where(eq(offers.userId, user1Id)).limit(2);
    const offerIds = userOffers.map((o) => o.id);

    // 2. Send message
    const sendRes = await app.handle(
      new Request(`http://localhost/api/messages/threads/${threadId}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ content: "Tagging two offers", contextOfferIds: offerIds }),
      }),
    );
    expect(sendRes.status).toBe(201);
    const sendBody = (await sendRes.json()) as {
      data?: { content?: string; contexts?: unknown[] };
    };
    expect(sendBody.data?.content).toBe("Tagging two offers");
    expect(sendBody.data?.contexts?.length).toBe(2);

    // 3. Get messages and verify eager-loading
    const getRes = await app.handle(
      new Request(`http://localhost/api/messages/threads/${threadId}`, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${user2Token}`,
        },
      }),
    );
    expect(getRes.status).toBe(200);
    const getBody = (await getRes.json()) as { data: { content?: string; contexts?: unknown[] }[] };
    const latestMessage = getBody.data[getBody.data.length - 1];
    expect(latestMessage?.content).toBe("Tagging two offers");
    expect(latestMessage?.contexts?.length).toBe(2);
  });

  it("should fail to send a message with more than 5 context offers", async () => {
    // 1. Create thread
    const createRes = await app.handle(
      new Request("http://localhost/api/messages/threads", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ participantId: user2Id }),
      }),
    );
    const createBody = (await createRes.json()) as { data?: { id?: string } };
    const threadId = createBody.data?.id;

    // Get 6 offers from user1
    const userOffers = await db.select().from(offers).where(eq(offers.userId, user1Id)).limit(6);
    const offerIds = userOffers.map((o) => o.id);

    // 2. Send message
    const sendRes = await app.handle(
      new Request(`http://localhost/api/messages/threads/${threadId}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${user1Token}`,
        },
        body: JSON.stringify({ content: "Tagging six offers", contextOfferIds: offerIds }),
      }),
    );
    expect(sendRes.status).toBe(422); // TooManyContextsError
  });
});

import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { app } from "../../../index";
import { db } from "../../../database/db";
import { users, accounts, sessions } from "../../../database/schema";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";
import { tokenService } from "../../../auth/infrastructure/services/token.service";

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
      .values({ id: user1Id, accountId: account1Id, username: `e2e1_${randomUUID()}` });
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
      .values({ id: user2Id, accountId: account2Id, username: `e2e2_${randomUUID()}` });
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: account2Id,
      token: tokenService.hash(user2Token),
      expiresAt: new Date(Date.now() + 1000000),
    });
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
    const body = await response.json();
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
});
